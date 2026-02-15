use worker::*;
use crate::db::{Database, AppDatabase, User, Session, UserItem};
use bcrypt::{DEFAULT_COST, hash, verify};
use uuid::Uuid;
use cookie::{Cookie, SameSite, time::Duration};
use serde::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;

const SESSION_COOKIE_NAME: &str = "housou_session";
const SESSION_DURATION_DAYS: i64 = 30;

// Helper to get DB
fn get_db(env: &Env) -> Result<AppDatabase> {
    let d1 = env.d1("DB")?;
    Ok(AppDatabase::new(d1))
}

// Helper to get authenticated user
async fn get_auth(req: &Request, env: &Env) -> Result<Option<(User, String)>> {
    let db = get_db(env)?;
    let cookies_header = req.headers().get("Cookie")?.unwrap_or_default();

    // Simple cookie parsing
    for cookie_str in cookies_header.split(';') {
        if let Ok(cookie) = Cookie::parse(cookie_str.trim()) {
            if cookie.name() == SESSION_COOKIE_NAME {
                let token = cookie.value();
                if let Some(session) = db.get_session(token).await? {
                    if let Some(user) = db.get_user_by_id(session.user_id).await? {
                        return Ok(Some((user, token.to_string())));
                    }
                }
            }
        }
    }
    Ok(None)
}

fn create_session_cookie(token: &str) -> String {
    Cookie::build((SESSION_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::days(SESSION_DURATION_DAYS))
        .to_string()
}

fn clear_session_cookie() -> String {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(0))
        .to_string()
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
}

#[derive(Deserialize)]
struct UpdateItemRequest {
    item_id: String,
    status: i32,
    score: Option<i32>,
}

pub async fn handle_register(mut req: Request, env: Env) -> Result<Response> {
    let body: RegisterRequest = req.json().await?;
    let db = get_db(&env)?;

    if db.get_user_by_email(&body.email).await?.is_some() {
        return Response::error("Email already registered", 400);
    }

    let password_hash = hash(&body.password, DEFAULT_COST).map_err(|e| Error::RustError(e.to_string()))?;
    let user = db.create_user(&body.email, &body.username, Some(&password_hash), None).await?;

    // Auto login
    let token = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    Response::from_json(&user)?
        .with_header("Set-Cookie", &create_session_cookie(&token))
}

pub async fn handle_login(mut req: Request, env: Env) -> Result<Response> {
    let body: LoginRequest = req.json().await?;
    let db = get_db(&env)?;

    let user = db.get_user_by_email(&body.email).await?
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

    Response::from_json(&user)?
        .with_header("Set-Cookie", &create_session_cookie(&token))
}

pub async fn handle_logout(req: Request, env: Env) -> Result<Response> {
    if let Some((_, token)) = get_auth(&req, &env).await? {
        let db = get_db(&env)?;
        db.delete_session(&token).await?;
    }
    Response::ok("Logged out")?.with_header("Set-Cookie", &clear_session_cookie())
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

    db.update_username(user.id, &body.username).await?;

    // Return updated user
    let updated_user = db.get_user_by_id(user.id).await?.unwrap();
    Response::from_json(&updated_user)
}

pub async fn handle_update_item(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let body: UpdateItemRequest = req.json().await?;
    let db = get_db(&env)?;

    db.update_user_item(user.id, &body.item_id, body.status, body.score).await?;
    Response::ok("Updated")
}

pub async fn handle_get_item(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let url = req.url()?;
    let item_id = url.query_pairs().find(|(k, _)| k == "item_id").map(|(_, v)| v.to_string());

    if let Some(id) = item_id {
        let db = get_db(&env)?;
        let item = db.get_user_item(user.id, &id).await?;
        Response::from_json(&item)
    } else {
        Response::error("Missing item_id", 400)
    }
}

// GitHub OAuth

#[derive(Deserialize)]
struct GithubUser {
    id: i64,
    login: String, // username
    email: Option<String>,
}

#[derive(Deserialize)]
struct GithubTokenResponse {
    access_token: String,
}

pub async fn handle_github_authorize(_req: Request, env: Env) -> Result<Response> {
    let client_id = env.var("GITHUB_CLIENT_ID")?.to_string();

    let base_url = env.var("BASE_URL").map(|s| s.to_string()).unwrap_or_else(|_| "http://localhost:8787".to_string());
    let redirect_uri = format!("{}/api/auth/github/callback", base_url);

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email",
        client_id, redirect_uri
    );

    Response::redirect(Url::parse(&url)?)
}

pub async fn handle_github_callback(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let code = url.query_pairs().find(|(k, _)| k == "code").map(|(_, v)| v.to_string());

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

        let mut headers = Headers::new();
        headers.set("Accept", "application/json")?;
        headers.set("Content-Type", "application/json")?;
        headers.set("User-Agent", "housou-worker")?;

        let init = RequestInit {
            method: Method::Post,
            headers: headers,
            body: Some(JsValue::from_str(&body.to_string())),
            ..Default::default()
        };

        let req_post = Request::new_with_init(token_url, &init)?;
        let mut resp = Fetch::Request(req_post).send().await?;

        if resp.status_code() != 200 {
            return Response::error(format!("GitHub Token Error: {}", resp.status_code()), 500);
        }

        let token_data: GithubTokenResponse = resp.json().await?;

        // Get user info
        let user_url = "https://api.github.com/user";
        let mut headers = Headers::new();
        headers.set("Authorization", &format!("Bearer {}", token_data.access_token))?;
        headers.set("User-Agent", "housou-worker")?;
        headers.set("Accept", "application/json")?;

        let init = RequestInit {
            method: Method::Get,
            headers: headers,
            ..Default::default()
        };

        let req_get = Request::new_with_init(user_url, &init)?;
        let mut user_resp = Fetch::Request(req_get).send().await?;

        if user_resp.status_code() != 200 {
             return Response::error(format!("GitHub User Error: {}", user_resp.status_code()), 500);
        }

        let gh_user: GithubUser = user_resp.json().await?;
        let gh_id_str = gh_user.id.to_string();

        let db = get_db(&env)?;

        // Find or create user
        let user = if let Some(u) = db.get_user_by_github_id(&gh_id_str).await? {
            u
        } else {
             // Link or Create
             let email = gh_user.email.clone().unwrap_or_else(|| format!("{}@github.com", gh_user.login));

             if let Some(_) = db.get_user_by_email(&email).await? {
                 return Response::error("Email already in use", 400);
             }

             db.create_user(&email, &gh_user.login, None, Some(&gh_id_str)).await?
        };

        // Create session
        let token = Uuid::new_v4().to_string();
        let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
        db.create_session(user.id, &token, expires_at).await?;

        // Redirect to home
        let base_url = env.var("BASE_URL").map(|s| s.to_string()).unwrap_or_else(|_| "/".to_string());

        Response::redirect(Url::parse(&base_url)?)?
            .with_header("Set-Cookie", &create_session_cookie(&token))
    } else {
        Response::error("Missing code", 400)
    }
}
