/// Build update-set pairs for `update_by_*` helpers.
///
/// Example:
/// `let sets = d1_orm::d1_sets! { "name" => JsValue::from_str("Alice") };`
#[macro_export]
macro_rules! d1_sets {
    ($($column:literal => $value:expr),+ $(,)?) => {{
        ::std::vec![
            $(
                ($column, ($value).into())
            ),+
        ]
    }};
}
