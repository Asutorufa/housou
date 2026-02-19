use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PasskeyConfig {
    pub rp_id: String,
    pub rp_name: String,
    pub origin: String,
    pub state_ttl: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredPasskey {
    pub user_id: String,
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
