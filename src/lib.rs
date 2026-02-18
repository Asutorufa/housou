use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use worker::*;

mod auth;
mod config;
mod db;
mod handlers;
mod model;
mod passkey;
mod provider;
mod utils;
use db::Database; // Import Database trait

pub trait ResponseExt {
    fn add_cors(self, env: &Env) -> Result<Response>;
    fn add_header(self, key: &str, value: &str) -> Result<Response>;
    fn add_security_headers(self) -> Result<Response>;
}

static CORS_ALLOWED_ORIGIN: OnceLock<String> = OnceLock::new();
static MIGRATION_DONE: AtomicBool = AtomicBool::new(false);

impl ResponseExt for Response {
    fn add_cors(mut self, env: &Env) -> Result<Response> {
        let allowed_origin = CORS_ALLOWED_ORIGIN.get_or_init(|| {
            env.var("CORS_ALLOWED_ORIGIN")
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "*".to_string())
        });
        self.headers_mut()
            .set("Access-Control-Allow-Origin", allowed_origin)?;
        Ok(self)
    }

    fn add_header(mut self, key: &str, value: &str) -> Result<Response> {
        self.headers_mut().set(key, value)?;
        Ok(self)
    }

    fn add_security_headers(self) -> Result<Response> {
        self.add_header(
            "Content-Security-Policy",
            "default-src 'none'; frame-ancestors 'none';",
        )?
        .add_header("X-Content-Type-Options", "nosniff")?
        .add_header("X-Frame-Options", "DENY")
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Migration Logic (Lazy)
    if let Ok(d1) = env.d1("DB")
        && MIGRATION_DONE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    {
        let db = db::AppDatabase::new(d1);
        // We ignore migration errors here to not block the whole app,
        // but ideally we should log them.
        if let Err(e) = db.migrate().await {
            console_error!("Migration failed: {}", e);
            MIGRATION_DONE.store(false, Ordering::Relaxed);
        }
    }

    // Handle request logic (caching and routing)
    let resp_result = handle_request_logic(req, env.clone()).await;

    let resp = match resp_result {
        Ok(r) => r,
        Err(e) => {
            console_error!("Request failed: {}", e);
            Response::error(e.to_string(), 500)?
        }
    };

    // Add security headers to ALL responses
    resp.add_security_headers()?.add_cors(&env)
}

async fn handle_request_logic(req: Request, env: Env) -> Result<Response> {
    let cache = Cache::open(format!("housou-cache-{}", config::CACHE_VERSION)).await;
    let url = req.url()?;

    if req.method() == Method::Get {
        // Skip caching for auth routes and items when auth is enabled (user specific data)
        // Actually, we should check auth cookie presence before skipping cache, or make items endpoint handle caching carefully.
        // For simplicity: skip caching items endpoint if auth cookie is present or if we want to be safe.
        // But the router handles caching internally for items? No, router returns Response.
        // The main block handles caching.

        let is_auth_route =
            url.path().starts_with("/api/auth") || url.path().starts_with("/api/user");
        // We also want to skip caching /api/items if the user is authenticated, because the response is personalized.
        // Checking for cookie presence is a simple heuristic.
        let has_session_cookie = req
            .headers()
            .get("Cookie")?
            .unwrap_or_default()
            .contains("housou_session");

        if is_auth_route || (url.path() == "/api/items" && has_session_cookie) {
            router(req, env.clone()).await
        } else if let Ok(Some(mut cached_resp)) = cache.get(url.as_str(), true).await {
            // Use cached response, clone to make it mutable for adding security headers
            cached_resp.cloned()
        } else {
            // Generate new response
            let mut fresh_resp = router(req, env.clone()).await?;

            // Cache successful GET responses (except auth and personalized items)
            if url.path().starts_with("/api")
                && !is_auth_route
                && !(url.path() == "/api/items" && has_session_cookie)
                && fresh_resp.status_code() == 200
            {
                if !fresh_resp.headers().has("Cache-Control")? {
                    fresh_resp = fresh_resp.add_header(
                        "Cache-Control",
                        &format!("public, max-age={}", config::CACHE_TTL_API),
                    )?;
                }
                let _ = cache.put(url.as_str(), fresh_resp.cloned()?).await;
            }
            Ok(fresh_resp)
        }
    } else {
        router(req, env.clone()).await
    }
}

async fn router(req: Request, env: Env) -> Result<Response> {
    let method = req.method();
    let path = req.path();

    // Check if Auth is enabled (DB binding exists)
    let auth_enabled = env.d1("DB").is_ok();

    match (method.clone(), path.as_str()) {
        (Method::Get, "/api/config") => handlers::handle_config(req, env).await,
        (Method::Get, "/api/items") => handlers::handle_items(req, env).await,
        (Method::Post, "/api/user/status") if auth_enabled => {
            handlers::handle_user_status(req, env).await
        }
        (Method::Get, "/api/metadata") => handlers::handle_metadata(req, env).await,
        // Auth Routes (Only if enabled)
        (Method::Post, "/api/auth/register") if auth_enabled => {
            auth::handle_register(req, env.clone()).await
        }
        (Method::Post, "/api/auth/login") if auth_enabled => {
            auth::handle_login(req, env.clone()).await
        }
        (Method::Post, "/api/auth/logout") if auth_enabled => {
            auth::handle_logout(req, env.clone()).await
        }
        (Method::Get, "/api/auth/me") if auth_enabled => auth::handle_me(req, env.clone()).await,
        (Method::Put, "/api/auth/profile") if auth_enabled => {
            auth::handle_update_profile(req, env.clone()).await
        }
        (Method::Put, "/api/auth/password") if auth_enabled => {
            auth::handle_change_password(req, env.clone()).await
        }
        (Method::Get, "/api/auth/github/authorize") if auth_enabled => {
            auth::handle_github_authorize(req, env.clone()).await
        }
        (Method::Get, "/api/auth/github/callback") if auth_enabled => {
            auth::handle_github_callback(req, env.clone()).await
        }
        (Method::Get, "/api/user/item") if auth_enabled => {
            auth::handle_get_item(req, env.clone()).await
        }
        (Method::Post, "/api/user/item") if auth_enabled => {
            auth::handle_update_item(req, env.clone()).await
        }

        // Passkey Routes
        (Method::Post, "/api/auth/passkey/register/start") if auth_enabled => {
            passkey::handle_register_start(req, env.clone()).await
        }
        (Method::Post, "/api/auth/passkey/register/finish") if auth_enabled => {
            passkey::handle_register_finish(req, env.clone()).await
        }
        (Method::Post, "/api/auth/passkey/login/start") if auth_enabled => {
            passkey::handle_login_start(req, env.clone()).await
        }
        (Method::Post, "/api/auth/passkey/login/finish") if auth_enabled => {
            passkey::handle_login_finish(req, env.clone()).await
        }
        (Method::Get, "/api/auth/passkey") if auth_enabled => {
            passkey::handle_list(req, env.clone()).await
        }
        (Method::Delete, "/api/auth/passkey") if auth_enabled => {
            passkey::handle_delete(req, env.clone()).await
        }
        (Method::Patch, "/api/auth/passkey") if auth_enabled => {
            passkey::handle_rename(req, env.clone()).await
        }

        // Handle Options for CORS on auth routes
        (Method::Options, path)
            if path.starts_with("/api/auth") || path.starts_with("/api/user") =>
        {
            Response::empty()
        }

        _ => Response::error("Not Found", 404),
    }
}
