use worker::wasm_bindgen::JsValue;

#[derive(Clone, Debug)]
pub(crate) enum DatabaseValue {
    Text(String),
    Int(i64),
    Real(f64),
    #[allow(dead_code)]
    Blob(Vec<u8>),
    Null,
}

impl DatabaseValue {
    pub fn into_js(self) -> JsValue {
        match self {
            DatabaseValue::Text(s) => JsValue::from_str(&s),
            DatabaseValue::Int(i) => JsValue::from_f64(i as f64),
            DatabaseValue::Real(r) => JsValue::from_f64(r),
            DatabaseValue::Blob(_) => JsValue::NULL, // TODO: support blobs if needed
            DatabaseValue::Null => JsValue::NULL,
        }
    }
}

pub(crate) trait ToDatabaseValue {
    fn to_value(self) -> DatabaseValue;
}

impl ToDatabaseValue for i32 {
    fn to_value(self) -> DatabaseValue {
        DatabaseValue::Int(self as i64)
    }
}

impl ToDatabaseValue for i64 {
    fn to_value(self) -> DatabaseValue {
        DatabaseValue::Int(self)
    }
}

impl ToDatabaseValue for f64 {
    fn to_value(self) -> DatabaseValue {
        DatabaseValue::Real(self)
    }
}

impl ToDatabaseValue for &str {
    fn to_value(self) -> DatabaseValue {
        DatabaseValue::Text(self.to_string())
    }
}

impl ToDatabaseValue for String {
    fn to_value(self) -> DatabaseValue {
        DatabaseValue::Text(self)
    }
}

impl ToDatabaseValue for Option<&str> {
    fn to_value(self) -> DatabaseValue {
        match self {
            Some(s) => DatabaseValue::Text(s.to_string()),
            None => DatabaseValue::Null,
        }
    }
}

impl ToDatabaseValue for JsValue {
    fn to_value(self) -> DatabaseValue {
        if self.is_null() || self.is_undefined() {
            DatabaseValue::Null
        } else if let Some(s) = self.as_string() {
            DatabaseValue::Text(s)
        } else if let Some(f) = self.as_f64() {
            DatabaseValue::Real(f)
        } else {
            DatabaseValue::Null
        }
    }
}

impl ToDatabaseValue for DatabaseValue {
    fn to_value(self) -> DatabaseValue {
        self
    }
}

impl ToDatabaseValue for &DatabaseValue {
    fn to_value(self) -> DatabaseValue {
        self.clone()
    }
}

trait CollectParams {
    fn collect_params(self, params: &mut Vec<DatabaseValue>);
}

impl<T: ToDatabaseValue> CollectParams for T {
    fn collect_params(self, params: &mut Vec<DatabaseValue>) {
        params.push(self.to_value());
    }
}

impl CollectParams for Vec<DatabaseValue> {
    fn collect_params(self, params: &mut Vec<DatabaseValue>) {
        params.extend(self);
    }
}

impl CollectParams for Vec<JsValue> {
    fn collect_params(self, params: &mut Vec<DatabaseValue>) {
        for v in self {
            params.push(v.to_value());
        }
    }
}

macro_rules! sql_params {
    ($p:ident $field:ident [sql]) => {
        let _ = $field;
    };
    ($p:ident $field:ident) => {
        $field.clone().collect_params($p);
    };
}

macro_rules! define_sql {
    (
        $(
            $name:ident $( { $($field:ident : $ftype:ty $( [ $mode:ident ] )? ),* $(,)? } )? => $sql:expr
        ),* $(,)?
    ) => {
        #[derive(Clone, Debug)]
        pub enum Sql<'a> {
            Raw { sql: &'a str },
            $(
                $name $( { $($field : $ftype),* } )?,
            )*
        }

        impl<'a> Sql<'a> {
            pub fn sql(&self) -> String {
                match self {
                    Sql::Raw { sql } => sql.to_string(),
                    $(
                        Sql::$name $( { $($field,)* } )? => {
                             $( $(let _ = $field;)* )?
                             $sql.into()
                        },
                    )*
                }
            }

            pub fn values(&self) -> Vec<DatabaseValue> {
                let mut v = Vec::new();
                let values = &mut v;
                match self {
                    Sql::Raw { .. } => {}
                    $(
                        Sql::$name $( { $($field,)* } )? => {
                            $(
                                $(
                                    sql_params!(values $field $( [$mode] )? );
                                )*
                            )?
                        }
                    )*
                }
                v
            }

            pub fn params(&self) -> Vec<JsValue> {
                self.values().into_iter().map(|v| v.into_js()).collect()
            }
        }
    };
}

