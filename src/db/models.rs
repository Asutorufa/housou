use crate::model::UserStatus;
use serde_derive::{Deserialize, Serialize};

macro_rules! is_pk_helper {
    (@pk) => {
        true
    };
}

macro_rules! define_model {
    ($(#[$struct_meta:meta])* $name:ident, $enum_name:ident, $update_enum:ident {
        $( $(#[$field_meta:meta])* $field:ident : $ftype:ty $( [ $mode:ident ] )? ),* $(,)?
    }) => {
        #[derive(Debug, Serialize, Deserialize, Clone)]
        $(#[$struct_meta])*
        pub struct $name {
            $( $(#[$field_meta])* pub $field : $ftype ),*
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[allow(non_camel_case_types, dead_code)]
        pub enum $enum_name {
            $( $field ),*
        }

        impl $enum_name {
            #[allow(dead_code)]
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$field => stringify!($field) ),*
                }
            }
        }

        #[allow(non_camel_case_types, dead_code)]
        #[derive(Debug, Clone)]
        pub enum $update_enum {
             $( $field($ftype) ),*
        }

        impl crate::db::sql::FieldUpdate for $update_enum {
            fn field(&self) -> &'static str {
                match self {
                    $( Self::$field(_) => stringify!($field) ),*
                }
            }
            fn into_value(&self) -> crate::db::sql::DatabaseValue {
                match self {
                    $( Self::$field(v) => v.clone().into() ),*
                }
            }
        }

        impl crate::db::sql::FieldMeta for $update_enum {
            fn is_primary_key(&self) -> bool {
                match self {
                    $( Self::$field(_) => {
                        false $( || is_pk_helper!(@$mode) )?
                    } ),*
                }
            }
        }

        impl crate::db::sql::ToParams for $update_enum {
            fn add_params(&self, params: &mut Vec<crate::db::sql::DatabaseValue>) {
                match self {
                    $( Self::$field(v) => params.push(v.clone().into()) ),*
                }
            }
        }

        impl crate::db::sql::ToParams for Vec<$update_enum> {
            fn add_params(&self, params: &mut Vec<crate::db::sql::DatabaseValue>) {
                for u in self {
                    u.add_params(params);
                }
            }
        }
    };
}

define_model!(User, UserField, UserUpdate {
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

define_model!(
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

define_model!(
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

define_model!(
    PasskeyState,
    PasskeyStateField,
    PasskeyStateUpdate {
        id: String[pk],
        state_json: String,
        expires_at: i64,
    }
);

define_model!(UserItem, UserItemField, UserItemUpdate {
    user_id: i32 [pk],
    title: String [pk],
    status: UserStatus,
    score: Option<i32>,
    updated_at: i64,
    begin_at: Option<i64>,
});

define_model!(
    #[serde(rename_all = "camelCase")]
    UserItemSummary,
    UserItemSummaryField,
    UserItemSummaryUpdate {
        status: UserStatus,
        score: Option<i32>,
    }
);

define_model!(
    SchemaVersion,
    SchemaVersionField,
    SchemaVersionUpdate {
        version: Option<i32>,
    }
);
