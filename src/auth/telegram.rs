use crate::ResponseExt;
use crate::auth::{
    SESSION_DURATION_DAYS, UserResponse, create_session_cookie, get_auth, get_db, is_secure,
};
use crate::db::Database;
use hmac::{Hmac, Mac};
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

fn verify_telegram_auth(data: &TelegramAuthData, bot_token: &str) -> Result<()> {
    // Check auth_date (prevent replay attacks, e.g., allow within 24 hours)
    let now = Date::now().as_millis() as i64 / 1000;
    if (now - data.auth_date).abs() > 86400 {
        return Err(Error::RustError("Telegram auth data expired".to_string()));
    }

    // Construct data-check-string
    let mut params = BTreeMap::new();
    params.insert("id", data.id.to_string());
    params.insert("first_name", data.first_name.clone());
    if let Some(ln) = &data.last_name {
        params.insert("last_name", ln.clone());
    }
    if let Some(un) = &data.username {
        params.insert("username", un.clone());
    }
    if let Some(pu) = &data.photo_url {
        params.insert("photo_url", pu.clone());
    }
    params.insert("auth_date", data.auth_date.to_string());

    let data_check_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    // Compute secret key: SHA256(bot_token)
    let mut hasher = Sha256::new();
    hasher.update(bot_token.as_bytes());
    let secret_key = hasher.finalize();

    // Compute HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&secret_key)
        .map_err(|_| Error::RustError("HMAC initialization failed".to_string()))?;
    mac.update(data_check_string.as_bytes());
    let result = mac.finalize().into_bytes();
    let calculated_hash = hex::encode(result);

    if calculated_hash != data.hash {
        return Err(Error::RustError("Invalid Telegram hash".to_string()));
    }

    Ok(())
}

pub async fn handle_telegram_login(mut req: Request, env: Env) -> Result<Response> {
    let data: TelegramAuthData = req.json().await?;
    let bot_token = env.var("TELEGRAM_BOT_TOKEN")?.to_string();

    verify_telegram_auth(&data, &bot_token)?;

    let db = get_db(&env)?;
    let telegram_id_str = data.id.to_string();

    let user = if let Some(u) = db.get_user_by_telegram_id(&telegram_id_str).await? {
        u
    } else {
        // Create new user
        // Generate a placeholder email since Telegram doesn't provide one
        // Pattern: telegram_{id}@telegram.bot (or similar, ensure uniqueness)
        let email = format!("telegram_{}@telegram.bot", data.id);
        let username = data.username.unwrap_or_else(|| data.first_name.clone()); // Fallback to first_name if username is missing

        // Ensure username uniqueness
        let mut final_username = username.clone();
        let mut counter = 1;
        while (db.get_user_by_username(&final_username).await?).is_some() {
            final_username = format!("{}_{}", username, counter);
            counter += 1;
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
    let expires_at = Date::now().as_millis() as i64 + (SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    let secure = is_secure(&env);
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

    verify_telegram_auth(&data, &bot_token)?;

    let db = get_db(&env)?;
    let telegram_id_str = data.id.to_string();

    // Check if Telegram ID is already used by another user
    if let Some(existing) = db.get_user_by_telegram_id(&telegram_id_str).await?
        && existing.id != current_user.id
    {
        return Response::error("Telegram account already connected to another user", 409);
    }

    // Update user
    db.update_user_telegram_id(current_user.id, Some(&telegram_id_str))
        .await?;

    // Return updated user profile
    let updated_user = db
        .get_user_by_id(current_user.id)
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
    db.update_user_telegram_id(user.id, None).await?;

    Response::ok("Telegram account disconnected")
}
