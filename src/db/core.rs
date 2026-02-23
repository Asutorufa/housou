use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum DatabaseValue {
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

impl From<&DatabaseValue> for DatabaseValue {
    fn from(v: &DatabaseValue) -> Self {
        v.clone()
    }
}

#[derive(Debug, Clone)]
pub enum BuilderError {
    EmptyUpdate,
    #[allow(dead_code)]
    MissingPrimaryKey,
    #[allow(dead_code)]
    Other(String),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyUpdate => write!(f, "Empty update fields"),
            Self::MissingPrimaryKey => write!(f, "Missing primary key"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}
impl std::error::Error for BuilderError {}

pub trait IntoResultCow {
    fn into_result_cow(self) -> Result<Cow<'static, str>, BuilderError>;
}

impl IntoResultCow for &'static str {
    fn into_result_cow(self) -> Result<Cow<'static, str>, BuilderError> {
        Ok(Cow::Borrowed(self))
    }
}

impl IntoResultCow for String {
    fn into_result_cow(self) -> Result<Cow<'static, str>, BuilderError> {
        Ok(Cow::Owned(self))
    }
}

impl IntoResultCow for Cow<'static, str> {
    fn into_result_cow(self) -> Result<Cow<'static, str>, BuilderError> {
        Ok(self)
    }
}

impl IntoResultCow for Result<Cow<'static, str>, BuilderError> {
    fn into_result_cow(self) -> Result<Cow<'static, str>, BuilderError> {
        self
    }
}

pub trait SqlBackend {
    type Param;
    fn convert(value: DatabaseValue) -> Self::Param;
}

pub trait Query {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), BuilderError>;
}

pub trait QueryExt: Query {
    fn build_params<B: SqlBackend>(
        &self,
    ) -> Result<(Cow<'static, str>, Vec<B::Param>), BuilderError> {
        self.build()
            .map(|(sql, values)| (sql, values.into_iter().map(B::convert).collect()))
    }
}

impl<T: Query + ?Sized> QueryExt for T {}

pub trait FieldUpdate {
    fn field(&self) -> &'static str;
    fn to_value(&self) -> DatabaseValue;
}

pub trait FieldMeta {
    fn is_primary_key(&self) -> bool;
}

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

pub trait MigrationMeta {
    fn migration_info(&self) -> Option<MigrationInfo>;
}

#[macro_export]
macro_rules! migration_info_helper {
    (@table($name:expr)) => {
        $crate::db::core::MigrationInfo::Table($name)
    };
    (@index($name:expr)) => {
        $crate::db::core::MigrationInfo::Index($name)
    };
    (@column($t:expr, $c:expr)) => {
        $crate::db::core::MigrationInfo::Column {
            table: $t,
            column: $c,
        }
    };
    (@adhoc($info:expr)) => {
        *$info
    };
}

pub fn build_update_sql<T: FieldMeta + FieldUpdate>(
    table: &str,
    key_field: &str,
    updates: &[T],
) -> Result<Cow<'static, str>, BuilderError> {
    let valid = updates.iter().filter(|u| !u.is_primary_key());
    let count = valid.clone().count();
    if count == 0 {
        return Err(BuilderError::EmptyUpdate);
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

    Ok(Cow::Owned(sql))
}

pub type ConflictResolution<'a> = dyn Fn(&str) -> Option<&'static str> + 'a;

pub struct UpsertConfig<'a> {
    pub table: &'a str,
    pub primary_keys: &'a [&'a str],
    pub custom_conflict_resolution: Option<&'a ConflictResolution<'a>>,
}

pub fn build_upsert_sql<T: FieldMeta + FieldUpdate>(
    config: &UpsertConfig,
    updates: &[T],
) -> Result<Cow<'static, str>, BuilderError> {
    let valid = updates.iter().filter(|u| !u.is_primary_key());
    let count = valid.clone().count();
    if count == 0 {
        return Err(BuilderError::EmptyUpdate);
    }

    use std::fmt::Write;
    let mut sql = String::with_capacity(128 + config.table.len() + count * 40);
    write!(sql, "INSERT INTO {} (", config.table).unwrap();

    for (i, pk) in config.primary_keys.iter().enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        sql.push_str(pk);
    }
    for u in valid.clone() {
        write!(sql, ", {}", u.field()).unwrap();
    }

    sql.push_str(") VALUES (");
    for i in 0..config.primary_keys.len() {
        if i > 0 {
            sql.push_str(", ?");
        } else {
            sql.push('?');
        }
    }
    for _ in 0..count {
        sql.push_str(", ?");
    }

    sql.push_str(") ON CONFLICT(");
    for (i, pk) in config.primary_keys.iter().enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        sql.push_str(pk);
    }
    sql.push_str(") DO UPDATE SET ");

    for (i, u) in valid.enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        let f = u.field();

        let mut resolved = false;
        if let Some(custom_sql) = config.custom_conflict_resolution.and_then(|cf| cf(f)) {
            sql.push_str(custom_sql);
            resolved = true;
        }
        if !resolved {
            write!(sql, "{} = excluded.{}", f, f).unwrap();
        }
    }

    Ok(Cow::Owned(sql))
}

