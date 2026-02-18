use crate::ResponseExt;
use crate::auth;
use crate::db::{Database, User};
use base64::prelude::*;
use coset::cbor::value::Value;
use coset::{CborSerializable, CoseKey, Label};
use p256::EncodedPoint;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use worker::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialUserEntity {
    pub id: String, // Base64url encoded
    pub name: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialRpEntity {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialParameters {
    #[serde(rename = "type")]
    pub type_: String,
    pub alg: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialDescriptor {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String, // Base64url encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transports: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorSelectionCriteria {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticator_attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_resident_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resident_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_verification: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialCreationOptions {
    pub rp: PublicKeyCredentialRpEntity,
    pub user: PublicKeyCredentialUserEntity,
    pub challenge: String, // Base64url encoded
    pub pub_key_cred_params: Vec<PublicKeyCredentialParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_credentials: Option<Vec<PublicKeyCredentialDescriptor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticator_selection: Option<AuthenticatorSelectionCriteria>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialRequestOptions {
    pub challenge: String, // Base64url encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    pub rp_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_credentials: Option<Vec<PublicKeyCredentialDescriptor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_verification: Option<String>,
}

// Client Response Structures

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub response: AuthenticatorAttestationResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extension_results: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAttestationResponse {
    pub client_data_json: String,   // Base64url encoded
    pub attestation_object: String, // Base64url encoded
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub id: String,
    pub raw_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub response: AuthenticatorAssertionResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extension_results: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAssertionResponse {
    pub client_data_json: String,   // Base64url encoded
    pub authenticator_data: String, // Base64url encoded
    pub signature: String,          // Base64url encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_handle: Option<String>, // Base64url encoded
}

// Internal State
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistrationState {
    pub challenge: String,
    pub user_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginState {
    pub challenge: String,
}

// --- Implementation ---

const CHALLENGE_LENGTH: usize = 32;
const STATE_TTL_SECONDS: i64 = 60 * 5; // 5 minutes

fn generate_challenge() -> String {
    let mut buf = [0u8; CHALLENGE_LENGTH];
    OsRng.fill_bytes(&mut buf);
    BASE64_URL_SAFE_NO_PAD.encode(buf)
}

fn get_rp_id(env: &Env) -> String {
    let base_url = env
        .var("BASE_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "http://localhost:8787".to_string());
    // Parse hostname from URL
    match Url::parse(&base_url) {
        Ok(url) => url.host_str().unwrap_or("localhost").to_string(),
        Err(_) => "localhost".to_string(),
    }
}

fn get_rp_name() -> String {
    "Housou".to_string()
}

// Helper structs and functions
#[derive(Deserialize)]
struct ClientDataJson {
    challenge: String,
    origin: String,
    #[serde(rename = "type")]
    type_: String,
}

struct AuthData {
    rp_id_hash: Vec<u8>,
    flags: u8,
    sign_count: u32,
    credential_data: Option<Vec<u8>>,
}

fn verify_client_data(
    client_data_json_b64: &str,
    expected_challenge: &str,
    env: &Env,
    expected_type: &str,
) -> Result<ClientDataJson> {
    let client_data_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(client_data_json_b64)
        .map_err(|e| Error::RustError(format!("Invalid clientDataJSON base64: {}", e)))?;
    let client_data: ClientDataJson = serde_json::from_slice(&client_data_bytes)?;

    if client_data.challenge != expected_challenge {
        return Err(Error::RustError("Challenge mismatch".to_string()));
    }

    let expected_origin = env
        .var("BASE_URL")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "http://localhost:8787".to_string());
    let expected_origin = expected_origin.trim_end_matches('/');

    if client_data.origin != expected_origin {
        return Err(Error::RustError(format!(
            "Origin mismatch: expected {}, got {}",
            expected_origin, client_data.origin
        )));
    }

    if client_data.type_ != expected_type {
        return Err(Error::RustError("Invalid operation type".to_string()));
    }

    Ok(client_data)
}

fn parse_auth_data(auth_data_bytes: &[u8]) -> Result<AuthData> {
    if auth_data_bytes.len() < 37 {
        return Err(Error::RustError("authData too short".to_string()));
    }

    let rp_id_hash = auth_data_bytes[0..32].to_vec();
    let flags = auth_data_bytes[32];
    let sign_count_bytes: [u8; 4] = auth_data_bytes[33..37].try_into().unwrap();
    let sign_count = u32::from_be_bytes(sign_count_bytes);

    let credential_data = if (flags & 0x40) != 0 {
        Some(auth_data_bytes[37..].to_vec())
    } else {
        None
    };

    Ok(AuthData {
        rp_id_hash,
        flags,
        sign_count,
        credential_data,
    })
}

fn verify_rp_id_hash(rp_id_hash: &[u8], env: &Env) -> Result<()> {
    let expected_rp_id = get_rp_id(env);
    let mut hasher = Sha256::new();
    hasher.update(expected_rp_id.as_bytes());
    let expected_hash = hasher.finalize();
    if rp_id_hash != expected_hash.as_slice() {
        return Err(Error::RustError("RP ID Hash mismatch".to_string()));
    }
    Ok(())
}

// Start Registration
pub async fn start_registration<D: Database>(
    db: &D,
    user: &User,
    env: &Env,
) -> Result<PublicKeyCredentialCreationOptions> {
    let challenge = generate_challenge();
    let user_handle = BASE64_URL_SAFE_NO_PAD.encode(user.id.to_string().as_bytes()); // Simple user handle

    // Check existing passkeys to exclude them
    let existing_passkeys = db.list_passkeys(user.id).await?;
    let exclude_credentials = if existing_passkeys.is_empty() {
        None
    } else {
        Some(
            existing_passkeys
                .into_iter()
                .map(|pk| PublicKeyCredentialDescriptor {
                    type_: "public-key".to_string(),
                    id: pk.cred_id,
                    transports: None,
                })
                .collect(),
        )
    };

    let options = PublicKeyCredentialCreationOptions {
        rp: PublicKeyCredentialRpEntity {
            name: get_rp_name(),
            id: get_rp_id(env),
        },
        user: PublicKeyCredentialUserEntity {
            id: user_handle,
            name: user.email.clone(),
            display_name: user.username.clone(),
        },
        challenge: challenge.clone(),
        pub_key_cred_params: vec![
            PublicKeyCredentialParameters {
                type_: "public-key".to_string(),
                alg: -7, // ES256
            },
            PublicKeyCredentialParameters {
                type_: "public-key".to_string(),
                alg: -257, // RS256
            },
        ],
        timeout: Some(60000),
        exclude_credentials,
        authenticator_selection: Some(AuthenticatorSelectionCriteria {
            authenticator_attachment: None, // Cross-platform or platform
            require_resident_key: Some(false),
            resident_key: Some("preferred".to_string()),
            user_verification: Some("preferred".to_string()),
        }),
        attestation: Some("none".to_string()), // We don't verify attestation trust path in this simple impl
    };

    // Save state
    let state = RegistrationState {
        challenge,
        user_id: user.id,
    };
    let state_json = serde_json::to_string(&state)?;
    let expires_at = Date::now().as_millis() as i64 + (STATE_TTL_SECONDS * 1000);
    // Use user ID as key for simplicity in this flow, or a separate ID if needed.
    // Ideally the frontend sends back a session ID, but here we can just key by user ID for "current registration attempt".
    // Better: frontend doesn't send ID back in standard WebAuthn flow until finish.
    // So we need to store it associated with the user via cookie or just rely on the fact that the user is logged in.
    // We will use a unique ID and set it in a cookie, or return it?
    // Actually, common pattern: return options, client sends back response.
    // Server needs to match response to challenge.
    // We can key state by challenge? Or by user_id if we only allow one pending registration per user.
    // Let's key by user_id for registration since user must be logged in.
    let state_id = format!("reg:{}", user.id);
    db.save_passkey_state(&state_id, &state_json, expires_at)
        .await?;

    Ok(options)
}

pub async fn finish_registration<D: Database>(
    db: &D,
    user: &User,
    env: &Env,
    response: RegistrationResponse,
) -> Result<()> {
    // 1. Retrieve state
    let state_id = format!("reg:{}", user.id);
    let state_record = db
        .get_passkey_state(&state_id)
        .await?
        .ok_or_else(|| Error::RustError("Registration session expired or invalid".to_string()))?;
    let state: RegistrationState = serde_json::from_str(&state_record.state_json)?;

    // 2. Parse & Verify ClientDataJSON
    let _client_data = verify_client_data(
        &response.response.client_data_json,
        &state.challenge,
        env,
        "webauthn.create",
    )?;

    // 3. Parse AttestationObject (CBOR)
    let att_obj_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.attestation_object)
        .map_err(|e| Error::RustError(format!("Invalid attestationObject base64: {}", e)))?;

    let att_obj: Value = ciborium::from_reader(att_obj_bytes.as_slice())
        .map_err(|e| Error::RustError(format!("Invalid attestationObject CBOR: {}", e)))?;

    // Extract authData
    let auth_data_bytes = match &att_obj {
        Value::Map(m) => m
            .iter()
            .find(|(k, _)| k.as_text().map(|s| s == "authData").unwrap_or(false))
            .map(|(_, v)| {
                v.as_bytes()
                    .ok_or(Error::RustError("authData not bytes".into()))
            })
            .unwrap_or(Err(Error::RustError("authData missing".into())))?,
        _ => {
            return Err(Error::RustError(
                "Invalid attestation object structure".into(),
            ));
        }
    };

    // 4. Parse & Verify AuthData
    let auth_data = parse_auth_data(auth_data_bytes)?;
    verify_rp_id_hash(&auth_data.rp_id_hash, env)?;

    // Verify User Present flag (bit 0)
    if (auth_data.flags & 0x01) == 0 {
        return Err(Error::RustError("User Present flag not set".to_string()));
    }

    // Extract AttestedCredentialData
    let credential_data_bytes = auth_data
        .credential_data
        .ok_or_else(|| Error::RustError("Attested Credential Data missing".to_string()))?;

    if credential_data_bytes.len() < 18 {
        return Err(Error::RustError("Credential Data too short".to_string()));
    }

    let cred_id_len_bytes: [u8; 2] = credential_data_bytes[16..18].try_into().unwrap();
    let cred_id_len = u16::from_be_bytes(cred_id_len_bytes) as usize;

    if credential_data_bytes.len() < 18 + cred_id_len {
        return Err(Error::RustError("Credential ID incomplete".to_string()));
    }

    let credential_id = &credential_data_bytes[18..18 + cred_id_len];
    let public_key_cbor = &credential_data_bytes[18 + cred_id_len..];

    // Validate Public Key (Basic check that it parses)
    let _cose_key: CoseKey = CoseKey::from_slice(public_key_cbor)
        .map_err(|e| Error::RustError(format!("Invalid Public Key CBOR: {}", e)))?;

    // Store in DB
    // We store the public key as Bytes or serialized CBOR (Base64 encoded)
    let cred_id_str = BASE64_URL_SAFE_NO_PAD.encode(credential_id);
    let pub_key_str = BASE64_URL_SAFE_NO_PAD.encode(public_key_cbor);

    // Use a default name, user can rename later
    let name = "Passkey".to_string();

    db.create_passkey(
        user.id,
        &cred_id_str,
        &pub_key_str,
        &name,
        auth_data.sign_count as i64,
    )
    .await?;

    // Cleanup state
    db.delete_passkey_state(&state_id).await?;

    Ok(())
}

// Start Login
pub async fn start_login<D: Database>(
    db: &D,
    env: &Env,
) -> Result<(PublicKeyCredentialRequestOptions, String)> {
    let challenge = generate_challenge();
    let challenge_b64 = challenge.clone();

    let options = PublicKeyCredentialRequestOptions {
        challenge: challenge_b64,
        timeout: Some(60000),
        rp_id: get_rp_id(env),
        allow_credentials: None, // Allow any credential for this RP (discoverable) or we could list user's creds if user identifier is known
        user_verification: Some("preferred".to_string()),
    };

    // Save state
    let state = LoginState { challenge };
    let state_json = serde_json::to_string(&state)?;
    let expires_at = Date::now().as_millis() as i64 + (STATE_TTL_SECONDS * 1000);

    // We use the challenge itself as the state ID since we don't have a user ID yet for discoverable flow
    // But wait, the frontend might send an email first?
    // If we want to support passwordless/usernameless flow, we key by challenge.
    // However, to keep it simple and consistent with typical "login with passkey" button that might be clicked *after* entering email or just "login with passkey",
    // let's use the challenge as the key to retrieve state later.
    // The finish endpoint will receive the response which contains the challenge in clientDataJSON.
    let state_id = format!("login:{}", options.challenge); // Use the base64 challenge as ID
    db.save_passkey_state(&state_id, &state_json, expires_at)
        .await?;

    Ok((options, state_id))
}

// Finish Login
pub async fn finish_login<D: Database>(db: &D, env: &Env, response: LoginResponse) -> Result<User> {
    // 1. Retrieve state using challenge from response (insecure to trust client data blindly? No, we verify signature later)
    // But we need the challenge to look up the state.
    // We can parse clientDataJSON first to get the challenge.
    let client_data_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.client_data_json)
        .map_err(|e| Error::RustError(format!("Invalid clientDataJSON base64: {}", e)))?;
    let client_data: ClientDataJson = serde_json::from_slice(&client_data_bytes)?;

    let state_id = format!("login:{}", client_data.challenge);
    let state_record = db.get_passkey_state(&state_id).await?.ok_or_else(|| {
        Error::RustError("Login session expired or invalid (challenge not found)".to_string())
    })?;
    let state: LoginState = serde_json::from_str(&state_record.state_json)?;

    // 2. Parse & Verify ClientDataJSON
    let _client_data = verify_client_data(
        &response.response.client_data_json,
        &state.challenge,
        env,
        "webauthn.get",
    )?;

    // 3. Parse & Verify AuthenticatorData
    let auth_data_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.authenticator_data)
        .map_err(|e| Error::RustError(format!("Invalid authenticatorData base64: {}", e)))?;

    let auth_data = parse_auth_data(&auth_data_bytes)?;
    verify_rp_id_hash(&auth_data.rp_id_hash, env)?;

    // Verify User Present flag
    if (auth_data.flags & 0x01) == 0 {
        return Err(Error::RustError("User Present flag not set".to_string()));
    }

    // 4. Verify Signature
    // Retrieve Passkey from DB
    let passkey = db
        .get_passkey(&response.id)
        .await?
        .ok_or_else(|| Error::RustError("Passkey not found".to_string()))?;

    // Verify User Handle if present
    if let Some(user_handle_b64) = &response.response.user_handle {
        let user_handle_bytes = BASE64_URL_SAFE_NO_PAD
            .decode(user_handle_b64)
            .map_err(|e| Error::RustError(format!("Invalid userHandle base64: {}", e)))?;
        let user_id_str = String::from_utf8(user_handle_bytes)
            .map_err(|_| Error::RustError("Invalid userHandle utf8".to_string()))?;

        if user_id_str != passkey.user_id.to_string() {
            return Err(Error::RustError("User Handle mismatch".to_string()));
        }
    }

    // Parse Public Key (COSE)
    let pub_key_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&passkey.passkey_json)
        .map_err(|e| Error::RustError(format!("Invalid stored public key base64: {}", e)))?;

    let cose_key = CoseKey::from_slice(&pub_key_bytes)
        .map_err(|e| Error::RustError(format!("Invalid stored public key CBOR: {}", e)))?;

    // Extract key parameters (assuming EC2/P-256 for now as we requested it)
    // We should check kty (1 for OKP/EC2) and crv (1 for P-256)
    // CoseKey structure is generic.
    // kty: 1 (EC2), crv: 1 (P-256), x, y

    let x = match cose_key.params.iter().find(|(k, _)| k == &Label::Int(-2)) {
        Some((_, Value::Bytes(b))) => b,
        _ => return Err(Error::RustError("Missing x coordinate".to_string())),
    };
    let y = match cose_key.params.iter().find(|(k, _)| k == &Label::Int(-3)) {
        Some((_, Value::Bytes(b))) => b,
        _ => return Err(Error::RustError("Missing y coordinate".to_string())),
    };

    if x.len() != 32 || y.len() != 32 {
        return Err(Error::RustError("Invalid coordinate length".to_string()));
    }

    // Construct VerifyingKey
    let x_arr = p256::FieldBytes::from_slice(x);
    let y_arr = p256::FieldBytes::from_slice(y);
    let encoded_point = EncodedPoint::from_affine_coordinates(x_arr, y_arr, false);
    let verifying_key = VerifyingKey::from_encoded_point(&encoded_point)
        .map_err(|e| Error::RustError(format!("Invalid P-256 key: {}", e)))?;

    // Construct Signed Data: authData || hash(clientDataJSON)
    let mut hasher = Sha256::new();
    hasher.update(&client_data_bytes);
    let client_data_hash = hasher.finalize();

    let mut signed_data = Vec::new();
    signed_data.extend_from_slice(&auth_data_bytes);
    signed_data.extend_from_slice(&client_data_hash);

    // Parse Signature (ASN.1 DER)
    let signature_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.signature)
        .map_err(|e| Error::RustError(format!("Invalid signature base64: {}", e)))?;

    let signature = Signature::from_der(&signature_bytes)
        .map_err(|e| Error::RustError(format!("Invalid signature DER: {}", e)))?;

    // Verify
    verifying_key
        .verify(&signed_data, &signature)
        .map_err(|e| {
            console_error!("Signature verification failed: {}", e);
            Error::RustError("Signature verification failed".to_string())
        })?;

    // 7. Counter Check (Clone Protection)
    if auth_data.sign_count <= passkey.counter as u32 && auth_data.sign_count != 0 {
        // Note: Some authenticators return 0. If it was non-zero before and now 0 or less, it's suspicious.
        // But for simplicity, we just enforce strictly increasing if stored is > 0
        if passkey.counter > 0 {
            console_error!(
                "Signature counter regression! Stored: {}, Received: {}",
                passkey.counter,
                auth_data.sign_count
            );
            return Err(Error::RustError("Signature counter regression".to_string()));
        }
    }

    // Update counter and last used
    let now = Date::now().as_millis() as i64;
    db.update_passkey_counter(&passkey.cred_id, auth_data.sign_count as i64, now)
        .await?;

    // Cleanup state
    db.delete_passkey_state(&state_id).await?;

    // Return User
    let user = db
        .get_user_by_id(passkey.user_id)
        .await?
        .ok_or_else(|| Error::RustError("User not found".to_string()))?;

    Ok(user)
}

// List Passkeys
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeySummary {
    pub id: String, // cred_id
    pub name: String,
    pub created_at: i64,
    pub last_used_at: i64,
}

pub async fn list_user_passkeys<D: Database>(db: &D, user_id: i32) -> Result<Vec<PasskeySummary>> {
    let passkeys = db.list_passkeys(user_id).await?;
    let summaries = passkeys
        .into_iter()
        .map(|pk| PasskeySummary {
            id: pk.cred_id,
            name: pk.name,
            created_at: pk.created_at,
            last_used_at: pk.last_used_at,
        })
        .collect();
    Ok(summaries)
}

// Delete Passkey
pub async fn delete_user_passkey<D: Database>(db: &D, user_id: i32, cred_id: &str) -> Result<()> {
    db.delete_passkey(user_id, cred_id).await
}

// Handlers

pub async fn handle_register_start(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let db = auth::get_db(&env)?;
    let options = start_registration(&db, &user, &env).await?;
    Response::from_json(&options)
}

pub async fn handle_register_finish(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let response: RegistrationResponse = req.json().await?;
    let db = auth::get_db(&env)?;
    finish_registration(&db, &user, &env, response).await?;
    Response::ok("Passkey registered")
}

pub async fn handle_login_start(_req: Request, env: Env) -> Result<Response> {
    // Check if user is already logged in? Optional.
    let db = auth::get_db(&env)?;
    let (options, _) = start_login(&db, &env).await?;
    Response::from_json(&options)
}

pub async fn handle_login_finish(mut req: Request, env: Env) -> Result<Response> {
    let response: LoginResponse = req.json().await?;
    let db = auth::get_db(&env)?;
    let user = finish_login(&db, &env, response).await?;

    // Create session
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at =
        Date::now().as_millis() as i64 + (auth::SESSION_DURATION_DAYS * 24 * 60 * 60 * 1000);
    db.create_session(user.id, &token, expires_at).await?;

    let secure = auth::is_secure(&env);
    Response::from_json(&user)?
        .add_header("Set-Cookie", &auth::create_session_cookie(&token, secure))
}

pub async fn handle_list(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let db = auth::get_db(&env)?;
    let list = list_user_passkeys(&db, user.id).await?;
    Response::from_json(&list)
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
            delete_user_passkey(&db, user.id, &cred_id).await?;
            Response::ok("Deleted")
        }
        None => Response::error("Missing id", 400),
    }
}
