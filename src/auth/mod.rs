use crate::ResponseExt;
use crate::db::{AppDatabase, Database, User};
use crate::model::UserStatus;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use cookie::{Cookie, SameSite, time::Duration};
use serde::Deserialize;
use std::sync::OnceLock;
use uuid::Uuid;
use worker::*;

pub mod github;
pub mod passkey;
pub mod telegram;
pub use github::{
    handle_github_authorize, handle_github_bind_authorize, handle_github_callback,
    handle_github_unbind,
};
use serde::Serialize;
pub use telegram::{handle_telegram_bind, handle_telegram_login, handle_telegram_unbind};

const SESSION_COOKIE_NAME: &str = "housou_session";
pub const SESSION_DURATION_DAYS: i64 = 30;
const OAUTH_STATE_COOKIE_NAME: &str = "oauth_state";
const OAUTH_ACTION_COOKIE_NAME: &str = "oauth_action";
const OAUTH_STATE_DURATION_MINUTES: i64 = 5;

pub(crate) const EMAIL_IN_USE_ERR: &str = "Email already in use";
pub(crate) const USERNAME_TAKEN_ERR: &str = "Username already taken";

// Helper to get DB
pub fn get_db(env: &Env) -> Result<AppDatabase> {
    let d1 = env.d1("DB")?;
    Ok(AppDatabase::new(d1))
}

// Helper to parse cookies from header string
pub fn parse_cookie_values(header: &str, name: &str) -> Vec<String> {
    Cookie::split_parse(header)
        .filter_map(Result::ok)
        .filter(|c| c.name() == name)
        .map(|c| c.value().to_string())
        .collect()
}

// Helper to get cookie values from request
pub fn get_cookie_values(req: &Request, name: &str) -> Vec<String> {
    if let Ok(Some(header)) = req.headers().get("Cookie") {
        parse_cookie_values(&header, name)
    } else {
        Vec::new()
    }
}

// Helper to get authenticated user
pub async fn get_auth(req: &Request, env: &Env) -> Result<Option<(User, String)>> {
    let db = get_db(env)?;

    for token in get_cookie_values(req, SESSION_COOKIE_NAME) {
        if let Some(user) = db.get_user_by_session_token(&token).await? {
            return Ok(Some((user, token)));
        } else {
            console_log!("Auth failed for token: {}", token);
        }
    }
    Ok(None)
}

fn build_cookie(name: &str, value: &str, days: i64, minutes: i64, secure: bool) -> String {
    let mut duration = Duration::seconds(0);
    if days > 0 {
        duration = Duration::days(days);
    } else if minutes > 0 {
        duration = Duration::minutes(minutes);
    }

    Cookie::build((name, value))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(duration)
        .to_string()
}

pub fn create_session_cookie(token: &str, secure: bool) -> String {
    build_cookie(SESSION_COOKIE_NAME, token, SESSION_DURATION_DAYS, 0, secure)
}

fn clear_session_cookie(secure: bool) -> String {
    build_cookie(SESSION_COOKIE_NAME, "", 0, 0, secure)
}

pub(crate) fn create_oauth_state_cookie(state: &str, secure: bool) -> String {
    build_cookie(
        OAUTH_STATE_COOKIE_NAME,
        state,
        0,
        OAUTH_STATE_DURATION_MINUTES,
        secure,
    )
}

pub(crate) fn clear_oauth_state_cookie(secure: bool) -> String {
    build_cookie(OAUTH_STATE_COOKIE_NAME, "", 0, 0, secure)
}

pub(crate) fn create_oauth_action_cookie(action: &str, secure: bool) -> String {
    build_cookie(
        OAUTH_ACTION_COOKIE_NAME,
        action,
        0,
        OAUTH_STATE_DURATION_MINUTES,
        secure,
    )
}

pub(crate) fn clear_oauth_action_cookie(secure: bool) -> String {
    build_cookie(OAUTH_ACTION_COOKIE_NAME, "", 0, 0, secure)
}

