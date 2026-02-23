use std::borrow::Cow;

#[derive(Clone, Debug)]
pub(crate) enum DatabaseValue {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Real(f64),
    Text(String),
    #[allow(dead_code)]
    Blob(Vec<u8>),
}

impl From<i32> for DatabaseValue {
    fn from(v: i32) -> Self {
        DatabaseValue::Int(v as i64)
    }
}

impl From<u32> for DatabaseValue {
    fn from(v: u32) -> Self {
        DatabaseValue::UInt(v as u64)
    }
}

impl From<i64> for DatabaseValue {
    fn from(v: i64) -> Self {
        DatabaseValue::Int(v)
    }
}

impl From<u64> for DatabaseValue {
    fn from(v: u64) -> Self {
        DatabaseValue::UInt(v)
    }
}

impl From<bool> for DatabaseValue {
    fn from(v: bool) -> Self {
        DatabaseValue::Bool(v)
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
    fn into_value(&self) -> DatabaseValue;
}

impl DatabaseValue {}

pub trait ToParams {
    fn add_params(&self, params: &mut Vec<DatabaseValue>);
}

impl<T: Clone + Into<DatabaseValue>> ToParams for T {
    fn add_params(&self, params: &mut Vec<DatabaseValue>) {
        params.push(self.clone().into());
    }
}

impl ToParams for Vec<DatabaseValue> {
    fn add_params(&self, params: &mut Vec<DatabaseValue>) {
        for v in self {
            params.push(v.clone());
        }
    }
}

pub trait IntoOptionCow {
    fn into_option_cow(self) -> Option<Cow<'static, str>>;
}

impl IntoOptionCow for &'static str {
    fn into_option_cow(self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(self))
    }
}

impl IntoOptionCow for String {
    fn into_option_cow(self) -> Option<Cow<'static, str>> {
        Some(Cow::Owned(self))
    }
}

impl IntoOptionCow for Cow<'static, str> {
    fn into_option_cow(self) -> Option<Cow<'static, str>> {
        Some(self)
    }
}

impl IntoOptionCow for Option<Cow<'static, str>> {
    fn into_option_cow(self) -> Option<Cow<'static, str>> {
        self
    }
}

pub trait SqlBackend {
    type Param;
    fn convert(value: DatabaseValue) -> Self::Param;
}

pub trait Query {
    fn build(&self) -> Option<(Cow<'static, str>, Vec<DatabaseValue>)>;
}

pub trait QueryExt: Query {
    fn build_params<B: SqlBackend>(&self) -> Option<(Cow<'static, str>, Vec<B::Param>)> {
        self.build()
            .map(|(sql, values)| (sql, values.into_iter().map(B::convert).collect()))
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
    fn add_params(&self, _params: &mut Vec<DatabaseValue>) {}
}

macro_rules! sql_params {
    ($p:ident sql) => {};
    ($p:ident info) => {};
    ($p:ident $field:ident [skip_primary_key]) => {
        for u in $field.iter().filter(|u| !u.is_primary_key()) {
            $p.push(u.into_value());
        }
    };
    ($p:ident $field:ident) => {
        $crate::db::sql::ToParams::add_params($field, &mut $p);
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
            fn build(&self) -> Option<(::std::borrow::Cow<'static, str>, Vec<$crate::db::sql::DatabaseValue>)> {
                match self {
                    $(
                        $enum_name::$name $( { $($field,)* } )? => {
                            $( $(let _ = &$field;)* )?
                            let sql: Option<::std::borrow::Cow<'static, str>> = $crate::db::sql::IntoOptionCow::into_option_cow($sql);
                            sql.map(|sql| {
                                #[allow(unused_mut)]
                                let mut v = Vec::new();
                                $(
                                    $(
                                        sql_params!(v $field $( [$mode] )? );
                                    )*
                                )?
                                (sql, v)
                            })
                        },
                    )*
                }
            }
        }

        impl<'a> $crate::db::MigrationMeta for $enum_name<'a> {
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

define_sql! {
    Sql
    // General
    @adhoc(info)
    AdHoc { info: MigrationInfo, sql: Cow<'static, str> } => sql.clone(),

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
    GetTableInfo { table: &'a str } => "SELECT * FROM pragma_table_info(?)",

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

fn build_update_assignment_sql<T: FieldMeta + FieldUpdate>(
    table: &str,
    key_field: &str,
    updates: &[T],
) -> Option<Cow<'static, str>> {
    let valid = updates.iter().filter(|u| !u.is_primary_key());
    let count = valid.clone().count();
    if count == 0 {
        return None;
    }

    use std::fmt::Write;
    let mut sql = String::with_capacity(64 + table.len() + key_field.len() + count * 40);
    write!(sql, "UPDATE {} SET ", table).unwrap();

    for (i, u) in valid.enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        write!(sql, "{} = ?", u.field()).unwrap();
    }
    write!(sql, " WHERE {} = ?", key_field).unwrap();

    Some(Cow::Owned(sql))
}

fn build_update_user_sql(updates: &[crate::db::models::UserUpdate]) -> Option<Cow<'static, str>> {
    build_update_assignment_sql("users", "id", updates)
}

fn build_update_passkey_sql(
    updates: &[crate::db::models::PasskeyUpdate],
) -> Option<Cow<'static, str>> {
    build_update_assignment_sql("passkeys", "cred_id", updates)
}

fn build_update_user_item_sql(
    updates: &[crate::db::models::UserItemUpdate],
) -> Option<Cow<'static, str>> {
    let valid = updates.iter().filter(|u| !u.is_primary_key());
    let count = valid.clone().count();
    if count == 0 {
        return None;
    }

    use std::fmt::Write;
    let mut sql = String::with_capacity(64 + "user_items_v2".len() + "user_id".len() + count * 40);
    sql.push_str("INSERT INTO user_items_v2 (user_id, title");

    for u in valid.clone() {
        write!(sql, ", {}", u.field()).unwrap();
    }

    sql.push_str(") VALUES (?, ?");
    for _ in 0..count {
        sql.push_str(", ?");
    }

    sql.push_str(") ON CONFLICT(user_id, title) DO UPDATE SET ");

    for (i, u) in valid.enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        let f = u.field();
        if f == "begin_at" {
            sql.push_str("begin_at = COALESCE(excluded.begin_at, user_items_v2.begin_at)");
        } else {
            write!(sql, "{} = excluded.{}", f, f).unwrap();
        }
    }

    Some(Cow::Owned(sql))
}