define_sql! {
    // Migrations
    CreateMigrationsTable => "CREATE TABLE IF NOT EXISTS schema_migrations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        version INTEGER NOT NULL UNIQUE,
        applied_at INTEGER NOT NULL
    );",
    GetSchemaVersion => "SELECT MAX(version) as version FROM schema_migrations",
    InsertMigration {
        version: i32,
        applied_at: i64,
    } => "INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)",

    // Users
    CreateUser {
        email: &'a str,
        username: &'a str,
        password_hash: Option<&'a str>,
        github_id: Option<&'a str>,
        telegram_id: Option<&'a str>,
        avatar_url: Option<&'a str>,
        created_at: i64,
    } => "INSERT INTO users (email, username, password_hash, github_id, telegram_id, avatar_url, created_at) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *",
    GetUserByField {
        field: &'a str [sql],
        value: DatabaseValue,
    } => format!("SELECT * FROM users WHERE {} = ?", field),
    UpdateUserField {
        field: &'a str [sql],
        value: DatabaseValue,
        id: i32,
    } => format!("UPDATE users SET {} = ? WHERE id = ?", field),
    UpdateUserProfile {
        updates: &'a str [sql],
        params: Vec<DatabaseValue>,
        id: i32,
    } => format!("UPDATE users SET {} WHERE id = ?", updates),

    // Sessions
    CreateSession {
        user_id: i32,
        token: &'a str,
        expires_at: i64,
    } => "INSERT INTO sessions (user_id, token, expires_at) VALUES (?, ?, ?)",
    GetUserBySessionToken {
        token: &'a str,
        now: i64,
    } => "SELECT users.* FROM users INNER JOIN sessions ON users.id = sessions.user_id WHERE sessions.token = ? AND sessions.expires_at > ?",
    DeleteSession {
        token: &'a str,
    } => "DELETE FROM sessions WHERE token = ?",

    // User Items
    UpdateUserItem {
        user_id: i32,
        title: &'a str,
        status: i32,
        score: DatabaseValue,
        updated_at: i64,
        begin_at: DatabaseValue,
    } => "INSERT INTO user_items_v2 (user_id, title, status, score, updated_at, begin_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, title) DO UPDATE SET 
                status = excluded.status, 
                score = excluded.score, 
                updated_at = excluded.updated_at,
                begin_at = COALESCE(excluded.begin_at, user_items_v2.begin_at)",
    GetUserItemsAll {
        user_id: i32,
    } => "SELECT * FROM user_items_v2 WHERE user_id = ? AND status != 0",
    GetUserItemsByRange {
        user_id: i32,
        start_ts: i64,
        end_ts: i64,
    } => "SELECT * FROM user_items_v2
             WHERE user_id = ? AND status != 0 
             AND (begin_at IS NULL OR (begin_at >= ? AND begin_at <= ?))",

    // Passkeys
    CreatePasskey {
        user_id: i32,
        cred_id: &'a str,
        passkey_json: &'a str,
        name: &'a str,
        created_at: i64,
        last_used_at: i64,
        counter: i64,
    } => "INSERT INTO passkeys (user_id, cred_id, passkey_json, name, created_at, last_used_at, counter) VALUES (?, ?, ?, ?, ?, ?, ?)",
    GetPasskey {
        cred_id: &'a str,
    } => "SELECT * FROM passkeys WHERE cred_id = ?",
    ListPasskeys {
        user_id: i32,
    } => "SELECT * FROM passkeys WHERE user_id = ?",
    DeletePasskey {
        user_id: i32,
        cred_id: &'a str,
    } => "DELETE FROM passkeys WHERE user_id = ? AND cred_id = ?",
    UpdatePasskeyCounter {
        counter: i64,
        last_used_at: i64,
        cred_id: &'a str,
    } => "UPDATE passkeys SET counter = ?, last_used_at = ? WHERE cred_id = ?",
    UpdatePasskeyName {
        name: &'a str,
        cred_id: &'a str,
    } => "UPDATE passkeys SET name = ? WHERE cred_id = ?",

    // Passkey States
    CleanupPasskeyStates {
        now: i64,
    } => "DELETE FROM passkey_states WHERE expires_at < ?",
    SavePasskeyState {
        id: &'a str,
        state_json: &'a str,
        expires_at: i64,
    } => "INSERT OR REPLACE INTO passkey_states (id, state_json, expires_at) VALUES (?, ?, ?)",
    GetPasskeyState {
        id: &'a str,
        now: i64,
    } => "SELECT * FROM passkey_states WHERE id = ? AND expires_at > ?",
    DeletePasskeyState {
        id: &'a str,
    } => "DELETE FROM passkey_states WHERE id = ?",
}
