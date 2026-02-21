/// Execute multiple bindable statements in a single D1 batch.
///
/// Each entry must implement `d1_orm::Bindable`.
#[macro_export]
macro_rules! d1_exec_batch {
    ($db:expr, [$($stmt:expr),+ $(,)?]) => {{
        async {
            let mut __stmts = ::std::vec::Vec::new();
            $(
                let (__sql, __bind) = $stmt.to_sql();
                __stmts.push($db.prepare(&__sql).bind(&__bind)?);
            )+
            $db.batch(__stmts).await?;
            Ok::<(), $crate::Error>(())
        }
        .await
    }};
}
