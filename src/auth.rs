use crate::ResponseExt;
use crate::db::{AppDatabase, Database, User};
use crate::model::UserStatus;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use cookie::{Cookie, SameSite, time::Duration};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;
use webauthn_rs::prelude::*;
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

fn create_session_cookie(token: &str, secure: bool) -> String {
    Cookie::build((SESSION_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax) // Changed to Lax for easier dev/redirects
        .max_age(Duration::days(SESSION_DURATION_DAYS))
        .to_string()
}

fn clear_session_cookie(secure: bool) -> String {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .to_string()
}

fn create_oauth_state_cookie(state: &str, secure: bool) -> String {
    Cookie::build((OAUTH_STATE_COOKIE_NAME, state))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax) // Lax needed for redirect flow
        .max_age(Duration::minutes(OAUTH_STATE_DURATION_MINUTES))
        .to_string()
}

fn clear_oauth_state_cookie(secure: bool) -> String {
    Cookie::build((OAUTH_STATE_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .to_string()
}

fn get_base_url(env: &Env) -> String {
    env.var("BASE_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "http://localhost:8787".to_string())
}

fn is_secure(env: &Env) -> bool {
    get_base_url(env).starts_with("https")
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

static ARGON2_INSTANCE: OnceLock<Argon2> = OnceLock::new();

fn get_argon2_instance() -> &'static Argon2<'static> {
    ARGON2_INSTANCE.get_or_init(Argon2::default)
}

pub fn hash_password(password: &str) -> std::result::Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    get_argon2_instance()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    get_argon2_instance()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
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

    let password_hash = hash_password(&body.password).map_err(Error::RustError)?;
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

    let secure = is_secure(&env);
    Response::from_json(&user)?.add_header("Set-Cookie", &create_session_cookie(&token, secure))
}

pub async fn handle_login(mut req: Request, env: Env) -> Result<Response> {
    let body: LoginRequest = req.json().await?;
    let db = get_db(&env)?;

    let user = db
        .get_user_by_email(&body.email)
        .await?
        .ok_or_else(|| Error::RustError("Invalid credentials".to_string()))?;

    let valid = if let Some(hash_str) = &user.password_hash {
        verify_password(&body.password, hash_str)
    } else {
        false
    };

    if !valid {
        return Response::error("Invalid credentials", 401);
    }

    let token = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    let secure = is_secure(&env);
    Response::from_json(&user)?.add_header("Set-Cookie", &create_session_cookie(&token, secure))
}

pub async fn handle_logout(req: Request, env: Env) -> Result<Response> {
    if let Some((_, token)) = get_auth(&req, &env).await? {
        let db = get_db(&env)?;
        db.delete_session(&token).await?;
    }
    let secure = is_secure(&env);
    Response::ok("Logged out")?.add_header("Set-Cookie", &clear_session_cookie(secure))
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
        if !verify_password(&old_password, hash_str) {
            return Response::error("Invalid old password", 401);
        }
    }

    let new_password_hash = hash_password(&body.new_password).map_err(Error::RustError)?;
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

    let secure = base_url.starts_with("https");
    Response::redirect(Url::parse(&url)?)?
        .add_header("Set-Cookie", &create_oauth_state_cookie(&state, secure))
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
        let secure = base_url.starts_with("https");

        let mut resp = Response::redirect(Url::parse(&base_url)?)?;
        resp.headers_mut()
            .append("Set-Cookie", &create_session_cookie(&token, secure))?;
        resp.headers_mut()
            .append("Set-Cookie", &clear_oauth_state_cookie(secure))?;
        Ok(resp)
    } else {
        Response::error("Missing code", 400)
    }
}

// Passkey Logic
fn get_webauthn(env: &Env) -> Result<Webauthn> {
    let base_url_str = get_base_url(env);
    let base_url = Url::parse(&base_url_str)?;
    let host = base_url.host_str().unwrap_or("localhost");
    let rp_id = host;
    let origin = Url::parse(&base_url_str)?;

    let builder = WebauthnBuilder::new(rp_id, &origin).map_err(|e| Error::RustError(e.to_string()))?;
    builder.build().map_err(|e| Error::RustError(e.to_string()))
}

#[derive(Deserialize)]
struct PasskeyRegisterFinishRequest {
    state_id: String,
    register_response: RegisterPublicKeyCredential,
    name: Option<String>,
}

pub async fn handle_passkey_register_start(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let webauthn = get_webauthn(&env)?;
    // Deterministic UUID for user handle based on ID
    let user_uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, user.id.to_string().as_bytes());

    let (ccr, state) = webauthn
        .start_passkey_registration(user_uuid, &user.username, &user.username, None)
        .map_err(|e| Error::RustError(e.to_string()))?;

    let state_json = serde_json::to_string(&state)?;
    let state_id = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (5 * 60 * 1000); // 5 mins

    let db = get_db(&env)?;
    db.save_passkey_state(&state_id, &state_json, expires_at).await?;

    let response = serde_json::json!({
        "state_id": state_id,
        "options": ccr
    });

    Response::from_json(&response)
}

