#[derive(Clone, Debug)]
pub enum DatabaseValue {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Real(f64),
    Text(String),
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

impl From<Vec<u8>> for DatabaseValue {
    fn from(v: Vec<u8>) -> Self {
        DatabaseValue::Blob(v)
    }
}
