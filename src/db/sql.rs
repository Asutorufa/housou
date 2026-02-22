use std::borrow::Cow;

#[derive(Clone, Debug)]
pub(crate) enum DatabaseValue {
    Text(String),
    Int(i64),
    Real(f64),
    #[allow(dead_code)]
    Blob(Vec<u8>),
    Null,
}

impl From<i32> for DatabaseValue {
    fn from(v: i32) -> Self {
        DatabaseValue::Int(v as i64)
    }
}

impl From<i64> for DatabaseValue {
    fn from(v: i64) -> Self {
        DatabaseValue::Int(v)
    }
}

impl From<f64> for DatabaseValue {
    fn from(v: f64) -> Self {
        DatabaseValue::Real(v)
    }
}

impl From<String> for DatabaseValue {
    fn from(v: String) -> Self {
        DatabaseValue::Text(v)
    }
}

impl From<&str> for DatabaseValue {
    fn from(v: &str) -> Self {
        DatabaseValue::Text(v.to_string())
    }
}

impl<T> From<Option<T>> for DatabaseValue
where
    T: Into<DatabaseValue>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => DatabaseValue::Null,
        }
    }
}

impl From<crate::model::UserStatus> for DatabaseValue {
    fn from(v: crate::model::UserStatus) -> Self {
        DatabaseValue::Int(v as i64)
    }
}

impl From<&DatabaseValue> for DatabaseValue {
    fn from(v: &DatabaseValue) -> Self {
        v.clone()
    }
}

pub trait FieldUpdate {
    fn field(&self) -> &'static str;
    #[allow(dead_code)]
    fn into_value(self) -> DatabaseValue;
}

impl DatabaseValue {}

pub trait ToParams {
    fn to_params(self) -> Vec<DatabaseValue>;
}

impl<T: Into<DatabaseValue>> ToParams for T {
    fn to_params(self) -> Vec<DatabaseValue> {
        vec![self.into()]
    }
}

impl ToParams for Vec<DatabaseValue> {
    fn to_params(self) -> Vec<DatabaseValue> {
        self
    }
}

pub trait SqlBackend {
    type Param;
    fn convert(value: DatabaseValue) -> Self::Param;
}

pub trait Query {
    fn sql(&self) -> Cow<'static, str>;
    fn values(&self) -> Vec<DatabaseValue>;
}

pub trait QueryExt: Query {
    fn params<B: SqlBackend>(&self) -> Vec<B::Param> {
        self.values().into_iter().map(B::convert).collect()
    }
}

impl<T: Query + ?Sized> QueryExt for T {}

