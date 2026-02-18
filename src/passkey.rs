use crate::ResponseExt;
use crate::auth;
use crate::db::{Database, User};
use async_trait::async_trait;
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

// Data Models

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredPasskey {
    pub user_id: i32,
    pub cred_id: String,
    pub public_key: String, // Base64url-encoded COSE key
    pub name: String,
    pub created_at: i64,
    pub last_used_at: i64,
    pub counter: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasskeyState {
    pub id: String,
    pub state_json: String,
    pub expires_at: i64,
}

// Storage Trait

#[async_trait(?Send)]
pub trait PasskeyStore {
    // Credential CRUD
    async fn create_passkey(
        &self,
        user_id: i32,
        cred_id: &str,
        public_key: &str,
        name: &str,
        counter: i64,
    ) -> Result<()>;
    async fn get_passkey(&self, cred_id: &str) -> Result<Option<StoredPasskey>>;
    async fn list_passkeys(&self, user_id: i32) -> Result<Vec<StoredPasskey>>;
    async fn delete_passkey(&self, user_id: i32, cred_id: &str) -> Result<()>;
    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> Result<()>;
    async fn update_passkey_name(&self, cred_id: &str, new_name: &str) -> Result<()>;

    // Ephemeral state (challenge ↔ session)
    async fn save_state(&self, id: &str, state_json: &str, expires_at: i64) -> Result<()>;
    async fn get_state(&self, id: &str) -> Result<Option<PasskeyState>>;
    async fn delete_state(&self, id: &str) -> Result<()>;
}

// User Lookup Trait

#[async_trait(?Send)]
pub trait UserLookup {
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>>;
}

// Configuration

pub struct PasskeyConfig {
    pub rp_id: String,
    pub rp_name: String,
    pub origin: String,
}

impl PasskeyConfig {
    pub fn from_env(env: &Env) -> Self {
        let base_url = env
            .var("BASE_URL")
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "http://localhost:8787".to_string());

        let rp_id = match Url::parse(&base_url) {
            Ok(url) => url.host_str().unwrap_or("localhost").to_string(),
            Err(_) => "localhost".to_string(),
        };

        let origin = base_url.trim_end_matches('/').to_string();

        Self {
            rp_id,
            rp_name: "Housou".to_string(),
            origin,
        }
    }
}

