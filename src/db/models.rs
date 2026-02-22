use crate::model::UserStatus;
use serde_derive::{Deserialize, Serialize};

macro_rules! define_model {
    ($(#[$struct_meta:meta])* $name:ident, $enum_name:ident, $update_enum:ident {
        $( $(#[$field_meta:meta])* $field:ident : $ftype:ty ),* $(,)?
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
            fn into_value(self) -> crate::db::sql::DatabaseValue {
                use crate::db::sql::ToDatabaseValue;
                match self {
                    $( Self::$field(v) => v.to_value() ),*
                }
            }
        }

        impl crate::db::sql::CollectParams for $update_enum {
            fn collect_params(self, params: &mut Vec<crate::db::sql::DatabaseValue>) {
                use crate::db::sql::FieldUpdate;
                params.push(self.into_value());
            }
        }

        impl crate::db::sql::CollectParams for Vec<$update_enum> {
            fn collect_params(self, params: &mut Vec<crate::db::sql::DatabaseValue>) {
                use crate::db::sql::FieldUpdate;
                for u in self {
                    params.push(u.into_value());
                }
            }
        }
    };
}

define_model!(User, UserField, UserUpdate {
    id: i32,
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
        id: i32,
        user_id: i32,
        token: String,
        expires_at: i64,
    }
);

define_model!(UserItem, UserItemField, UserItemUpdate {
    user_id: i32,
    title: String,
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
