use crate::model::UserStatus;

crate::define_model!(User, UserField, UserUpdate {
    id: i32 [pk],
    email: String,
    username: String,
    avatar_url: Option<String>,
    #[serde(skip_serializing)]
    password_hash: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    github_id: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    telegram_id: Option<String>,
    created_at: i64,
});

crate::define_model!(
    #[allow(dead_code)]
    Session,
    SessionField,
    SessionUpdate {
        id: i32[pk],
        user_id: i32,
        token: String,
        expires_at: i64,
    }
);

crate::define_model!(
    Passkey,
    PasskeyField,
    PasskeyUpdate {
        user_id: i32[pk],
        cred_id: String[pk],
        passkey_json: String,
        name: String,
        created_at: i64,
        last_used_at: i64,
        counter: i64,
    }
);

crate::define_model!(
    PasskeyState,
    PasskeyStateField,
    PasskeyStateUpdate {
        id: String[pk],
        state_json: String,
        expires_at: i64,
    }
);

crate::define_model!(UserItem, UserItemField, UserItemUpdate {
    user_id: i32 [pk],
    title: String [pk],
    status: UserStatus,
    score: Option<i32>,
    updated_at: i64,
    begin_at: Option<i64>,
});

crate::define_model!(
    #[serde(rename_all = "camelCase")]
    UserItemSummary,
    UserItemSummaryField,
    UserItemSummaryUpdate {
        status: UserStatus,
        score: Option<i32>,
    }
);

crate::define_model!(
    SchemaVersion,
    SchemaVersionField,
    SchemaVersionUpdate {
        version: Option<i32>,
    }
);