// WebAuthn Protocol Types

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialCreationOptions {
    pub rp: RpEntity,
    pub user: UserEntity,
    pub challenge: String,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_credentials: Option<Vec<CredentialDescriptor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticator_selection: Option<AuthenticatorSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyCredentialRequestOptions {
    pub challenge: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    pub rp_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_credentials: Option<Vec<CredentialDescriptor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_verification: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpEntity {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserEntity {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PubKeyCredParam {
    #[serde(rename = "type")]
    pub type_: String,
    pub alg: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDescriptor {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transports: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorSelection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticator_attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_resident_key: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resident_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_verification: Option<String>,
}

// Client response types

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub response: AttestationResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extension_results: Option<serde_json::Value>,
    // Extension: Allow defining a name for the credential
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttestationResponse {
    #[serde(rename = "clientDataJSON")]
    pub client_data_json: String,
    pub attestation_object: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub id: String,
    pub raw_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub response: AssertionResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_extension_results: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssertionResponse {
    #[serde(rename = "clientDataJSON")]
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_handle: Option<String>,
}

// Public summary (for API responses)

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasskeySummary {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub last_used_at: i64,
}

// Internal helpers

const CHALLENGE_LEN: usize = 32;
const STATE_TTL_SECONDS: i64 = 60 * 5;

#[derive(Serialize, Deserialize)]
struct RegState {
    challenge: String,
    user_id: i32,
}

#[derive(Serialize, Deserialize)]
struct LoginState {
    challenge: String,
}

#[derive(Deserialize)]
struct ClientData {
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

fn generate_challenge() -> String {
    let mut buf = [0u8; CHALLENGE_LEN];
    OsRng.fill_bytes(&mut buf);
    BASE64_URL_SAFE_NO_PAD.encode(buf)
}

fn verify_client_data(
    client_data_b64: &str,
    expected_challenge: &str,
    config: &PasskeyConfig,
    expected_type: &str,
) -> Result<(ClientData, Vec<u8>)> {
    let bytes = BASE64_URL_SAFE_NO_PAD
        .decode(client_data_b64)
        .map_err(|e| Error::RustError(format!("Invalid clientDataJSON base64: {e}")))?;
    let data: ClientData = serde_json::from_slice(&bytes)?;

    if data.challenge != expected_challenge {
        return Err(Error::RustError("Challenge mismatch".into()));
    }
    if data.origin != config.origin {
        return Err(Error::RustError(format!(
            "Origin mismatch: expected {}, got {}",
            config.origin, data.origin
        )));
    }
    if data.type_ != expected_type {
        return Err(Error::RustError("Invalid operation type".into()));
    }
    Ok((data, bytes))
}

fn parse_auth_data(raw: &[u8]) -> Result<AuthData> {
    if raw.len() < 37 {
        return Err(Error::RustError("authData too short".into()));
    }
    let rp_id_hash = raw[0..32].to_vec();
    let flags = raw[32];
    let sign_count = u32::from_be_bytes(raw[33..37].try_into().unwrap());
    let credential_data = if (flags & 0x40) != 0 {
        Some(raw[37..].to_vec())
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

fn verify_rp_id_hash(hash: &[u8], config: &PasskeyConfig) -> Result<()> {
    let expected = Sha256::digest(config.rp_id.as_bytes());
    if hash != expected.as_slice() {
        return Err(Error::RustError("RP ID Hash mismatch".into()));
    }
    Ok(())
}

fn verify_user_present(flags: u8) -> Result<()> {
    if (flags & 0x01) == 0 {
        return Err(Error::RustError("User Present flag not set".into()));
    }
    Ok(())
}

fn extract_credential(data: &[u8]) -> Result<(&[u8], &[u8])> {
    if data.len() < 18 {
        return Err(Error::RustError("Credential Data too short".into()));
    }
    let cred_id_len = u16::from_be_bytes(data[16..18].try_into().unwrap()) as usize;
    if data.len() < 18 + cred_id_len {
        return Err(Error::RustError("Credential ID incomplete".into()));
    }
    let cred_id = &data[18..18 + cred_id_len];
    let pub_key_cbor = &data[18 + cred_id_len..];
    Ok((cred_id, pub_key_cbor))
}

fn verify_p256_signature(
    pub_key_cbor: &[u8],
    signed_data: &[u8],
    signature_der: &[u8],
) -> Result<()> {
    let cose_key = CoseKey::from_slice(pub_key_cbor)
        .map_err(|e| Error::RustError(format!("Invalid COSE key: {e}")))?;

    let x = match cose_key.params.iter().find(|(k, _)| k == &Label::Int(-2)) {
        Some((_, Value::Bytes(b))) => b,
        _ => return Err(Error::RustError("Missing x coordinate".into())),
    };
    let y = match cose_key.params.iter().find(|(k, _)| k == &Label::Int(-3)) {
        Some((_, Value::Bytes(b))) => b,
        _ => return Err(Error::RustError("Missing y coordinate".into())),
    };

    if x.len() != 32 || y.len() != 32 {
        return Err(Error::RustError("Invalid coordinate length".into()));
    }

    let encoded_point = EncodedPoint::from_affine_coordinates(
        p256::FieldBytes::from_slice(x),
        p256::FieldBytes::from_slice(y),
        false,
    );
    let verifying_key = VerifyingKey::from_encoded_point(&encoded_point)
        .map_err(|e| Error::RustError(format!("Invalid P-256 key: {e}")))?;

    let signature = Signature::from_der(signature_der)
        .map_err(|e| Error::RustError(format!("Invalid signature DER: {e}")))?;

    verifying_key.verify(signed_data, &signature).map_err(|e| {
        console_error!("Signature verification failed: {e}");
        Error::RustError("Signature verification failed".into())
    })
}

// Core WebAuthn Flows

pub async fn start_registration<S: PasskeyStore>(
    store: &S,
    user: &User,
    config: &PasskeyConfig,
) -> Result<PublicKeyCredentialCreationOptions> {
    let challenge = generate_challenge();
    let user_handle = BASE64_URL_SAFE_NO_PAD.encode(user.id.to_string().as_bytes());

    let existing = store.list_passkeys(user.id).await?;
    let exclude_credentials = if existing.is_empty() {
        None
    } else {
        Some(
            existing
                .into_iter()
                .map(|pk| CredentialDescriptor {
                    type_: "public-key".into(),
                    id: pk.cred_id,
                    transports: None,
                })
                .collect(),
        )
    };

    let options = PublicKeyCredentialCreationOptions {
        rp: RpEntity {
            name: config.rp_name.clone(),
            id: config.rp_id.clone(),
        },
        user: UserEntity {
            id: user_handle,
            name: user.email.clone(),
            display_name: user.username.clone(),
        },
        challenge: challenge.clone(),
        pub_key_cred_params: vec![PubKeyCredParam {
            type_: "public-key".into(),
            alg: -7, // ES256
        }],
        timeout: Some(60000),
        exclude_credentials,
        authenticator_selection: Some(AuthenticatorSelection {
            authenticator_attachment: None,
            require_resident_key: Some(false),
            resident_key: Some("preferred".into()),
            user_verification: Some("preferred".into()),
        }),
        attestation: Some("none".into()),
    };

    // Persist challenge state
    let state = RegState {
        challenge,
        user_id: user.id,
    };
    let state_id = format!("reg:{}", user.id);
    let expires_at = Date::now().as_millis() as i64 + (STATE_TTL_SECONDS * 1000);
    store
        .save_state(&state_id, &serde_json::to_string(&state)?, expires_at)
        .await?;

    Ok(options)
}

pub async fn finish_registration<S: PasskeyStore>(
    store: &S,
    user: &User,
    config: &PasskeyConfig,
    response: RegistrationResponse,
) -> Result<()> {
    // 1. Retrieve & validate state
    let state_id = format!("reg:{}", user.id);
    let record = store
        .get_state(&state_id)
        .await?
        .ok_or_else(|| Error::RustError("Registration session expired or invalid".into()))?;
    let state: RegState = serde_json::from_str(&record.state_json)?;

    // 2. Verify clientDataJSON
    verify_client_data(
        &response.response.client_data_json,
        &state.challenge,
        config,
        "webauthn.create",
    )?;

    // 3. Parse attestation object (CBOR)
    let att_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.attestation_object)
        .map_err(|e| Error::RustError(format!("Invalid attestationObject base64: {e}")))?;

    let att_obj: Value = ciborium::from_reader(att_bytes.as_slice())
        .map_err(|e| Error::RustError(format!("Invalid attestationObject CBOR: {e}")))?;

    let auth_data_bytes = match &att_obj {
        Value::Map(m) => m
            .iter()
            .find(|(k, _)| k.as_text().is_some_and(|s| s == "authData"))
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

    // 4. Verify authData
    let auth_data = parse_auth_data(auth_data_bytes)?;
    verify_rp_id_hash(&auth_data.rp_id_hash, config)?;
    verify_user_present(auth_data.flags)?;

    // 5. Extract credential
    let cred_bytes = auth_data
        .credential_data
        .ok_or_else(|| Error::RustError("Attested Credential Data missing".into()))?;
    let (cred_id, pub_key_cbor) = extract_credential(&cred_bytes)?;

    // Extract AAGUID from credential data (first 16 bytes)
    let aaguid = if cred_bytes.len() >= 16 {
        let aaguid_bytes: [u8; 16] = cred_bytes[0..16].try_into().unwrap();
        let uuid = uuid::Uuid::from_bytes(aaguid_bytes);
        if uuid.is_nil() {
            None
        } else {
            Some(uuid.to_string())
        }
    } else {
        None
    };

    // Validate the public key parses
    CoseKey::from_slice(pub_key_cbor)
        .map_err(|e| Error::RustError(format!("Invalid Public Key CBOR: {e}")))?;

    // 6. Store credential
    let cred_id_b64 = BASE64_URL_SAFE_NO_PAD.encode(cred_id);
    let pub_key_b64 = BASE64_URL_SAFE_NO_PAD.encode(pub_key_cbor);

    let passkey_name = match (response.name.as_deref(), aaguid) {
        (Some(name), Some(id)) => format!("{}-{}", name, id),
        (Some(name), None) => name.to_string(),
        (None, Some(id)) => format!("Passkey-{}", id),
        (None, None) => "Passkey".to_string(),
    };

    store
        .create_passkey(
            user.id,
            &cred_id_b64,
            &pub_key_b64,
            &passkey_name,
            auth_data.sign_count as i64,
        )
        .await?;

    store.delete_state(&state_id).await?;
    Ok(())
}

pub async fn start_login<S: PasskeyStore>(
    store: &S,
    config: &PasskeyConfig,
) -> Result<PublicKeyCredentialRequestOptions> {
    let challenge = generate_challenge();

    let options = PublicKeyCredentialRequestOptions {
        challenge: challenge.clone(),
        timeout: Some(60000),
        rp_id: config.rp_id.clone(),
        allow_credentials: None,
        user_verification: Some("preferred".into()),
    };

    let state = LoginState { challenge };
    let state_id = format!("login:{}", options.challenge);
    let expires_at = Date::now().as_millis() as i64 + (STATE_TTL_SECONDS * 1000);
    store
        .save_state(&state_id, &serde_json::to_string(&state)?, expires_at)
        .await?;

    Ok(options)
}

pub async fn finish_login<S: PasskeyStore + UserLookup>(
    store: &S,
    config: &PasskeyConfig,
    response: LoginResponse,
) -> Result<User> {
    // 1. Parse clientDataJSON to retrieve the challenge for state lookup
    let client_data_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.client_data_json)
        .map_err(|e| Error::RustError(format!("Invalid clientDataJSON base64: {e}")))?;
    let client_data_peek: ClientData = serde_json::from_slice(&client_data_bytes)?;

    let state_id = format!("login:{}", client_data_peek.challenge);
    let record = store
        .get_state(&state_id)
        .await?
        .ok_or_else(|| Error::RustError("Login session expired or invalid".into()))?;
    let state: LoginState = serde_json::from_str(&record.state_json)?;

    // 2. Full clientDataJSON verification
    verify_client_data(
        &response.response.client_data_json,
        &state.challenge,
        config,
        "webauthn.get",
    )?;

    // 3. Parse & verify authenticator data
    let auth_data_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.authenticator_data)
        .map_err(|e| Error::RustError(format!("Invalid authenticatorData base64: {e}")))?;

    let auth_data = parse_auth_data(&auth_data_bytes)?;
    verify_rp_id_hash(&auth_data.rp_id_hash, config)?;
    verify_user_present(auth_data.flags)?;

    // 4. Look up stored credential
    let passkey = store
        .get_passkey(&response.id)
        .await?
        .ok_or_else(|| Error::RustError("Passkey not found".into()))?;

    // 5. Verify user handle if present
    if let Some(ref uh_b64) = response.response.user_handle {
        let uh_bytes = BASE64_URL_SAFE_NO_PAD
            .decode(uh_b64)
            .map_err(|e| Error::RustError(format!("Invalid userHandle base64: {e}")))?;
        let uid_str = String::from_utf8(uh_bytes)
            .map_err(|_| Error::RustError("Invalid userHandle utf8".into()))?;
        if uid_str != passkey.user_id.to_string() {
            return Err(Error::RustError("User Handle mismatch".into()));
        }
    }

    // 6. Verify signature
    let pub_key_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&passkey.public_key)
        .map_err(|e| Error::RustError(format!("Invalid stored public key base64: {e}")))?;

    let client_data_hash = Sha256::digest(&client_data_bytes);
    let mut signed_data = Vec::with_capacity(auth_data_bytes.len() + 32);
    signed_data.extend_from_slice(&auth_data_bytes);
    signed_data.extend_from_slice(&client_data_hash);

    let sig_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&response.response.signature)
        .map_err(|e| Error::RustError(format!("Invalid signature base64: {e}")))?;

    verify_p256_signature(&pub_key_bytes, &signed_data, &sig_bytes)?;

    // 7. Counter check (clone detection)
    if (auth_data.sign_count as i64) <= passkey.counter
        && auth_data.sign_count != 0
        && passkey.counter > 0
    {
        console_error!(
            "Signature counter regression! Stored: {}, Received: {}",
            passkey.counter,
            auth_data.sign_count
        );
        return Err(Error::RustError("Signature counter regression".into()));
    }

    // 8. Update counter
    let now = Date::now().as_millis() as i64;
    store
        .update_passkey_counter(&passkey.cred_id, auth_data.sign_count as i64, now)
        .await?;

    store.delete_state(&state_id).await?;

    // 9. Return the user
    store
        .get_user_by_id(passkey.user_id)
        .await?
        .ok_or_else(|| Error::RustError("User not found".into()))
}

pub async fn list_user_passkeys<S: PasskeyStore>(
    store: &S,
    user_id: i32,
) -> Result<Vec<PasskeySummary>> {
    Ok(store
        .list_passkeys(user_id)
        .await?
        .into_iter()
        .map(|pk| PasskeySummary {
            id: pk.cred_id,
            name: pk.name,
            created_at: pk.created_at,
            last_used_at: pk.last_used_at,
        })
        .collect())
}

pub async fn delete_user_passkey<S: PasskeyStore>(
    store: &S,
    user_id: i32,
    cred_id: &str,
) -> Result<()> {
    store.delete_passkey(user_id, cred_id).await
}

// HTTP Handlers

pub async fn handle_register_start(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let db = auth::get_db(&env)?;
    let config = PasskeyConfig::from_env(&env);
    let options = start_registration(&db, &user, &config).await?;
    Response::from_json(&options)
}

pub async fn handle_register_finish(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };
    let response: RegistrationResponse = req.json().await?;
    let db = auth::get_db(&env)?;
    let config = PasskeyConfig::from_env(&env);
    finish_registration(&db, &user, &config, response).await?;
    Response::ok("Passkey registered")
}

pub async fn handle_login_start(_req: Request, env: Env) -> Result<Response> {
    let db = auth::get_db(&env)?;
    let config = PasskeyConfig::from_env(&env);
    let options = start_login(&db, &config).await?;
    Response::from_json(&options)
}

pub async fn handle_login_finish(mut req: Request, env: Env) -> Result<Response> {
    let response: LoginResponse = req.json().await?;
    let db = auth::get_db(&env)?;
    let config = PasskeyConfig::from_env(&env);
    let user = finish_login(&db, &config, response).await?;

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

    // Verify ownership and existence
    let passkey = db.get_passkey(&body.id).await?;
    match passkey {
        Some(pk) if pk.user_id == user.id => {
            db.update_passkey_name(&body.id, &body.name).await?;
            Response::ok("Renamed")
        }
        Some(_) => Response::error("Unauthorized", 401),
        None => Response::error("Passkey not found", 404),
    }
}