#[macro_export]
macro_rules! sql_params {
    ($p:ident sql) => {};
    ($p:ident info) => {};
    ($p:ident $field:ident [skip_primary_key]) => {
        for u in $field
            .iter()
            .filter(|u| !$crate::db::core::FieldMeta::is_primary_key(*u))
        {
            $p.push($crate::db::core::FieldUpdate::to_value(u));
        }
    };
    ($p:ident $field:ident) => {
        $crate::db::core::ToParams::add_params($field, &mut $p);
    };
}

#[macro_export]
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

        impl<'a> $crate::db::core::Query for $enum_name<'a> {
            fn build(&self) -> Result<(::std::borrow::Cow<'static, str>, Vec<$crate::db::core::DatabaseValue>), $crate::db::core::BuilderError> {
                match self {
                    $(
                        $enum_name::$name $( { $($field,)* } )? => {
                            $( $(let _ = &$field;)* )?
                            let sql: Result<::std::borrow::Cow<'static, str>, _> =
                                $crate::db::core::IntoResultCow::into_result_cow($sql);

                            sql.map(|sql| {
                                #[allow(unused_mut)]
                                let mut v = Vec::new();
                                $(
                                    $(
                                        $crate::sql_params!(v $field $( [$mode] )? );
                                    )*
                                )?
                                (sql, v)
                            })
                        },
                    )*
                }
            }
        }

        impl<'a> $crate::db::core::MigrationMeta for $enum_name<'a> {
            fn migration_info(&self) -> Option<$crate::db::core::MigrationInfo> {
                match self {
                    $(
                        $enum_name::$name $( { $($field,)* } )? => {
                            $( $(let _ = $field;)* )?
                            None $( .or(Some($crate::migration_info_helper!(@$mtype($($margs)*)))) )?
                        }
                    ),*
                }
            }
        }
    };
}

#[async_trait::async_trait(?Send)]
pub trait DatabaseExecutor {
    async fn query_all<T, Q>(&self, sql: Q) -> worker::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait;

    async fn query_first<T, Q>(&self, sql: Q) -> worker::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait;

    async fn execute<Q>(&self, sql: Q) -> worker::Result<()>
    where
        Q: Query + 'async_trait;

    async fn execute_batch<Q>(&self, sqls: Vec<Q>) -> worker::Result<()>
    where
        Q: Query + 'async_trait;
}

#[macro_export]
macro_rules! is_pk_helper {
    (@pk) => {
        true
    };
}

#[macro_export]
macro_rules! define_model {
    ($(#[$struct_meta:meta])* $name:ident, $enum_name:ident, $update_enum:ident {
        $( $(#[$field_meta:meta])* $field:ident : $ftype:ty $( [ $mode:ident ] )? ),* $(,)?
    }) => {
        #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, Clone)]
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

        impl $crate::db::core::FieldUpdate for $update_enum {
            fn field(&self) -> &'static str {
                match self {
                    $( Self::$field(_) => stringify!($field) ),*
                }
            }
            fn to_value(&self) -> $crate::db::core::DatabaseValue {
                match self {
                    $( Self::$field(v) => v.clone().into() ),*
                }
            }
        }

        impl $crate::db::core::FieldMeta for $update_enum {
            fn is_primary_key(&self) -> bool {
                match self {
                    $( Self::$field(_) => {
                        false $( || $crate::is_pk_helper!(@$mode) )?
                    } ),*
                }
            }
        }

        impl $crate::db::core::ToParams for $update_enum {
            fn add_params(&self, params: &mut Vec<$crate::db::core::DatabaseValue>) {
                match self {
                    $( Self::$field(v) => params.push(v.clone().into()) ),*
                }
            }
        }

        impl $crate::db::core::ToParams for Vec<$update_enum> {
            fn add_params(&self, params: &mut Vec<$crate::db::core::DatabaseValue>) {
                for u in self {
                    $crate::db::core::ToParams::add_params(u, params);
                }
            }
        }
    };
}
