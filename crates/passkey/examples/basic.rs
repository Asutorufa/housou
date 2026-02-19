use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use passkey_server::types::{
    LoginResponse, PasskeyState, PublicKeyCredentialCreationOptions,
    PublicKeyCredentialRequestOptions, RegistrationResponse, StoredPasskey,
};
use passkey_server::{
    PasskeyConfig, PasskeyStore, Result as PasskeyResult, finish_login, finish_registration,
    start_login, start_registration,
};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// --- In-Memory Store Implementation ---

struct MemoryStore {
    passkeys: Mutex<HashMap<String, StoredPasskey>>,
    states: Mutex<HashMap<String, PasskeyState>>,
}

#[cfg_attr(not(feature = "send"), async_trait(?Send))]
#[cfg_attr(feature = "send", async_trait)]
impl PasskeyStore for MemoryStore {
    async fn create_passkey(
        &self,
        user_id: i32,
        cred_id: &str,
        public_key: &str,
        name: &str,
        counter: i64,
        created_at: i64,
    ) -> PasskeyResult<()> {
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

    async fn get_passkey(&self, cred_id: &str) -> PasskeyResult<Option<StoredPasskey>> {
        Ok(self.passkeys.lock().unwrap().get(cred_id).cloned())
    }

    async fn list_passkeys(&self, user_id: i32) -> PasskeyResult<Vec<StoredPasskey>> {
        let pks = self.passkeys.lock().unwrap();
        Ok(pks
            .values()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete_passkey(&self, _user_id: i32, cred_id: &str) -> PasskeyResult<()> {
        self.passkeys.lock().unwrap().remove(cred_id);
        Ok(())
    }

    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> PasskeyResult<()> {
        if let Some(pk) = self.passkeys.lock().unwrap().get_mut(cred_id) {
            pk.counter = new_counter;
            pk.last_used_at = last_used_at;
        }
        Ok(())
    }

    async fn update_passkey_name(&self, cred_id: &str, new_name: &str) -> PasskeyResult<()> {
        if let Some(pk) = self.passkeys.lock().unwrap().get_mut(cred_id) {
            pk.name = new_name.to_string();
        }
        Ok(())
    }

    async fn save_state(&self, id: &str, state_json: &str, expires_at: i64) -> PasskeyResult<()> {
        let state = PasskeyState {
            id: id.to_string(),
            state_json: state_json.to_string(),
            expires_at,
        };
        self.states.lock().unwrap().insert(id.to_string(), state);
        Ok(())
    }

    async fn get_state(&self, id: &str) -> PasskeyResult<Option<PasskeyState>> {
        Ok(self.states.lock().unwrap().get(id).cloned())
    }

    async fn delete_state(&self, id: &str) -> PasskeyResult<()> {
        self.states.lock().unwrap().remove(id);
        Ok(())
    }
}

// --- Axum Handlers ---

struct AppState {
    store: MemoryStore,
    config: PasskeyConfig,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

async fn register_start(
    State(state): State<Arc<AppState>>,
) -> Json<PublicKeyCredentialCreationOptions> {
    let user_id = 1; // In real app, get from session
    let options = start_registration(
        &state.store,
        user_id,
        "alice@example.com",
        "Alice",
        &state.config,
        now_ms(),
    )
    .await
    .unwrap();
    Json(options)
}

async fn register_finish(
    State(state): State<Arc<AppState>>,
    Json(response): Json<RegistrationResponse>,
) -> &'static str {
    let user_id = 1;
    finish_registration(&state.store, user_id, &state.config, response, now_ms())
        .await
        .unwrap();
    "Registration successful"
}

async fn login_start(
    State(state): State<Arc<AppState>>,
) -> Json<PublicKeyCredentialRequestOptions> {
    let options = start_login(&state.store, &state.config, now_ms())
        .await
        .unwrap();
    Json(options)
}

async fn login_finish(
    State(state): State<Arc<AppState>>,
    Json(response): Json<LoginResponse>,
) -> String {
    let user_id = finish_login(&state.store, &state.config, response, now_ms())
        .await
        .unwrap();
    format!("Login successful for user: {}", user_id)
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState {
        store: MemoryStore {
            passkeys: Mutex::new(HashMap::new()),
            states: Mutex::new(HashMap::new()),
        },
        config: PasskeyConfig {
            rp_id: "localhost".to_string(),
            rp_name: "Example Service".to_string(),
            origin: "http://localhost:3000".to_string(),
        },
    });

    let app = Router::new()
        .route("/auth/register/start", get(register_start))
        .route("/auth/register/finish", post(register_finish))
        .route("/auth/login/start", get(login_start))
        .route("/auth/login/finish", post(login_finish))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
