use crate::ResponseExt;
use crate::auth;
use crate::db::{Database, UserUpdate};
use passkey_server::types::*;
use serde::Deserialize;
use worker::*;

// Configuration helper
struct ConfigHelper;

impl ConfigHelper {
    fn from_req(req: &Request, env: &Env) -> PasskeyConfig {
        let rp_id = req
            .url()
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "localhost".to_string());

        let origin = req
            .headers()
            .get("Origin")
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                env.var("BASE_URL")
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "http://localhost:8787".to_string())
                    .trim_end_matches('/')
                    .to_string()
            });

        PasskeyConfig {
            rp_id,
            rp_name: "Housou".to_string(),
            origin,
            state_ttl: 300,
        }
    }
}

// HTTP Handlers

pub async fn handle_register_start(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let db = auth::get_db(&env)?;
    let config = ConfigHelper::from_req(&req, &env);
    let now = crate::utils::now_utc_ms();

    let options = passkey_server::start_registration(
        &db,
        &user.id.to_string(),
        &user.username,
        &user.username, // display_name same as username for now
        &config,
        now,
    )
    .await
    .map_err(|e| Error::RustError(e.to_string()))?;

    Response::from_json(&options)
}

pub async fn handle_register_finish(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let config = ConfigHelper::from_req(&req, &env);
    let body: RegistrationResponse = req.json().await?;
    let db = auth::get_db(&env)?;
    let now = crate::utils::now_utc_ms();

    passkey_server::finish_registration(&db, &user.id.to_string(), &config, body, now)
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    Response::ok("Passkey registered")
}

pub async fn handle_login_start(req: Request, env: Env) -> Result<Response> {
    let db = auth::get_db(&env)?;
    let config = ConfigHelper::from_req(&req, &env);
    let now = crate::utils::now_utc_ms();

    let options = passkey_server::start_login(&db, &config, now)
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    Response::from_json(&options)
}

pub async fn handle_login_finish(mut req: Request, env: Env) -> Result<Response> {
    let config = ConfigHelper::from_req(&req, &env);
    let response: LoginResponse = req.json().await?;
    let db = auth::get_db(&env)?;
    let now = crate::utils::now_utc_ms();

    let user_id_str = passkey_server::finish_login(&db, &config, response, now)
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    let user_id = user_id_str
        .parse::<i32>()
        .map_err(|_| Error::RustError("Invalid user ID format".into()))?;

    // Fetch user to create session
    let user = db
        .get_user(UserUpdate::id(user_id))
        .await?
        .ok_or_else(|| Error::RustError("User not found".into()))?;

    let session_cookie = auth::create_user_session(&db, user.id, auth::is_secure(&env)).await?;
    Response::from_json(&auth::UserResponse::from(user))?.add_header("Set-Cookie", &session_cookie)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PasskeySummary {
    id: String,
    name: String,
    created_at: i64,
    last_used_at: i64,
}

pub async fn handle_list(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let db = auth::get_db(&env)?;

    // PasskeyStore trait defines list_passkeys returning Vec<StoredPasskey>
    // We want to return PasskeySummary
    use passkey_server::PasskeyStore;

    let passkeys = db
        .list_passkeys(user.id.to_string())
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    let summary: Vec<PasskeySummary> = passkeys
        .into_iter()
        .map(|pk| PasskeySummary {
            id: pk.cred_id,
            name: pk.name,
            created_at: pk.created_at,
            last_used_at: pk.last_used_at,
        })
        .collect();

    Response::from_json(&summary)
}

pub async fn handle_delete(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let url = req.url()?;
    let id = url
        .query_pairs()
        .find(|(k, _)| k == "id")
        .map(|(_, v)| v.to_string());

    match id {
        Some(cred_id) => {
            let db = auth::get_db(&env)?;
            use passkey_server::PasskeyStore;
            db.delete_passkey(user.id.to_string(), &cred_id)
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;
            Response::ok("Deleted")
        }
        None => Response::error("Missing id", 400),
    }
}

pub async fn handle_rename(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    #[derive(Deserialize)]
    struct RenameRequest {
        id: String,
        name: String,
    }

    let body: RenameRequest = req.json().await?;
    let db = auth::get_db(&env)?;
    use passkey_server::PasskeyStore;

    // Verify ownership and existence
    // PasskeyStore trait: get_passkey returns Result<Option<StoredPasskey>>
    let passkey = db
        .get_passkey(&body.id)
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    match passkey {
        Some(pk) if pk.user_id == user.id.to_string() => {
            db.update_passkey_name(&body.id, &body.name)
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;
            Response::ok("Renamed")
        }
        Some(_) => Response::error("Unauthorized", 401),
        None => Response::error("Passkey not found", 404),
    }
}
