use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use worker::*;

mod auth;
mod config;
mod db;
mod handlers;
mod model;
mod provider;
mod utils;
use db::Database; // Import Database trait

pub trait ResponseExt {
    fn add_cors(self, env: &Env) -> Result<Response>;
    fn add_header(self, key: &str, value: &str) -> Result<Response>;
    fn add_security_headers(self) -> Result<Response>;
}

// Abstraction for setting headers to allow testing without worker::Response
pub(crate) trait HeaderSetter {
    fn set_header(&mut self, key: &str, value: &str) -> Result<()>;
}

impl HeaderSetter for Response {
    fn set_header(&mut self, key: &str, value: &str) -> Result<()> {
        self.headers_mut().set(key, value)
    }
}

// Pure logic implementations
fn add_cors_header_impl(setter: &mut impl HeaderSetter, origin: &str) -> Result<()> {
    setter.set_header("Access-Control-Allow-Origin", origin)
}

fn add_header_impl(setter: &mut impl HeaderSetter, key: &str, value: &str) -> Result<()> {
    setter.set_header(key, value)
}

fn add_security_headers_impl(setter: &mut impl HeaderSetter) -> Result<()> {
    setter.set_header(
        "Content-Security-Policy",
        "default-src 'none'; frame-ancestors 'none';",
    )?;
    setter.set_header("X-Content-Type-Options", "nosniff")?;
    setter.set_header("X-Frame-Options", "DENY")?;
    Ok(())
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
        add_cors_header_impl(&mut self, allowed_origin)?;
        Ok(self)
    }

    fn add_header(mut self, key: &str, value: &str) -> Result<Response> {
        add_header_impl(&mut self, key, value)?;
        Ok(self)
    }

    fn add_security_headers(mut self) -> Result<Response> {
        add_security_headers_impl(&mut self)?;
        Ok(self)
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    // Migration Logic (Lazy)
    if let Ok(d1) = env.d1("DB")
        && MIGRATION_DONE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    {
        let db = db::AppDatabase::new(d1);
        // We log migration errors but do not block the whole app.
        if let Err(e) = db.migrate().await {
            console_error!("Migration failed: {}", e);
        }
    }

    // Handle request logic (caching and routing)
    let resp_result = handle_request_logic(req, env.clone(), ctx).await;

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

async fn handle_request_logic(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let cache = Cache::open(format!("housou-cache-{}", config::CACHE_VERSION)).await;
    let url = req.url()?;

    if req.method() == Method::Get {
        let skip_shared_cache = should_skip_shared_cache(url.path());

        if skip_shared_cache {
            router(req, env.clone(), ctx).await
        } else if let Ok(Some(mut cached_resp)) = cache.get(url.as_str(), true).await {
            // Use cached response, clone to make it mutable for adding security headers
            cached_resp.cloned()
        } else {
            // Generate new response
            let mut fresh_resp = router(req, env.clone(), ctx).await?;

            // Cache successful GET responses unless they may vary by viewer.
            if url.path().starts_with("/api")
                && !skip_shared_cache
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
        router(req, env.clone(), ctx).await
    }
}

fn should_skip_shared_cache(path: &str) -> bool {
    path.starts_with("/api/auth")
        || path.starts_with("/api/user")
        || path.starts_with("/api/comments")
}

async fn router(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let auth_enabled = env.d1("DB").is_ok();

    let mut router = Router::with_data(ctx);

    router = router
        .get_async("/api/config", |req, ctx| async move {
            handlers::handle_config(req, ctx.env).await
        })
        .get_async("/api/items", |req, ctx| async move {
            handlers::handle_items(req, ctx.env).await
        })
        .get_async("/api/metadata", |req, ctx| async move {
            handlers::handle_metadata(req, ctx).await
        })
        .post_async("/api/metadata", |req, ctx| async move {
            handlers::handle_metadata(req, ctx).await
        })
        .get_async("/api/favicon", |req, ctx| async move {
            handlers::handle_favicon(req, ctx.env).await
        })
        .get_async("/api/comments", |req, ctx| async move {
            handlers::handle_get_comments(req, ctx.env).await
        });

    if auth_enabled {
        router = router
            .get_async("/api/user/status", |req, ctx| async move {
                handlers::handle_user_status(req, ctx.env).await
            })
            .post_async("/api/auth/register", |req, ctx| async move {
                auth::handle_register(req, ctx.env).await
            })
            .post_async("/api/auth/login", |req, ctx| async move {
                auth::handle_login(req, ctx.env).await
            })
            .post_async("/api/auth/logout", |req, ctx| async move {
                auth::handle_logout(req, ctx.env).await
            })
            .get_async("/api/auth/me", |req, ctx| async move {
                auth::handle_me(req, ctx.env).await
            })
            .put_async("/api/auth/profile", |req, ctx| async move {
                auth::handle_update_profile(req, ctx.env).await
            })
            .put_async("/api/auth/password", |req, ctx| async move {
                auth::handle_change_password(req, ctx.env).await
            })
            .get_async("/api/auth/github/authorize", |req, ctx| async move {
                auth::handle_github_authorize(req, ctx.env).await
            })
            .get_async("/api/auth/github/callback", |req, ctx| async move {
                auth::handle_github_callback(req, ctx.env).await
            })
            .get_async("/api/auth/github/bind", |req, ctx| async move {
                auth::handle_github_bind_authorize(req, ctx.env).await
            })
            .delete_async("/api/auth/github", |req, ctx| async move {
                auth::handle_github_unbind(req, ctx.env).await
            })
            .post_async("/api/auth/telegram/login", |req, ctx| async move {
                auth::handle_telegram_login(req, ctx.env).await
            })
            .post_async("/api/auth/telegram/bind", |req, ctx| async move {
                auth::handle_telegram_bind(req, ctx.env).await
            })
            .delete_async("/api/auth/telegram", |req, ctx| async move {
                auth::handle_telegram_unbind(req, ctx.env).await
            })
            .post_async("/api/user/item", |req, ctx| async move {
                auth::handle_update_item(req, ctx.env).await
            })
            .post_async("/api/auth/passkey/register/start", |req, ctx| async move {
                auth::passkey::handle_register_start(req, ctx.env).await
            })
            .post_async("/api/auth/passkey/register/finish", |req, ctx| async move {
                auth::passkey::handle_register_finish(req, ctx.env).await
            })
            .post_async("/api/auth/passkey/login/start", |req, ctx| async move {
                auth::passkey::handle_login_start(req, ctx.env).await
            })
            .post_async("/api/auth/passkey/login/finish", |req, ctx| async move {
                auth::passkey::handle_login_finish(req, ctx.env).await
            })
            .get_async("/api/auth/passkey", |req, ctx| async move {
                auth::passkey::handle_list(req, ctx.env).await
            })
            .delete_async("/api/auth/passkey", |req, ctx| async move {
                auth::passkey::handle_delete(req, ctx.env).await
            })
            .patch_async("/api/auth/passkey", |req, ctx| async move {
                auth::passkey::handle_rename(req, ctx.env).await
            })
            .post_async("/api/comments", |req, ctx| async move {
                auth::handle_post_comment(req, ctx.env).await
            })
            .delete_async("/api/comments/:id", |req, ctx| async move {
                auth::handle_delete_comment(req, ctx.env).await
            });
    }

    // Handle Options for CORS on auth routes
    router = router
        .options("/api/metadata", |_, _| Response::empty())
        .options("/api/comments", |_, _| Response::empty())
        .options("/api/comments/*path", |_, _| Response::empty())
        .options("/api/user/*path", |_, _| Response::empty())
        .options("/api/auth/*path", |_, _| Response::empty());

    router.run(req, env).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockHeaderSetter {
        headers: HashMap<String, String>,
    }

    impl MockHeaderSetter {
        fn new() -> Self {
            Self {
                headers: HashMap::new(),
            }
        }
    }

    impl HeaderSetter for MockHeaderSetter {
        fn set_header(&mut self, key: &str, value: &str) -> Result<()> {
            self.headers.insert(key.to_string(), value.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_add_header() {
        let mut setter = MockHeaderSetter::new();
        add_header_impl(&mut setter, "Content-Type", "application/json").unwrap();
        assert_eq!(
            setter.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_add_security_headers() {
        let mut setter = MockHeaderSetter::new();
        add_security_headers_impl(&mut setter).unwrap();
        assert_eq!(
            setter.headers.get("Content-Security-Policy"),
            Some(&"default-src 'none'; frame-ancestors 'none';".to_string())
        );
        assert_eq!(
            setter.headers.get("X-Content-Type-Options"),
            Some(&"nosniff".to_string())
        );
        assert_eq!(
            setter.headers.get("X-Frame-Options"),
            Some(&"DENY".to_string())
        );
    }

    #[test]
    fn test_add_cors_header() {
        let mut setter = MockHeaderSetter::new();
        let origin = "https://example.com";
        add_cors_header_impl(&mut setter, origin).unwrap();
        assert_eq!(
            setter.headers.get("Access-Control-Allow-Origin"),
            Some(&origin.to_string())
        );
    }

    #[test]
    fn test_should_skip_shared_cache() {
        assert!(should_skip_shared_cache("/api/auth/me"));
        assert!(should_skip_shared_cache("/api/user/status"));
        assert!(should_skip_shared_cache("/api/comments"));
        assert!(should_skip_shared_cache("/api/comments?title=test"));
        assert!(!should_skip_shared_cache("/api/items"));
        assert!(!should_skip_shared_cache("/api/metadata"));
    }
}
