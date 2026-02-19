use super::*;
use crate::error::Result;
use crate::store::PasskeyStore;
use crate::types::*;
use async_trait::async_trait;
use base64::prelude::*;
use coset::{Algorithm, CborSerializable, CoseKey, KeyType, Label, iana};
use p256::SecretKey;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::elliptic_curve::rand_core::OsRng;

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// --- Mock Store ---

#[derive(Clone, Default)]
struct MockStore {
    passkeys: Arc<Mutex<HashMap<String, StoredPasskey>>>, // Keyed by cred_id
    states: Arc<Mutex<HashMap<String, PasskeyState>>>,
}

#[async_trait(?Send)]
impl PasskeyStore for MockStore {
    async fn create_passkey(
        &self,
        user_id: i32,
        cred_id: &str,
        public_key: &str,
        name: &str,
        counter: i64,
        created_at: i64,
    ) -> Result<()> {
        let pk = StoredPasskey {
            user_id,
            cred_id: cred_id.to_string(),
            public_key: public_key.to_string(),
            name: name.to_string(),
            created_at,
            last_used_at: created_at,
            counter,
        };
        self.passkeys
            .lock()
            .unwrap()
            .insert(cred_id.to_string(), pk);
        Ok(())
    }

    async fn get_passkey(&self, cred_id: &str) -> Result<Option<StoredPasskey>> {
        Ok(self.passkeys.lock().unwrap().get(cred_id).cloned())
    }