pub trait FieldMeta {
    fn is_primary_key(&self) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub enum MigrationInfo {
    Table(&'static str),
    Index(&'static str),
    Column {
        table: &'static str,
        column: &'static str,
    },
}

impl ToParams for MigrationInfo {
    fn to_params(self) -> Vec<DatabaseValue> {
        vec![]
    }
}

pub(crate) fn filter_updates<T: FieldMeta>(updates: &[T]) -> Vec<&T> {
    updates.iter().filter(|u| !u.is_primary_key()).collect()
}

macro_rules! sql_params {
    ($p:ident $field:ident [sql]) => {
        let _ = $field;
    };
    ($p:ident $field:ident [skip_primary_key]) => {
        let valid_updates = $crate::db::sql::filter_updates($field);
        for u in valid_updates {
            $p.extend(u.clone().to_params());
        }
    };
    ($p:ident $field:ident) => {
        $p.extend($field.clone().to_params());
    };
}

macro_rules! migration_info_helper {
    (@table($name:expr)) => {
        $crate::db::sql::MigrationInfo::Table($name)
    };
    (@index($name:expr)) => {
        $crate::db::sql::MigrationInfo::Index($name)
    };
    (@column($t:expr, $c:expr)) => {
        $crate::db::sql::MigrationInfo::Column {
            table: $t,
            column: $c,
        }
    };
    (@adhoc($info:expr)) => {
        *$info
    };
}

pub trait MigrationMeta {
    fn migration_info(&self) -> Option<MigrationInfo>;
}

macro_rules! impl_migration_meta_for_variants {
    (
        $enum_name:ident<$lt:lifetime>;
        $(
            $( @$mtype:ident ( $($margs:tt)* ) )?
            $name:ident $( { $($field:ident : $ftype:ty $( [ $mode:ident ] )? ),* $(,)? } )? => $sql:expr
        ),* $(,)?
    ) => {
        impl<$lt> $crate::db::MigrationMeta for $enum_name<$lt> {
            fn migration_info(&self) -> Option<$crate::db::sql::MigrationInfo> {
                match self {
                    $(
                        $enum_name::$name $( { $($field,)* } )? => {
                            $( $(let _ = $field;)* )?
                            None $( .or(Some(migration_info_helper!(@$mtype($($margs)*)))) )?
                        }
                    ),*
                }
            }
        }
    };
}

macro_rules! maybe_impl_migration_meta {
    ($enum_name:ident<$lt:lifetime>; [ $($all:tt)* ]; [ @ $mtype:ident ( $($margs:tt)* ) $($rest:tt)* ]) => {
        impl_migration_meta_for_variants!($enum_name<$lt>; $($all)*);
    };
    ($enum_name:ident<$lt:lifetime>; [ $($all:tt)* ]; [ $first:tt $($rest:tt)* ]) => {
        maybe_impl_migration_meta!($enum_name<$lt>; [ $($all)* ]; [ $($rest)* ]);
    };
    ($enum_name:ident<$lt:lifetime>; [ $($all:tt)* ]; [ ]) => {};
}

macro_rules! define_sql {
    (
        $enum_name:ident
        $(
            $( @$mtype:ident ( $($margs:tt)* ) )?
            $name:ident $( { $($field:ident : $ftype:ty $( [ $mode:ident ] )? ),* $(,)? } )? => $sql:expr
        ),* $(,)?
    ) => {
        #[derive(Clone, Debug)]
        pub enum $enum_name<'a> {
            $(
                #[allow(dead_code)]
                $name $( { $($field : $ftype),* } )?,
            )*
        }

        impl<'a> $crate::db::sql::Query for $enum_name<'a> {
            fn sql(&self) -> ::std::borrow::Cow<'static, str> {
                match self {
                    $(
                        $enum_name::$name $( { $($field,)* } )? => {
                             $( $(let _ = $field;)* )?
                             $sql.into()
                        },
                    )*
                }
            }

            fn values(&self) -> Vec<$crate::db::sql::DatabaseValue> {
                if self.sql().is_empty() {
                    Vec::new()
                } else {
                    let mut v = Vec::new();
                    match self {
                        $(
                            $enum_name::$name $( { $($field,)* } )? => {
                                $(
                                    $(
                                        sql_params!(v $field $( [$mode] )? );
                                    )*
                                )?
                            }
                        )*
                    }
                    v
                }
            }
        }

        maybe_impl_migration_meta!(
            $enum_name<'a>;
            [
                $(
                    $( @$mtype ( $($margs)* ) )?
                    $name $( { $($field : $ftype $( [ $mode ] )? ),* } )? => $sql,
                )*
            ];
            [
                $(
                    $( @$mtype ( $($margs)* ) )?
                    $name $( { $($field : $ftype $( [ $mode ] )? ),* } )? => $sql,
                )*
            ]
        );

    };
}

define_sql! {
    Sql
    // General
    Raw { sql: Cow<'a, str> [sql] } => clone_query_sql_cow(sql),
    @adhoc(info)
    AdHoc { info: MigrationInfo, sql: Cow<'a, str> [sql] } => clone_query_sql_cow(sql),

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

    // Schema Checks
    CheckTableExists { name: &'a str } => "SELECT name FROM sqlite_master WHERE type='table' AND name = ?",
    CheckIndexExists { name: &'a str } => "SELECT name FROM sqlite_master WHERE type='index' AND name = ?",
    GetTableInfo { table: &'a str [sql] } => format!("PRAGMA table_info({})", table),

    // Migration Steps
    @table("users")
    CreateUsersTable => "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE,
            username TEXT,
            password_hash TEXT,
            github_id TEXT UNIQUE,
            created_at INTEGER
        );",
    @table("sessions")
    CreateSessionsTable => "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            token TEXT UNIQUE,
            expires_at INTEGER,
            FOREIGN KEY(user_id) REFERENCES users(id)
        );",
    @index("idx_users_username")
    CreateUsersUsernameIndex => "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username);",
    @table("user_items_v2")
    CreateUserItemsV2Table => "CREATE TABLE IF NOT EXISTS user_items_v2 (
            user_id INTEGER,
            title TEXT,
            status INTEGER,
            score INTEGER,
            updated_at INTEGER,
            PRIMARY KEY (user_id, title),
            FOREIGN KEY(user_id) REFERENCES users(id)
        );",
    @index("idx_user_items_v2_user_id")
    CreateUserItemsV2UserIdIndex => "CREATE INDEX IF NOT EXISTS idx_user_items_v2_user_id ON user_items_v2(user_id);",
    @column("users", "avatar_url")
    AddUsersAvatarUrlColumn => "ALTER TABLE users ADD COLUMN avatar_url TEXT;",
    @table("passkeys")
    CreatePasskeysTable => "CREATE TABLE IF NOT EXISTS passkeys (
            user_id INTEGER NOT NULL,
            cred_id TEXT PRIMARY KEY,
            passkey_json TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_used_at INTEGER NOT NULL,
            counter INTEGER NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id)
        );",
    @index("idx_passkeys_user_id")
    CreatePasskeysUserIdIndex => "CREATE INDEX IF NOT EXISTS idx_passkeys_user_id ON passkeys(user_id);",
    @table("passkey_states")
    CreatePasskeyStatesTable => "CREATE TABLE IF NOT EXISTS passkey_states (
            id TEXT PRIMARY KEY,
            state_json TEXT NOT NULL,
            expires_at INTEGER NOT NULL
        );",
    @column("users", "telegram_id")
    AddUsersTelegramIdColumn => "ALTER TABLE users ADD COLUMN telegram_id TEXT;",
    @index("idx_users_telegram_id")
    CreateUsersTelegramIdIndex => "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_telegram_id ON users(telegram_id);",
    @column("user_items_v2", "begin_at")
    AddUserItemsV2BeginAtColumn => "ALTER TABLE user_items_v2 ADD COLUMN begin_at INTEGER;",
    @index("idx_user_items_v2_begin_at")
    CreateUserItemsV2BeginAtIndex => "CREATE INDEX IF NOT EXISTS idx_user_items_v2_begin_at ON user_items_v2(begin_at);",
    @index("idx_user_items_v2_user_id_begin_at")
    CreateUserItemsV2UserIdBeginAtIndex => "CREATE INDEX IF NOT EXISTS idx_user_items_v2_user_id_begin_at ON user_items_v2(user_id, begin_at);",

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
        filter: crate::db::models::UserUpdate,
    } => format!("SELECT * FROM users WHERE {} = ?", filter.field()),
    UpdateUser {
        updates: Vec<crate::db::models::UserUpdate> [skip_primary_key],
        id: i32,
    } => build_update_user_sql(updates),

    // Sessions
    CreateSession {
        user_id: i32,
        token: &'a str,
        expires_at: i64,
    } => "INSERT INTO sessions (user_id, token, expires_at) VALUES (?, ?, ?)",
    GetUserBySessionToken {
        filter: crate::db::models::SessionUpdate,
        now: i64,
    } => format!(
        "SELECT users.* FROM users INNER JOIN sessions ON users.id = sessions.user_id WHERE sessions.{} = ? AND sessions.expires_at > ?",
        filter.field()
    ),
    DeleteSession {
        token: &'a str,
    } => "DELETE FROM sessions WHERE token = ?",

    // User Items
    UpdateUserItem {
        user_id: i32,
        title: &'a str,
        updates: Vec<crate::db::models::UserItemUpdate> [skip_primary_key],
    } => build_update_user_item_sql(updates),
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
    GetPasskeyByField {
        filter: crate::db::models::PasskeyUpdate,
    } => format!("SELECT * FROM passkeys WHERE {} = ?", filter.field()),
    UpdatePasskey {
        updates: Vec<crate::db::models::PasskeyUpdate> [skip_primary_key],
        cred_id: &'a str,
    } => build_update_passkey_sql(updates),
    DeletePasskey {
        user_id: i32,
        cred_id: &'a str,
    } => "DELETE FROM passkeys WHERE user_id = ? AND cred_id = ?",

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
        filter: crate::db::models::PasskeyStateUpdate,
        now: i64,
    } => format!("SELECT * FROM passkey_states WHERE {} = ? AND expires_at > ?", filter.field()),
    DeletePasskeyState {
        id: &'a str,
    } => "DELETE FROM passkey_states WHERE id = ?",
}

