use crate::auth::{
    EMAIL_IN_USE_ERR, SESSION_DURATION_DAYS, USERNAME_TAKEN_ERR, clear_oauth_action_cookie,
    clear_oauth_state_cookie, create_oauth_action_cookie, create_oauth_state_cookie,
    create_session_cookie, get_auth, get_base_url, get_cookie_values, get_db, verify_oauth_state,
};
use crate::db::{AppDatabase, Database, User};
use serde::Deserialize;
use uuid::Uuid;
use worker::wasm_bindgen::JsValue;
use worker::*;

#[derive(Deserialize)]
pub struct GithubUser {
    pub id: i64,
    pub login: String, // username
    pub email: Option<String>,
    pub avatar_url: Option<String>,
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

    let secure = base_url.starts_with("https");
    let mut resp = Response::redirect(Url::parse(&url)?)?;
    resp.headers_mut()
        .append("Set-Cookie", &create_oauth_state_cookie(&state, secure))?;
    // Clear action cookie just in case
    resp.headers_mut()
        .append("Set-Cookie", &clear_oauth_action_cookie(secure))?;
    Ok(resp)
}

pub async fn handle_github_bind_authorize(req: Request, env: Env) -> Result<Response> {
    // Ensure logged in
    if (get_auth(&req, &env).await?).is_none() {
        return Response::error("Unauthorized", 401);
    }

    let client_id = env.var("GITHUB_CLIENT_ID")?.to_string();
    let base_url = get_base_url(&env);
    let redirect_uri = format!("{base_url}/api/auth/github/callback");

    let state = Uuid::new_v4().to_string();
    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={client_id}&redirect_uri={redirect_uri}&scope=user:email&state={state}"
    );

    let secure = base_url.starts_with("https");
    let mut resp = Response::redirect(Url::parse(&url)?)?;
    resp.headers_mut()
        .append("Set-Cookie", &create_oauth_state_cookie(&state, secure))?;
    resp.headers_mut()
        .append("Set-Cookie", &create_oauth_action_cookie("bind", secure))?;
    Ok(resp)
}

pub async fn handle_github_unbind(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    if user.password_hash.is_none() {
        return Response::error(
            "Password not set. Please set a password before disconnecting GitHub.",
            400,
        );
    }

    let db = get_db(&env)?;
    db.update_user_github_id(user.id, None).await?;

    Response::ok("GitHub account disconnected")
}

async fn exchange_code_for_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String> {
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
        let error_body = resp
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        return Err(Error::RustError(format!(
            "GitHub Token Error: {} - {}",
            resp.status_code(),
            error_body
        )));
    }

    let token_data: GithubTokenResponse = resp.json().await?;
    Ok(token_data.access_token)
}

pub async fn fetch_github_user(access_token: &str) -> Result<GithubUser> {
    let user_url = "https://api.github.com/user";
    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {}", access_token))?;
    headers.set("User-Agent", "housou-worker")?;
    headers.set("Accept", "application/json")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Get);
    init.with_headers(headers);

    let req_get = Request::new_with_init(user_url, &init)?;
    let mut user_resp = Fetch::Request(req_get).send().await?;

    if user_resp.status_code() != 200 {
        let error_body = user_resp
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        return Err(Error::RustError(format!(
            "GitHub User Error: {} - {}",
            user_resp.status_code(),
            error_body
        )));
    }

    let gh_user: GithubUser = user_resp.json().await?;
    Ok(gh_user)
}

async fn find_or_create_github_user(db: &AppDatabase, gh_user: &GithubUser) -> Result<User> {
    let gh_id_str = gh_user.id.to_string();

    if let Some(u) = db.get_user_by_github_id(&gh_id_str).await? {
        Ok(u)
    } else {
        let email = gh_user
            .email
            .clone()
            .unwrap_or_else(|| format!("{}@github.com", gh_user.login));

        if (db.get_user_by_email(&email).await?).is_some() {
            return Err(Error::RustError(EMAIL_IN_USE_ERR.to_string()));
        }
        if (db.get_user_by_username(&gh_user.login).await?).is_some() {
            return Err(Error::RustError(USERNAME_TAKEN_ERR.to_string()));
        }

        let user = db
            .create_user(
                &email,
                &gh_user.login,
                None,
                Some(&gh_id_str),
                gh_user.avatar_url.as_deref(),
            )
            .await?;
        Ok(user)
    }
}

fn get_oauth_action(req: &Request) -> Option<String> {
    get_cookie_values(req, "oauth_action").first().cloned()
}

pub async fn handle_github_callback(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let query_params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let code = query_params.get("code").cloned();
    let state = query_params.get("state").cloned();

    // Verify State (CSRF)
    if verify_oauth_state(&req, state.as_deref()).is_err() {
        return Response::error("Invalid or missing OAuth state", 403);
    }

    let action = get_oauth_action(&req).unwrap_or_else(|| "login".to_string());

    if let Some(code) = code {
        let client_id = env.var("GITHUB_CLIENT_ID")?.to_string();
        let client_secret = env.var("GITHUB_CLIENT_SECRET")?.to_string();

        let access_token = exchange_code_for_token(&code, &client_id, &client_secret).await?;
        let gh_user = fetch_github_user(&access_token).await?;

        let db = get_db(&env)?;
        let base_url = get_base_url(&env);
        let secure = base_url.starts_with("https");

        if action == "bind" {
            // Bind flow
            let (current_user, _) = match get_auth(&req, &env).await? {
                Some(u) => u,
                None => return Response::error("Unauthorized", 401),
            };

            let gh_id_str = gh_user.id.to_string();

            // Check if GitHub ID is already used
            if let Some(existing_user) = db.get_user_by_github_id(&gh_id_str).await? {
                if existing_user.id != current_user.id {
                    return Response::error(
                        "GitHub account already connected to another user",
                        409,
                    );
                }
            } else {
                // Update user
                db.update_user_github_id(current_user.id, Some(&gh_id_str))
                    .await?;
            }

            let mut resp = Response::redirect(Url::parse(&base_url)?)?;
            resp.headers_mut()
                .append("Set-Cookie", &clear_oauth_state_cookie(secure))?;
            resp.headers_mut()
                .append("Set-Cookie", &clear_oauth_action_cookie(secure))?;
            Ok(resp)
        } else {
            // Login/Register flow
            let user = match find_or_create_github_user(&db, &gh_user).await {
                Ok(u) => u,
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains(EMAIL_IN_USE_ERR) || msg.contains(USERNAME_TAKEN_ERR) {
                        return Response::error(msg, 400);
                    }
                    return Err(e);
                }
            };

            // Create session
            let token = Uuid::new_v4().to_string();
            let expires_at =
                Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
            db.create_session(user.id, &token, expires_at).await?;

            let mut resp = Response::redirect(Url::parse(&base_url)?)?;
            resp.headers_mut()
                .append("Set-Cookie", &create_session_cookie(&token, secure))?;
            resp.headers_mut()
                .append("Set-Cookie", &clear_oauth_state_cookie(secure))?;
            resp.headers_mut()
                .append("Set-Cookie", &clear_oauth_action_cookie(secure))?;
            Ok(resp)
        }
    } else {
        Response::error("Missing code", 400)
    }
}