    async fn list_passkeys(&self, user_id: i32) -> Result<Vec<StoredPasskey>> {
        Ok(self
            .passkeys
            .lock()
            .unwrap()
            .values()
            .filter(|pk| pk.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete_passkey(&self, user_id: i32, cred_id: &str) -> Result<()> {
        let mut passkeys = self.passkeys.lock().unwrap();
        if let Some(pk) = passkeys.get(cred_id) {
            if pk.user_id == user_id {
                passkeys.remove(cred_id);
            }
        }
        Ok(())
    }

    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> Result<()> {
        if let Some(pk) = self.passkeys.lock().unwrap().get_mut(cred_id) {
            pk.counter = new_counter;
            pk.last_used_at = last_used_at;
        }
        Ok(())
    }

    async fn update_passkey_name(&self, cred_id: &str, new_name: &str) -> Result<()> {
        if let Some(pk) = self.passkeys.lock().unwrap().get_mut(cred_id) {
            pk.name = new_name.to_string();
        }
        Ok(())
    }

    async fn save_state(&self, id: &str, state_json: &str, expires_at: i64) -> Result<()> {
        let state = PasskeyState {
            id: id.to_string(),
            state_json: state_json.to_string(),
            expires_at,
        };
        self.states.lock().unwrap().insert(id.to_string(), state);
        Ok(())
    }

    async fn get_state(&self, id: &str) -> Result<Option<PasskeyState>> {
        Ok(self.states.lock().unwrap().get(id).cloned())
    }

    async fn delete_state(&self, id: &str) -> Result<()> {
        self.states.lock().unwrap().remove(id);
        Ok(())
    }
}

// --- Helpers ---

fn make_client_data(challenge: &str, origin: &str, type_: &str) -> String {
    let json = serde_json::json!({
        "challenge": challenge,
        "origin": origin,
        "type": type_,
    });
    let bytes = serde_json::to_vec(&json).unwrap();
    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}

fn make_cose_key(public_key: &VerifyingKey) -> Vec<u8> {
    let encoded = public_key.to_encoded_point(false);
    let x = encoded.x().unwrap().as_slice();
    let y = encoded.y().unwrap().as_slice();

    let key = CoseKey {
        kty: KeyType::Assigned(iana::KeyType::EC2),
        key_id: vec![],
        alg: Some(Algorithm::Assigned(iana::Algorithm::ES256)),
        key_ops: Default::default(),
        base_iv: vec![],
        params: vec![
            (Label::Int(-1), coset::cbor::value::Value::Integer(1.into())), // P-256
            (Label::Int(-2), coset::cbor::value::Value::Bytes(x.to_vec())), // x
            (Label::Int(-3), coset::cbor::value::Value::Bytes(y.to_vec())), // y
        ],
    };
    key.to_vec().unwrap()
}

fn make_auth_data(
    rp_id: &str,
    flags: u8,
    counter: u32,
    cred_id: Option<&[u8]>,
    public_key: Option<&[u8]>,
) -> Vec<u8> {
    let mut buf = Vec::new();
    // rpIdHash
    let hash = Sha256::digest(rp_id.as_bytes());
    buf.extend_from_slice(&hash);
    // flags
    buf.push(flags);
    // counter
    buf.extend_from_slice(&counter.to_be_bytes());

    // attested credential data
    if let (Some(cid), Some(pk)) = (cred_id, public_key) {
        // AAGUID (16 bytes zeros)
        buf.extend_from_slice(&[0u8; 16]);
        // Credential ID length
        buf.extend_from_slice(&(cid.len() as u16).to_be_bytes());
        // Credential ID
        buf.extend_from_slice(cid);
        // Public Key
        buf.extend_from_slice(pk);
    }
    buf
}

fn make_attestation_object(auth_data: &[u8]) -> String {
    let map = ciborium::value::Value::Map(vec![
        (
            ciborium::value::Value::Text("fmt".to_string()),
            ciborium::value::Value::Text("none".to_string()),
        ),
        (
            ciborium::value::Value::Text("attStmt".to_string()),
            ciborium::value::Value::Map(vec![]),
        ),
        (
            ciborium::value::Value::Text("authData".to_string()),
            ciborium::value::Value::Bytes(auth_data.to_vec()),
        ),
    ]);
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&map, &mut bytes).unwrap();
    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}

// --- Tests ---

#[tokio::test]
async fn test_registration_flow() {
    let store = MockStore::default();
    let user_id = 123;
    let username = "testuser";
    let display_name = "Test User";
    let origin = "https://example.com";
    let rp_id = "example.com";

    let config = PasskeyConfig {
        rp_id: rp_id.to_string(),
        rp_name: "Test RP".to_string(),
        origin: origin.to_string(),
    };

    let now = 1000000000;

    // 1. Start Registration
    let options = start_registration(&store, user_id, username, display_name, &config, now)
        .await
        .expect("Start registration failed");

    // Verify state
    let state_id = format!("reg:{}", user_id);
    let saved_state = store
        .get_state(&state_id)
        .await
        .unwrap()
        .expect("State not saved");
    assert!(saved_state.expires_at > now);

    // 2. Prepare Client Response
    let challenge = options.challenge;
    let client_data_json = make_client_data(&challenge, origin, "webauthn.create");

    // Generate key pair
    let secret_key = SecretKey::random(&mut OsRng);
    let signing_key = SigningKey::from(secret_key);
    let public_key = VerifyingKey::from(&signing_key);
    let cose_key = make_cose_key(&public_key);

    let cred_id = b"credential_id_123";

    // Auth Data (flags: UP=1, AT=1, UV=0) -> 0x41
    let auth_data = make_auth_data(rp_id, 0x41, 0, Some(cred_id), Some(&cose_key));
    let attestation_object = make_attestation_object(&auth_data);

    let response = RegistrationResponse {
        id: BASE64_URL_SAFE_NO_PAD.encode(cred_id),
        raw_id: BASE64_URL_SAFE_NO_PAD.encode(cred_id),
        type_: "public-key".to_string(),
        response: AttestationResponse {
            client_data_json,
            attestation_object,
        },
        client_extension_results: None,
        name: Some("My Passkey".to_string()),
    };

    // 3. Finish Registration
    finish_registration(&store, user_id, &config, response, now)
        .await
        .expect("Finish registration failed");

    // Verify stored credential
    let stored = store
        .get_passkey(&BASE64_URL_SAFE_NO_PAD.encode(cred_id))
        .await
        .unwrap()
        .expect("Passkey not stored");

    assert_eq!(stored.user_id, user_id);
    assert_eq!(stored.name, "My Passkey");
    assert!(stored.created_at == now);
}

#[tokio::test]
async fn test_login_flow() {
    let store = MockStore::default();
    let user_id = 456;
    let cred_id_bytes = b"login_cred_id";
    let cred_id = BASE64_URL_SAFE_NO_PAD.encode(cred_id_bytes);

    let origin = "https://login.com";
    let rp_id = "login.com";
    let config = PasskeyConfig {
        rp_id: rp_id.to_string(),
        rp_name: "Login RP".to_string(),
        origin: origin.to_string(),
    };
    let now = 2000000000;

    // Pre-register a key
    let secret_key = SecretKey::random(&mut OsRng);
    let signing_key = SigningKey::from(secret_key.clone());
    let public_key = VerifyingKey::from(&signing_key);
    let cose_key = make_cose_key(&public_key);
    let pub_key_b64 = BASE64_URL_SAFE_NO_PAD.encode(&cose_key);

    store
        .create_passkey(user_id, &cred_id, &pub_key_b64, "Login Key", 10, now - 1000)
        .await
        .unwrap();

    // 1. Start Login
    let options = start_login(&store, &config, now)
        .await
        .expect("Start login failed");

    let challenge = options.challenge;

    // 2. Prepare Client Response
    let client_data_json_b64 = make_client_data(&challenge, origin, "webauthn.get");
    let client_data_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&client_data_json_b64)
        .unwrap();

    // Auth Data (flags: UP=1) -> 0x01. Counter = 11 (must be > 10)
    let auth_data = make_auth_data(rp_id, 0x01, 11, None, None);

    // Sign (authData + clientDataHash)
    let client_data_hash = Sha256::digest(&client_data_bytes);
    let mut signed_data = Vec::new();
    signed_data.extend_from_slice(&auth_data);
    signed_data.extend_from_slice(&client_data_hash);

    let signing_key = SigningKey::from(secret_key);
    let signature: p256::ecdsa::Signature = signing_key.sign(&signed_data);
    let signature_der = signature.to_der();
    let signature_b64 = BASE64_URL_SAFE_NO_PAD.encode(signature_der.as_bytes());

    let response = LoginResponse {
        id: cred_id.clone(),
        raw_id: cred_id.clone(),
        type_: "public-key".to_string(),
        response: AssertionResponse {
            client_data_json: client_data_json_b64,
            authenticator_data: BASE64_URL_SAFE_NO_PAD.encode(&auth_data),
            signature: signature_b64,
            user_handle: Some(BASE64_URL_SAFE_NO_PAD.encode(user_id.to_string().as_bytes())),
        },
        client_extension_results: None,
    };

    // 3. Finish Login
    let logged_in_uid = finish_login(&store, &config, response, now)
        .await
        .expect("Finish login failed");

    assert_eq!(logged_in_uid, user_id);

    // Verify counter updated
    let stored = store.get_passkey(&cred_id).await.unwrap().unwrap();
    assert_eq!(stored.counter, 11);
    assert_eq!(stored.last_used_at, now);
}

#[tokio::test]
async fn test_origin_mismatch() {
    let store = MockStore::default();
    let config = PasskeyConfig {
        rp_id: "rp.com".into(),
        rp_name: "RP".into(),
        origin: "https://rp.com".into(),
    };

    let now = 100;
    let options = start_login(&store, &config, now).await.unwrap();

    // Use wrong origin
    let client_data_json_b64 =
        make_client_data(&options.challenge, "https://attacker.com", "webauthn.get");

    let response = LoginResponse {
        id: "any".into(),
        raw_id: "any".into(),
        type_: "public-key".into(),
        response: AssertionResponse {
            client_data_json: client_data_json_b64,
            authenticator_data: "aaaa".into(), // valid base64 but invalid content, will fail earlier?
            // Actually it fails at verify_client_data before parsing authData
            signature: "aaaa".into(),
            user_handle: None,
        },
        client_extension_results: None,
    };

    let result = finish_login(&store, &config, response, now).await;
    match result {
        Err(PasskeyError::OriginMismatch { .. }) => (),
        _ => panic!("Expected OriginMismatch, got {:?}", result),
    }
}
