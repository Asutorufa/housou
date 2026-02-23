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
    let valid_count = valid.clone().count();
    let pk_count = config.primary_keys.len();

    // Must have at least one field to insert (PK or update field)
    if pk_count == 0 && valid_count == 0 {
        return Err(Error::Build("Empty fields for upsert".to_string()));
    }

    use std::fmt::Write;
    let mut sql = String::with_capacity(128 + config.table.len() + (pk_count + valid_count) * 40);
    write!(sql, "INSERT INTO {} (", config.table).unwrap();

    // Columns: PKs then Update fields
    let mut first = true;
    for pk in config.primary_keys.iter() {
        if !first {
            sql.push_str(", ");
        }
        sql.push_str(pk);
        first = false;
    }
    for u in valid.clone() {
        if !first {
            sql.push_str(", ");
        }
        write!(sql, "{}", u.field()).unwrap();
        first = false;
    }

    sql.push_str(") VALUES (");

    // Placeholders
    first = true;
    for _ in 0..pk_count {
        if !first {
            sql.push_str(", ");
        }
        sql.push('?');
        first = false;
    }
    for _ in 0..valid_count {
        if !first {
            sql.push_str(", ");
        }
        sql.push('?');
        first = false;
    }
    sql.push(')');

    // ON CONFLICT clause
    if pk_count > 0 {
        sql.push_str(" ON CONFLICT(");
        first = true;
        for pk in config.primary_keys.iter() {
            if !first {
                sql.push_str(", ");
            }
            sql.push_str(pk);
            first = false;
        }
        if valid_count > 0 {
            sql.push_str(") DO UPDATE SET ");

            first = true;
            for u in valid {
                if !first {
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
                first = false;
            }
        } else {
            sql.push_str(") DO NOTHING");
        }
    }

    Ok(Cow::Owned(sql))
}