pub async fn handle_passkey_register_finish(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let body: PasskeyRegisterFinishRequest = req.json().await?;
    let db = get_db(&env)?;

    let state_record = db
        .get_passkey_state(&body.state_id)
        .await?
        .ok_or_else(|| Error::RustError("Invalid or expired state".to_string()))?;

    let state: PasskeyRegistration = serde_json::from_str(&state_record.state)?;
    let webauthn = get_webauthn(&env)?;

    let passkey = webauthn
        .finish_passkey_registration(&body.register_response, &state)
        .map_err(|e| Error::RustError(e.to_string()))?;

    // Store in DB
    let cred_id_b64 = URL_SAFE_NO_PAD.encode(&passkey.cred_id());
    let public_key_json = serde_json::to_string(&passkey.get_public_key())?;

    // Extract counter and aaguid via serialization to ensure compatibility
    let passkey_val = serde_json::to_value(&passkey)?;
    let counter = passkey_val.get("counter").and_then(|v| v.as_u64()).unwrap_or(0);
    let aaguid = passkey_val.get("aaguid").and_then(|v| v.as_str()).map(|s| s.to_string());

    db.create_passkey(
        user.id,
        &cred_id_b64,
        &public_key_json,
        counter as i64,
        body.name.as_deref(),
        aaguid.as_deref(),
    )
    .await?;

    // Cleanup state
    db.delete_passkey_state(&body.state_id).await?;

    Response::ok("Passkey registered")
}

// Passkey Login

#[derive(Deserialize)]
struct PasskeyLoginStartRequest {
    email: String,
}

#[derive(Deserialize)]
struct PasskeyLoginFinishRequest {
    state_id: String,
    login_response: PublicKeyCredential,
}

#[derive(Serialize, Deserialize)]
struct PasskeyLoginState {
    user_id: i32,
    webauthn_state: PasskeyAuthentication,
}

fn db_passkey_to_webauthn(db_pk: &crate::db::Passkey) -> Result<Passkey, Error> {
    let cred_id_bytes = URL_SAFE_NO_PAD.decode(&db_pk.cred_id).map_err(|_| Error::RustError("Bad cred_id".into()))?;
    let public_key: serde_json::Value = serde_json::from_str(&db_pk.public_key)?;

    // Try to reconstruct Passkey from JSON.
    // We assume standard field names: cred_id, key, counter, aaguid.
    let json = serde_json::json!({
        "cred_id": cred_id_bytes,
        "key": public_key,
        "counter": db_pk.counter,
        "aaguid": db_pk.aaguid
    });

    serde_json::from_value(json).map_err(|e| Error::RustError(format!("Failed to reconstruct passkey: {}", e)))
}

pub async fn handle_passkey_login_start(mut req: Request, env: Env) -> Result<Response> {
    let body: PasskeyLoginStartRequest = req.json().await?;
    let db = get_db(&env)?;

    let user = db
        .get_user_by_email(&body.email)
        .await?
        .ok_or_else(|| Error::RustError("User not found".to_string()))?;

    let db_passkeys = db.get_passkeys_by_user(user.id).await?;
    if db_passkeys.is_empty() {
         return Response::error("No passkeys found for user", 400);
    }

    let mut webauthn_passkeys = Vec::new();
    for pk in &db_passkeys {
        webauthn_passkeys.push(db_passkey_to_webauthn(pk)?);
    }

    let webauthn = get_webauthn(&env)?;
    let (rcr, state) = webauthn
        .start_passkey_authentication(&webauthn_passkeys)
        .map_err(|e| Error::RustError(e.to_string()))?;

    let full_state = PasskeyLoginState {
        user_id: user.id,
        webauthn_state: state,
    };
    let state_json = serde_json::to_string(&full_state)?;
    let state_id = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (5 * 60 * 1000);

    db.save_passkey_state(&state_id, &state_json, expires_at).await?;

    let response = serde_json::json!({
        "state_id": state_id,
        "options": rcr
    });

    Response::from_json(&response)
}