pub(crate) fn get_base_url(env: &Env) -> String {
    env.var("BASE_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "http://localhost:8787".to_string())
}

pub fn is_secure(env: &Env) -> bool {
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
    begin_at: Option<i64>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub github_id: Option<String>,
    pub telegram_id: Option<String>,
    pub created_at: i64,
    pub has_password: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            avatar_url: user.avatar_url,
            github_id: user.github_id,
            telegram_id: user.telegram_id,
            created_at: user.created_at,
            has_password: user.password_hash.is_some(),
        }
    }
}

static ARGON2_INSTANCE: OnceLock<Argon2> = OnceLock::new();

fn get_argon2_instance() -> &'static Argon2<'static> {
    ARGON2_INSTANCE.get_or_init(Argon2::default)
}

pub fn hash_password(password: &str) -> std::result::Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    get_argon2_instance()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| Error::RustError(e.to_string()))
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

pub async fn create_user_session(db: &AppDatabase, user_id: i32, secure: bool) -> Result<String> {
    let token = Uuid::new_v4().to_string();
    let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user_id, &token, expires_at).await?;
    Ok(create_session_cookie(&token, secure))
}

pub async fn handle_register(mut req: Request, env: Env) -> Result<Response> {
    let body: RegisterRequest = req.json().await?;
    let db = get_db(&env)?;

    if (db.get_user_by_email(&body.email).await?).is_some() {
        return Response::error(EMAIL_IN_USE_ERR, 400);
    }
    if (db.get_user_by_username(&body.username).await?).is_some() {
        return Response::error(USERNAME_TAKEN_ERR, 400);
    }

    let password_hash = hash_password(&body.password)?;
    let user = db
        .create_user(
            &body.email,
            &body.username,
            Some(&password_hash),
            None,
            None,
            None,
        )
        .await?;

    let session_cookie = create_user_session(&db, user.id, is_secure(&env)).await?;
    Response::from_json(&UserResponse::from(user))?.add_header("Set-Cookie", &session_cookie)
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

    let session_cookie = create_user_session(&db, user.id, is_secure(&env)).await?;
    Response::from_json(&UserResponse::from(user))?.add_header("Set-Cookie", &session_cookie)
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
        Some((user, _)) => Response::from_json(&UserResponse::from(user)),
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
        return Response::error(USERNAME_TAKEN_ERR, 409);
    }

    // Check unique email if changed and provided
    if let Some(email) = &body.email
        && email != &user.email
        && let Some(existing) = db.get_user_by_email(email).await?
        && existing.id != user.id
    {
        return Response::error(EMAIL_IN_USE_ERR, 409);
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
    Response::from_json(&UserResponse::from(updated_user))
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

    let new_password_hash = hash_password(&body.new_password)?;
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

    db.update_user_item(user.id, &body.title, body.status, body.score, body.begin_at)
        .await?;
    Response::ok("Updated")
}

pub(crate) fn verify_oauth_state(req: &Request, query_state: Option<&str>) -> Result<()> {
    let stored_states = get_cookie_values(req, OAUTH_STATE_COOKIE_NAME);
    let stored_state = stored_states.first().map(|s| s.as_str());

    if query_state
        .zip(stored_state)
        .filter(|(q, s)| q == s)
        .is_none()
    {
        return Err(Error::RustError(
            "Invalid or missing OAuth state".to_string(),
        ));
    }
    Ok(())
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

    #[test]
    fn test_parse_cookie_values() {
        // Single cookie
        let header = "housou_session=abc";
        let values = parse_cookie_values(header, "housou_session");
        assert_eq!(values, vec!["abc"]);

        // Multiple cookies
        let header = "housou_session=abc; oauth_state=xyz; housou_session=def";
        let values = parse_cookie_values(header, "housou_session");
        assert_eq!(values, vec!["abc", "def"]);

        // No matching cookie
        let header = "oauth_state=xyz";
        let values = parse_cookie_values(header, "housou_session");
        assert!(values.is_empty());

        // Empty header
        let header = "";
        let values = parse_cookie_values(header, "housou_session");
        assert!(values.is_empty());

        // Malformed cookie ignored
        let header = "housou_session=abc; invalid_cookie; housou_session=def";
        let values = parse_cookie_values(header, "housou_session");
        assert_eq!(values, vec!["abc", "def"]);
    }
}
