use crate::ResponseExt;
use crate::db::{AppDatabase, Database, User};
use crate::model::UserStatus;
use bcrypt::{DEFAULT_COST, hash, verify};
use cookie::{Cookie, SameSite, time::Duration};
use serde::Deserialize;
use uuid::Uuid;
use worker::wasm_bindgen::JsValue;
use worker::*;

const SESSION_COOKIE_NAME: &str = "housou_session";
const SESSION_DURATION_DAYS: i64 = 30;
const OAUTH_STATE_COOKIE_NAME: &str = "oauth_state";
const OAUTH_STATE_DURATION_MINUTES: i64 = 5;

// Helper to get DB
pub fn get_db(env: &Env) -> Result<AppDatabase> {
    let d1 = env.d1("DB")?;
    Ok(AppDatabase::new(d1))
}

// Helper to get authenticated user
pub async fn get_auth(req: &Request, env: &Env) -> Result<Option<(User, String)>> {
    let db = get_db(env)?;
    let cookies_header = req.headers().get("Cookie")?.unwrap_or_default();

    // Parse cookies more robustly using cookie crate
    for cookie in Cookie::split_parse(cookies_header).filter_map(Result::ok) {
        if cookie.name() == SESSION_COOKIE_NAME {
            let token = cookie.value();
            if let Some(user) = db.get_user_by_session_token(token).await? {
                return Ok(Some((user, token.to_string())));
            } else {
                console_log!("Auth failed for token: {}", token);
            }
        }
    }
    Ok(None)
}

