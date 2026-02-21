/// Execute a raw SQL query with bindings and deserialize all rows as `Vec<T>`.
///
/// This macro is intended for complex predicates that are awkward to express
/// with the query builder.
#[macro_export]
macro_rules! d1_query_all {
    ($db:expr, $ty:ty, $sql:expr, [$($bind:expr),* $(,)?]) => {{
        let __bindings: ::std::vec::Vec<$crate::JsValue> = ::std::vec![$($bind),*];
        let __result = $db.prepare($sql).bind(&__bindings)?.all().await?;
        let __rows: ::std::vec::Vec<$ty> = __result.results()?;
        Ok::<::std::vec::Vec<$ty>, $crate::Error>(__rows)
    }};
}

/// Execute a raw SQL query with bindings and deserialize the first row as `Option<T>`.
#[macro_export]
macro_rules! d1_query_first {
    ($db:expr, $ty:ty, $sql:expr, [$($bind:expr),* $(,)?]) => {{
        let __bindings: ::std::vec::Vec<$crate::JsValue> = ::std::vec![$($bind),*];
        let __row = $db.prepare($sql).bind(&__bindings)?.first::<$ty>(None).await?;
        Ok::<::std::option::Option<$ty>, $crate::Error>(__row)
    }};
}
