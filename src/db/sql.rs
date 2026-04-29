use d1_orm::{FieldUpdate, MigrationInfo, UpsertConfig, build_update_sql, build_upsert_sql};
use std::borrow::Cow;

d1_orm::define_sql! {
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
    @table("comments")
    CreateCommentsTable => "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            title TEXT,
            content TEXT,
            created_at INTEGER,
            FOREIGN KEY(user_id) REFERENCES users(id),
            UNIQUE(user_id, title)
        );",
    @index("idx_comments_title")
    CreateCommentsTitleIndex => "CREATE INDEX IF NOT EXISTS idx_comments_title ON comments(title);",
    @column("comments", "score")
    AddCommentsScoreColumn => "ALTER TABLE comments ADD COLUMN score INTEGER;",
    @column("comments", "updated_at")
    AddCommentsUpdatedAtColumn => "ALTER TABLE comments ADD COLUMN updated_at INTEGER;",
    BackfillCommentsUpdatedAt => "UPDATE comments SET updated_at = created_at WHERE updated_at IS NULL;",

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
    } => build_update_sql("users", "id", updates),

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
    } => {
        let config = UpsertConfig {
            table: "user_items_v2",
            primary_keys: &["user_id", "title"],
            custom_conflict_resolution: Some(&|field| {
                if field == "begin_at" {
                    Some("begin_at = COALESCE(excluded.begin_at, user_items_v2.begin_at)")
                } else {
                    None
                }
            }),
        };
        build_upsert_sql(&config, updates)
    },
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
    } => build_update_sql("passkeys", "cred_id", updates),
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

    // Comments
    CreateComment {
        user_id: i32,
        title: &'a str,
        content: &'a str,
        score: Option<i32>,
        created_at: i64,
        updated_at: i64,
    } => "INSERT INTO comments (user_id, title, content, score, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?) RETURNING id, user_id as userId, title, content, score, created_at as createdAt, updated_at as updatedAt",
    GetCommentsWithUser {
        title: &'a str,
        viewer_id: i32,
        limit: i32,
        offset: i32,
    } => "SELECT
            c.id, c.user_id as userId, u.username, u.avatar_url as avatarUrl, c.content, c.created_at as createdAt, c.updated_at as updatedAt,
            COALESCE(c.score, ui.score) as score
          FROM comments c
          INNER JOIN users u ON c.user_id = u.id
          LEFT JOIN user_items_v2 ui ON c.user_id = ui.user_id AND c.title = ui.title
          WHERE c.title = ?
          ORDER BY CASE WHEN c.user_id = ? THEN 0 ELSE 1 END, c.updated_at DESC, c.created_at DESC
          LIMIT ? OFFSET ?",
    GetCommentsWithUserGuest {
        title: &'a str,
        limit: i32,
        offset: i32,
    } => "SELECT
            c.id, c.user_id as userId, u.username, u.avatar_url as avatarUrl, c.content, c.created_at as createdAt, c.updated_at as updatedAt,
            COALESCE(c.score, ui.score) as score
          FROM comments c
          INNER JOIN users u ON c.user_id = u.id
          LEFT JOIN user_items_v2 ui ON c.user_id = ui.user_id AND c.title = ui.title
          WHERE c.title = ?
          ORDER BY c.updated_at DESC, c.created_at DESC
          LIMIT ? OFFSET ?",
    GetCommentsCount {
        title: &'a str,
    } => "SELECT COUNT(*) as count FROM comments WHERE title = ?",
    DeleteComment {
        id: i32,
        user_id: i32,
    } => "DELETE FROM comments WHERE id = ? AND user_id = ?",
    UpdateComment {
        content: &'a str,
        score: Option<i32>,
        updated_at: i64,
        user_id: i32,
        title: &'a str,
    } => "UPDATE comments SET content = ?, score = ?, updated_at = ? WHERE user_id = ? AND title = ? RETURNING id, user_id as userId, title, content, score, created_at as createdAt, updated_at as updatedAt",
}
