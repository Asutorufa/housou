# Passkey

A storage-agnostic Rust implementation of the WebAuthn (Passkey) protocol.

## Features

- **Generic Storage**: Implement the `PasskeyStore` trait to use any database.
- **WASM Compatible**: Optimized for WASM environments like Cloudflare Workers (use `wasm` feature).
- **Native Support**: Fully compatible with multi-threaded runtimes like Tokio (enabled by default).
- **Easy Integration**: Provides high-level handlers for registration and authentication.
- **Full Example**: See `examples/basic.rs` for a complete Axum-based HTTP server implementation.

## Installation


Add this to your `Cargo.toml`:

```toml
[dependencies]
# For native applications (Tokio, etc.)
passkey-server = { path = "path/to/crates/passkey-server" }

# For Cloudflare Workers / WASM (non-Send)
passkey-server = { path = "path/to/crates/passkey-server", default-features = false, features = ["wasm"] }
```


## Quick Start

### 1. Implement `PasskeyStore`

```rust
use async_trait::async_trait;
use passkey-server::{PasskeyStore, Result};
use passkey-server::types::{StoredPasskey, PasskeyState};

struct MyDb;

#[async_trait(?Send)]
impl PasskeyStore for MyDb {
    async fn create_passkey(&self, user_id: i32, cred_id: &str, public_key: &str, name: &str, counter: i64, created_at: i64) -> Result<()> {

        // Save to DB
        Ok(())
    }
    // ... implement other methods
}
```

### 2. Registration Flow

```rust
use passkey-server::{PasskeyConfig, start_registration};

async fn handle_registration_start() {
    let config = PasskeyConfig {
        rp_id: "localhost".into(),
        rp_name: "Housou".into(),
        origin: "http://localhost:8787".into(),
    };
    let store = MyDb;
    let now = 1708358400000;

    let options = start_registration(&store, 1, "alice", "Alice", &config, now).await.unwrap();
    // Return options to frontend
}
```

## Credits

Part of the [Housou](https://github.com/Asutorufa/housou) project.