struct EffectiveUpdates<'a, T> {
    valid: Vec<&'a T>,
}

impl<'a, T: FieldMeta> EffectiveUpdates<'a, T> {
    fn from_slice(updates: &'a [T]) -> Self {
        Self {
            valid: filter_updates(updates),
        }
    }

    fn is_empty(&self) -> bool {
        self.valid.is_empty()
    }
}

fn clone_query_sql_cow(sql: &Cow<'_, str>) -> Cow<'static, str> {
    match sql {
        Cow::Borrowed(v) => Cow::Owned((*v).to_owned()),
        Cow::Owned(v) => Cow::Owned(v.clone()),
    }
}

fn build_update_assignment_sql<T: FieldMeta + FieldUpdate>(
    table: &str,
    key_field: &str,
    updates: &[T],
) -> Cow<'static, str> {
    let effective = EffectiveUpdates::from_slice(updates);
    if effective.is_empty() {
        Cow::Borrowed("")
    } else {
        let fields = effective
            .valid
            .iter()
            .map(|u| format!("{} = ?", u.field()))
            .collect::<Vec<_>>()
            .join(", ");
        Cow::Owned(format!(
            "UPDATE {} SET {} WHERE {} = ?",
            table, fields, key_field
        ))
    }
}

fn build_update_user_sql(updates: &[crate::db::models::UserUpdate]) -> Cow<'static, str> {
    build_update_assignment_sql("users", "id", updates)
}

fn build_update_passkey_sql(updates: &[crate::db::models::PasskeyUpdate]) -> Cow<'static, str> {
    build_update_assignment_sql("passkeys", "cred_id", updates)
}

fn build_update_user_item_sql(updates: &[crate::db::models::UserItemUpdate]) -> Cow<'static, str> {
    let effective = EffectiveUpdates::from_slice(updates);
    if effective.is_empty() {
        Cow::Borrowed("")
    } else {
        let field_names: Vec<_> = effective.valid.iter().map(|u| u.field()).collect();
        let cols = field_names
            .iter()
            .map(|f| format!(", {}", f))
            .collect::<String>();
        let placeholders = field_names.iter().map(|_| ", ?").collect::<String>();

        let sets = field_names
            .iter()
            .map(|f| {
                if *f == "begin_at" {
                    "begin_at = COALESCE(excluded.begin_at, user_items_v2.begin_at)".to_string()
                } else {
                    format!("{} = excluded.{}", f, f)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        Cow::Owned(format!(
            "INSERT INTO user_items_v2 (user_id, title {}) VALUES (?, ? {}) ON CONFLICT(user_id, title) DO UPDATE SET {}",
            cols, placeholders, sets
        ))
    }
}