fn create_session_cookie(token: &str) -> String {
    Cookie::build((SESSION_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .secure(false) // Changed for dev environment
        .same_site(SameSite::Lax) // Changed to Lax for easier dev/redirects
        .max_age(Duration::days(SESSION_DURATION_DAYS))
        .to_string()
}

fn clear_session_cookie() -> String {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .to_string()
}

fn create_oauth_state_cookie(state: &str) -> String {
    Cookie::build((OAUTH_STATE_COOKIE_NAME, state))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax) // Lax needed for redirect flow
        .max_age(Duration::minutes(OAUTH_STATE_DURATION_MINUTES))
        .to_string()
}

fn clear_oauth_state_cookie() -> String {
    Cookie::build((OAUTH_STATE_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .to_string()
}

fn get_base_url(env: &Env) -> String {
    env.var("BASE_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "http://localhost:8787".to_string())
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    username: String,
    email: Option<String>,
    avatar_url: Option<String>,
}
#[derive(Deserialize)]
struct ChangePasswordRequest {
    old_password: Option<String>,
    new_password: String,
}
#[derive(Deserialize)]
struct UpdateItemRequest {
    title: String,
    status: UserStatus,
    score: Option<i32>,
}

pub async fn handle_register(mut req: Request, env: Env) -> Result<Response> {
    let body: RegisterRequest = req.json().await?;
    let db = get_db(&env)?;

    if (db.get_user_by_email(&body.email).await?).is_some() {
        return Response::error("Email already registered", 400);
    }
    if (db.get_user_by_username(&body.username).await?).is_some() {
        return Response::error("Username already taken", 400);
    }

    let password_hash =
        hash(&body.password, DEFAULT_COST).map_err(|e| Error::RustError(e.to_string()))?;
    let user = db
        .create_user(
            &body.email,
            &body.username,
            Some(&password_hash),
            None,
            None,
        )
        .await?;

    // Auto login
    let token = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    Response::from_json(&user)?.add_header("Set-Cookie", &create_session_cookie(&token))
}

pub async fn handle_login(mut req: Request, env: Env) -> Result<Response> {
    let body: LoginRequest = req.json().await?;
    let db = get_db(&env)?;

    let user = db
        .get_user_by_email(&body.email)
        .await?
        .ok_or_else(|| Error::RustError("Invalid credentials".to_string()))?;

    let valid = if let Some(hash_str) = &user.password_hash {
        verify(&body.password, hash_str).unwrap_or(false)
    } else {
        false
    };

    if !valid {
        return Response::error("Invalid credentials", 401);
    }

    let token = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    Response::from_json(&user)?.add_header("Set-Cookie", &create_session_cookie(&token))
}

pub async fn handle_logout(req: Request, env: Env) -> Result<Response> {
    if let Some((_, token)) = get_auth(&req, &env).await? {
        let db = get_db(&env)?;
        db.delete_session(&token).await?;
    }
    Response::ok("Logged out")?.add_header("Set-Cookie", &clear_session_cookie())
}

pub async fn handle_me(req: Request, env: Env) -> Result<Response> {
    match get_auth(&req, &env).await? {
        Some((user, _)) => Response::from_json(&user),
        None => Response::error("Unauthorized", 401),
    }
}

pub async fn handle_update_profile(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let body: UpdateProfileRequest = req.json().await?;
    let db = get_db(&env)?;

    // Check unique username if changed
    if body.username != user.username
        && let Some(existing) = db.get_user_by_username(&body.username).await?
        && existing.id != user.id
    {
        return Response::error("Username already taken", 409);
    }

    // Check unique email if changed and provided
    if let Some(email) = &body.email
        && email != &user.email
        && let Some(existing) = db.get_user_by_email(email).await?
        && existing.id != user.id
    {
        return Response::error("Email already in use", 409);
    }

    db.update_user_profile(
        user.id,
        &body.username,
        body.email.as_deref(),
        body.avatar_url.as_deref(),
    )
    .await?;

    // Return updated user safely
    let updated_user = db
        .get_user_by_id(user.id)
        .await?
        .ok_or_else(|| Error::RustError("User not found after update".to_string()))?;
    Response::from_json(&updated_user)
}

pub async fn handle_change_password(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let body: ChangePasswordRequest = req.json().await?;

    let db = get_db(&env)?;

    // If user has a password (not GitHub-only), verify the old one
    if let Some(hash_str) = &user.password_hash {
        let old_password = body
            .old_password
            .ok_or_else(|| Error::RustError("Old password required".to_string()))?;
        let valid = verify(&old_password, hash_str).unwrap_or(false);
        if !valid {
            return Response::error("Invalid old password", 401);
        }
    }

    let new_password_hash =
        hash(&body.new_password, DEFAULT_COST).map_err(|e| Error::RustError(e.to_string()))?;
    db.update_user_password(user.id, &new_password_hash).await?;

    Response::ok("Password updated")
}

pub async fn handle_update_item(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let body: UpdateItemRequest = req.json().await?;
    let db = get_db(&env)?;

    db.update_user_item(user.id, &body.title, body.status, body.score)
        .await?;
    Response::ok("Updated")
}

pub async fn handle_get_item(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let url = req.url()?;
    let title = url
        .query_pairs()
        .find(|(k, _)| k == "title")
        .map(|(_, v)| v.to_string());

    if let Some(t) = title {
        let db = get_db(&env)?;
        let item = db.get_user_item(user.id, &t).await?;
        Response::from_json(&item)
    } else {
        Response::error("Missing title", 400)
    }
}

// GitHub OAuth

#[derive(Deserialize)]
struct GithubUser {
    id: i64,
    login: String, // username
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct GithubTokenResponse {
    access_token: String,
}

pub async fn handle_github_authorize(_req: Request, env: Env) -> Result<Response> {
    let client_id = env.var("GITHUB_CLIENT_ID")?.to_string();
    let base_url = get_base_url(&env);
    let redirect_uri = format!("{base_url}/api/auth/github/callback");

    // CSRF Protection: Generate State
    let state = Uuid::new_v4().to_string();

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={client_id}&redirect_uri={redirect_uri}&scope=user:email&state={state}"
    );

    Response::redirect(Url::parse(&url)?)?
        .add_header("Set-Cookie", &create_oauth_state_cookie(&state))
}

pub async fn handle_github_callback(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let query_params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let code = query_params.get("code").cloned();
    let state = query_params.get("state").cloned();

    // Verify State (CSRF)
    let cookies_header = req.headers().get("Cookie")?.unwrap_or_default();
    let mut stored_state = None;

    for cookie in Cookie::split_parse(cookies_header).filter_map(Result::ok) {
        if cookie.name() == OAUTH_STATE_COOKIE_NAME {
            stored_state = Some(cookie.value().to_string());
            break;
        }
    }

    if state.is_none() || stored_state.is_none() || state != stored_state {
        return Response::error("Invalid or missing OAuth state", 403);
    }

    if let Some(code) = code {
        let client_id = env.var("GITHUB_CLIENT_ID")?.to_string();
        let client_secret = env.var("GITHUB_CLIENT_SECRET")?.to_string();

        // Exchange code for token
        let token_url = "https://github.com/login/oauth/access_token";
        let body = serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code
        });

        let headers = Headers::new();
        headers.set("Accept", "application/json")?;
        headers.set("Content-Type", "application/json")?;
        headers.set("User-Agent", "housou-worker")?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_headers(headers);
        init.with_body(Some(JsValue::from_str(&body.to_string())));

        let req_post = Request::new_with_init(token_url, &init)?;
        let mut resp = Fetch::Request(req_post).send().await?;

        if resp.status_code() != 200 {
            return Response::error(format!("GitHub Token Error: {}", resp.status_code()), 500);
        }

        let token_data: GithubTokenResponse = resp.json().await?;

        // Get user info
        let user_url = "https://api.github.com/user";
        let headers = Headers::new();
        headers.set(
            "Authorization",
            &format!("Bearer {}", token_data.access_token),
        )?;
        headers.set("User-Agent", "housou-worker")?;
        headers.set("Accept", "application/json")?;

        let mut init = RequestInit::new();
        init.with_method(Method::Get);
        init.with_headers(headers);

        let req_get = Request::new_with_init(user_url, &init)?;
        let mut user_resp = Fetch::Request(req_get).send().await?;

        if user_resp.status_code() != 200 {
            return Response::error(
                format!("GitHub User Error: {}", user_resp.status_code()),
                500,
            );
        }

        let gh_user: GithubUser = user_resp.json().await?;
        let gh_id_str = gh_user.id.to_string();

        let db = get_db(&env)?;

        // Find or create user
        let user = if let Some(u) = db.get_user_by_github_id(&gh_id_str).await? {
            u
        } else {
            let email = gh_user
                .email
                .clone()
                .unwrap_or_else(|| format!("{}@github.com", gh_user.login));

            if (db.get_user_by_email(&email).await?).is_some() {
                return Response::error("Email already in use", 400);
            }
            if (db.get_user_by_username(&gh_user.login).await?).is_some() {
                return Response::error("Username already taken", 400);
            }

            db.create_user(
                &email,
                &gh_user.login,
                None,
                Some(&gh_id_str),
                gh_user.avatar_url.as_deref(),
            )
            .await?
        };

        // Create session
        let token = Uuid::new_v4().to_string();
        let expires_at =
            Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
        db.create_session(user.id, &token, expires_at).await?;

        // Redirect to home
        let base_url = get_base_url(&env);

        let mut resp = Response::redirect(Url::parse(&base_url)?)?;
        resp.headers_mut()
            .append("Set-Cookie", &create_session_cookie(&token))?;
        resp.headers_mut()
            .append("Set-Cookie", &clear_oauth_state_cookie())?;
        Ok(resp)
    } else {
        Response::error("Missing code", 400)
    }
}
