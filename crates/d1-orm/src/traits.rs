use crate::types::DatabaseValue;
use crate::error::Error;
use std::borrow::Cow;

pub trait SqlBackend {
    type Param;
    fn convert(value: DatabaseValue) -> Self::Param;
}

pub trait IntoResultCow {
    fn into_result_cow(self) -> Result<Cow<'static, str>, Error>;
}

impl IntoResultCow for &'static str {
    fn into_result_cow(self) -> Result<Cow<'static, str>, Error> {
        Ok(Cow::Borrowed(self))
    }
}

impl IntoResultCow for String {
    fn into_result_cow(self) -> Result<Cow<'static, str>, Error> {
        Ok(Cow::Owned(self))
    }
}

impl IntoResultCow for Cow<'static, str> {
    fn into_result_cow(self) -> Result<Cow<'static, str>, Error> {
        Ok(self)
    }
}

impl IntoResultCow for Result<Cow<'static, str>, Error> {
    fn into_result_cow(self) -> Result<Cow<'static, str>, Error> {
        self
    }
}

pub trait Query {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error>;
}

pub trait QueryExt: Query {
    fn build_params<B: SqlBackend>(
        &self,
    ) -> Result<(Cow<'static, str>, Vec<B::Param>), Error> {
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

#[async_trait::async_trait(?Send)]
pub trait DatabaseExecutor {
    async fn query_all<T, Q>(&self, sql: Q) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait;

    async fn query_first<T, Q>(&self, sql: Q) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait;

    async fn execute<Q>(&self, sql: Q) -> Result<(), Error>
    where
        Q: Query + 'async_trait;

    async fn execute_batch<Q>(&self, sqls: Vec<Q>) -> Result<(), Error>
    where
        Q: Query + 'async_trait;
}