pub async fn handle_passkey_login_finish(mut req: Request, env: Env) -> Result<Response> {
    let body: PasskeyLoginFinishRequest = req.json().await?;
    let db = get_db(&env)?;

    let state_record = db
        .get_passkey_state(&body.state_id)
        .await?
        .ok_or_else(|| Error::RustError("Invalid or expired state".to_string()))?;

    let full_state: PasskeyLoginState = serde_json::from_str(&state_record.state)?;
    let webauthn = get_webauthn(&env)?;

    let (user_id, webauthn_state) = (full_state.user_id, full_state.webauthn_state);

    let user = db.get_user_by_id(user_id).await?
        .ok_or_else(|| Error::RustError("User not found".to_string()))?;

    // Finish authentication
    let auth_result = webauthn
        .finish_passkey_authentication(&body.login_response, &webauthn_state)
        .map_err(|e| Error::RustError(e.to_string()))?;

    // Update counter
    // `auth_result` contains `cred_id` and `counter`.
    // Find the used passkey from DB (using cred_id) and update it.
    // auth_result.cred_id is likely `CredentialID` (bytes).
    // We stored it as base64 in DB.

    let used_cred_id_b64 = URL_SAFE_NO_PAD.encode(&auth_result.cred_id());

    // Update DB
    db.update_passkey_counter(&used_cred_id_b64, auth_result.counter() as i64, Date::now().as_millis() as i64).await?;

    // Cleanup state
    db.delete_passkey_state(&body.state_id).await?;

    // Issue Session
    let token = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    let secure = is_secure(&env);
    Response::from_json(&user)?.add_header("Set-Cookie", &create_session_cookie(&token, secure))
}

pub async fn handle_get_passkeys(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let db = get_db(&env)?;
    let passkeys = db.get_passkeys_by_user(user.id).await?;

    // Return sanitized list (no public key, no counter needed for UI usually, but okay to send)
    // Passkey struct is serializable.
    Response::from_json(&passkeys)
}

pub async fn handle_delete_passkey(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let url = req.url()?;
    let path_segments: Vec<&str> = url.path_segments().unwrap().collect();
    // /api/user/passkeys/:id
    let id_str = path_segments.last().unwrap();
    let id = id_str.parse::<i32>().map_err(|_| Error::RustError("Invalid ID".into()))?;

    let db = get_db(&env)?;
    db.delete_passkey(id, user.id).await?;

    Response::ok("Deleted")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "mysecretpassword";
        let hash = hash_password(password).expect("Hashing failed");
        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrongpassword", &hash));
    }

    #[test]
    fn test_verify_invalid_hash() {
        assert!(!verify_password("password", "invalidhash"));
        assert!(!verify_password(
            "password",
            "$2y$12$invalidbcrpythashformat"
        ));
    }

    #[test]
    fn test_cookie_secure_flag() {
        let token = "test_token";
        let secure_cookie = create_session_cookie(token, true);
        assert!(secure_cookie.contains("Secure"));
        assert!(secure_cookie.contains("HttpOnly"));
        assert!(secure_cookie.contains("SameSite=Lax"));

        let insecure_cookie = create_session_cookie(token, false);
        assert!(!insecure_cookie.contains("Secure"));
        assert!(insecure_cookie.contains("HttpOnly"));

        let secure_clear = clear_session_cookie(true);
        assert!(secure_clear.contains("Secure"));
        assert!(secure_clear.contains("Max-Age=0"));

        let state = "oauth_state";
        let secure_oauth = create_oauth_state_cookie(state, true);
        assert!(secure_oauth.contains("Secure"));

        let insecure_oauth = create_oauth_state_cookie(state, false);
        assert!(!insecure_oauth.contains("Secure"));
    }
}
