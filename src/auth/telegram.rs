use crate::ResponseExt;
use crate::auth::{
    SESSION_DURATION_DAYS, UserResponse, create_session_cookie, get_auth, get_db, is_secure,
};
use crate::db::{Database, UserUpdate};
use hmac::{Hmac, KeyInit, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;
use worker::*;

#[derive(Deserialize)]
pub struct TelegramAuthData {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: i64,
    pub hash: String,
}

fn build_data_check_string(data: &TelegramAuthData) -> String {
    let id_str = data.id.to_string();
    let auth_date_str = data.auth_date.to_string();

    let mut params = BTreeMap::new();
    params.insert("id", id_str.as_str());
    params.insert("first_name", data.first_name.as_str());
    if let Some(ln) = &data.last_name {
        params.insert("last_name", ln.as_str());
    }
    if let Some(un) = &data.username {
        params.insert("username", un.as_str());
    }
    if let Some(pu) = &data.photo_url {
        params.insert("photo_url", pu.as_str());
    }
    params.insert("auth_date", auth_date_str.as_str());

    let mut data_check_string = String::with_capacity(256);
    for (i, (k, v)) in params.iter().enumerate() {
        if i > 0 {
            data_check_string.push('\n');
        }
        data_check_string.push_str(k);
        data_check_string.push('=');
        data_check_string.push_str(v);
    }
    data_check_string
}

fn verify_telegram_hash_internal(data: &TelegramAuthData, bot_token: &str) -> Result<()> {
    // Construct data-check-string
    let data_check_string = build_data_check_string(data);

    // Compute secret key: SHA256(bot_token)
    let mut hasher = Sha256::new();
    hasher.update(bot_token.as_bytes());
    let secret_key = hasher.finalize();

    // Compute HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&secret_key)
        .map_err(|_| Error::RustError("HMAC initialization failed".to_string()))?;
    mac.update(data_check_string.as_bytes());

    // Verify using constant-time comparison
    let provided_hash_bytes = hex::decode(&data.hash)
        .map_err(|_| Error::RustError("Invalid Telegram hash format".to_string()))?;

    mac.verify_slice(&provided_hash_bytes)
        .map_err(|_| Error::RustError("Invalid Telegram hash".to_string()))?;

    Ok(())
}

fn verify_telegram_auth(data: &TelegramAuthData, bot_token: &str, now: i64) -> Result<()> {
    // Check auth_date (prevent replay attacks, e.g., allow within 24 hours)
    if (now - data.auth_date).abs() > 86400 {
        return Err(Error::RustError("Telegram auth data expired".to_string()));
    }

    verify_telegram_hash_internal(data, bot_token)
}

pub async fn handle_telegram_login(mut req: Request, env: Env) -> Result<Response> {
    let data: TelegramAuthData = req.json().await?;
    let bot_token = env.var("TELEGRAM_BOT_TOKEN")?.to_string();
    let now = crate::utils::now_utc()?.unix_timestamp();

    if let Err(e) = verify_telegram_auth(&data, &bot_token, now) {
        return Response::error(e.to_string(), 401);
    }

    let db = get_db(&env)?;
    let telegram_id_str = data.id.to_string();

    let user = if let Some(u) = db
        .get_user(UserUpdate::telegram_id(Some(telegram_id_str.clone())))
        .await?
    {
        u
    } else {
        // Create new user
        // Generate a placeholder email since Telegram doesn't provide one
        // Pattern: telegram_{id}@telegram.bot (or similar, ensure uniqueness)
        let email = format!("telegram_{}@telegram.bot", data.id);
        let username = data.username.unwrap_or_else(|| data.first_name.clone()); // Fallback to first_name if username is missing

        // Ensure username uniqueness
        // Try original, if taken try random suffix
        let mut final_username = username.clone();
        if (db
            .get_user(UserUpdate::username(final_username.clone()))
            .await?)
            .is_some()
        {
            // Append 4 random hex chars using Uuid
            let suffix = Uuid::new_v4().simple().to_string();
            final_username = format!("{}_{}", username, &suffix[..4]);

            // If still taken (extremely unlikely), append telegram_id
            if (db
                .get_user(UserUpdate::username(final_username.clone()))
                .await?)
                .is_some()
            {
                final_username = format!("{}_{}", username, data.id);
            }
        }

        db.create_user(
            &email,
            &final_username,
            None,
            None,
            Some(&telegram_id_str),
            data.photo_url.as_deref(),
        )
        .await?
    };

    // Create session
    let token = Uuid::new_v4().to_string();
    let expires_at = crate::utils::now_utc_ms() + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    let secure = is_secure(&env)?;
    Response::from_json(&UserResponse::from(user))?
        .add_header("Set-Cookie", &create_session_cookie(&token, secure))
}

pub async fn handle_telegram_bind(mut req: Request, env: Env) -> Result<Response> {
    let (current_user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let data: TelegramAuthData = req.json().await?;
    let bot_token = env.var("TELEGRAM_BOT_TOKEN")?.to_string();
    let now = crate::utils::now_utc()?.unix_timestamp();

    if let Err(e) = verify_telegram_auth(&data, &bot_token, now) {
        return Response::error(e.to_string(), 401);
    }

    let db = get_db(&env)?;
    let telegram_id_str = data.id.to_string();

    // Check if Telegram ID is already used by another user
    if let Some(existing) = db
        .get_user(UserUpdate::telegram_id(Some(telegram_id_str.clone())))
        .await?
        && existing.id != current_user.id
    {
        return Response::error("Telegram account already connected to another user", 409);
    }

    // Update user
    db.update_user(
        current_user.id,
        vec![UserUpdate::telegram_id(Some(telegram_id_str.clone()))],
    )
    .await?;

    // Return updated user profile
    let updated_user = db
        .get_user(UserUpdate::id(current_user.id))
        .await?
        .ok_or_else(|| Error::RustError("User not found".to_string()))?;

    Response::from_json(&UserResponse::from(updated_user))
}

pub async fn handle_telegram_unbind(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    if user.password_hash.is_none() && user.github_id.is_none() {
        return Response::error(
            "Cannot disconnect the only login method. Please set a password or connect GitHub first.",
            400,
        );
    }

    let db = get_db(&env)?;
    db.update_user(user.id, vec![UserUpdate::telegram_id(None)])
        .await?;

    Response::ok("Telegram account disconnected")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;
    use hmac::{Hmac, KeyInit, Mac};
    use sha2::Sha256;

    // Helper to compute hash manually for testing
    fn compute_hash(data: &TelegramAuthData, bot_token: &str) -> String {
        let data_check_string = build_data_check_string(data);

        let mut hasher = Sha256::new();
        hasher.update(bot_token.as_bytes());
        let secret_key = hasher.finalize();

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(&secret_key).unwrap();
        mac.update(data_check_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn test_verify_telegram_auth_success() {
        let now = 1600000000;

        let bot_token = "test_token";
        let mut data = TelegramAuthData {
            id: 12345,
            first_name: "Test".to_string(),
            last_name: Some("User".to_string()),
            username: Some("testuser".to_string()),
            photo_url: None,
            auth_date: now,
            hash: String::new(),
        };

        data.hash = compute_hash(&data, bot_token);

        assert!(verify_telegram_auth(&data, bot_token, now).is_ok());
    }

    #[test]
    fn test_verify_telegram_auth_invalid_hash() {
        let now = 1600000000;

        let bot_token = "test_token";
        let mut data = TelegramAuthData {
            id: 12345,
            first_name: "Test".to_string(),
            last_name: None,
            username: None,
            photo_url: None,
            auth_date: now,
            hash: "invalidhash".to_string(),
        };

        // Test with completely invalid hex
        assert!(verify_telegram_auth(&data, bot_token, now).is_err());

        // Test with valid hex but wrong hash
        data.hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert!(verify_telegram_auth(&data, bot_token, now).is_err());
    }

    #[test]
    fn test_verify_telegram_auth_expired() {
        let now = 1600000000;

        let bot_token = "test_token";
        let mut data = TelegramAuthData {
            id: 12345,
            first_name: "Test".to_string(),
            last_name: None,
            username: None,
            photo_url: None,
            auth_date: now - 86401, // Expired
            hash: String::new(),
        };
        data.hash = compute_hash(&data, bot_token);

        assert!(verify_telegram_auth(&data, bot_token, now).is_err());
    }
}
