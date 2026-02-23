use crate::error::Error;
use crate::traits::{FieldMeta, FieldUpdate};
use std::borrow::Cow;
use std::fmt::Write;

pub fn build_update_sql<T: FieldMeta + FieldUpdate>(
    table: &str,
    key_field: &str,
    updates: &[T],
) -> Result<Cow<'static, str>, Error> {
    let valid = updates.iter().filter(|u| !u.is_primary_key());
    let count = valid.clone().count();
    if count == 0 {
        return Err(Error::Build("Empty update fields".to_string()));
    }

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
) -> Result<Cow<'static, str>, Error> {
    let valid = updates.iter().filter(|u| !u.is_primary_key());
    let count = valid.clone().count();
    if count == 0 {
        return Err(Error::Build("Empty update fields".to_string()));
    }

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
