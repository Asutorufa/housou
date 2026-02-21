//! Procedural macros for `d1-orm`.
//!
//! This crate provides `#[derive(Model)]`, re-exported by `d1-orm`.
//! Most users should depend on `d1-orm` directly.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Lit};

fn to_snake_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for (idx, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// Derive `d1_orm::Model` and generate helper methods for schema and CRUD operations.
///
/// Supported container attributes:
/// - `#[d1(table_name = "...")]`
/// - `#[d1(constraint = "...")]` (repeatable)
/// - `#[d1(since = N)]`
/// - `#[d1(until = N)]`
///
/// Supported field attributes:
/// - `#[d1(primary_key)]`
/// - `#[d1(auto_increment)]`
/// - `#[d1(not_null)]`
/// - `#[d1(unique)]`
/// - `#[d1(index)]`
/// - `#[d1(unique_index)]`
/// - `#[d1(select_by)]` (generate `get_*_by_*`, `list_*_by_*`, `update_*_by_*`, `delete_*_by_*`)
/// - `#[d1(integer)]` (force SQLite integer mapping)
/// - `#[d1(since = N)]`
/// - `#[d1(until = N)]`
#[proc_macro_derive(Model, attributes(d1))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let model_snake = to_snake_case(&name.to_string());

    let mut table_name = name.to_string().to_lowercase();
    let mut primary_key = "id".to_string();
    let mut primary_key_ident: Option<Ident> = None;
    let mut table_constraints = Vec::<String>::new();
    let mut table_since: i32 = 1;
    let mut table_until: Option<i32> = None;
    let mut latest_version: i32 = 1;

    // Parse struct attributes for table name
    for attr in &input.attrs {
        if attr.path().is_ident("d1") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table_name") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit) = s {
                        table_name = lit.value();
                    }
                    Ok(())
                } else if meta.path.is_ident("constraint") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit) = s {
                        table_constraints.push(lit.value());
                    }
                    Ok(())
                } else if meta.path.is_ident("since") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Int(lit) = s {
                        if let Ok(v) = lit.base10_parse::<i32>() {
                            table_since = v;
                            latest_version = latest_version.max(v);
                        }
                    }
                    Ok(())
                } else if meta.path.is_ident("until") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Int(lit) = s {
                        if let Ok(v) = lit.base10_parse::<i32>() {
                            table_until = Some(v);
                            latest_version = latest_version.max(v);
                        }
                    }
                    Ok(())
                } else {
                    Ok(())
                }
            });
        }
    }

    let mut columns_at_setup = Vec::new();
    let mut indexes_at_setup = Vec::new();
    let mut insert_setters = Vec::new();
    let mut update_setters = Vec::new();
    let mut select_by_helpers = Vec::new();

    if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            for field in fields.named {
                let ident = field.ident.unwrap();
                let field_name = ident.to_string();
                let ty = field.ty;

                let mut col_type = quote! { d1_orm::ColumnType::Text }; // Default
                let mut constraints = Vec::new();
                let mut is_primary_key = false;
                let mut is_auto_increment = false;
                let mut has_index = false;
                let mut has_unique_index = false;
                let mut field_since: i32 = 1;
                let mut field_until: Option<i32> = None;
                let mut force_integer = false;
                let mut select_by = false;

                // Heuristic type mapping
                let type_str = quote!(#ty).to_string();
                if type_str.contains("i32")
                    || type_str.contains("i64")
                    || type_str.contains("u32")
                    || type_str.contains("u64")
                {
                    col_type = quote! { d1_orm::ColumnType::Integer };
                } else if type_str.contains("f32") || type_str.contains("f64") {
                    col_type = quote! { d1_orm::ColumnType::Real };
                } else if type_str.contains("bool") {
                    col_type = quote! { d1_orm::ColumnType::Boolean };
                } else if type_str.contains("Vec < u8 >") {
                    col_type = quote! { d1_orm::ColumnType::Blob };
                }

                // Parse field attributes
                for attr in &field.attrs {
                    if attr.path().is_ident("d1") {
                        let _ = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("primary_key") {
                                constraints.push(quote! { d1_orm::Constraint::PrimaryKey });
                                primary_key = field_name.clone();
                                primary_key_ident = Some(ident.clone());
                                is_primary_key = true;
                            } else if meta.path.is_ident("auto_increment") {
                                constraints.push(quote! { d1_orm::Constraint::AutoIncrement });
                                is_auto_increment = true;
                            } else if meta.path.is_ident("not_null") {
                                constraints.push(quote! { d1_orm::Constraint::NotNull });
                            } else if meta.path.is_ident("unique") {
                                constraints.push(quote! { d1_orm::Constraint::Unique });
                            } else if meta.path.is_ident("index") {
                                has_index = true;
                            } else if meta.path.is_ident("unique_index") {
                                has_unique_index = true;
                            } else if meta.path.is_ident("select_by") {
                                select_by = true;
                            } else if meta.path.is_ident("integer") {
                                force_integer = true;
                            } else if meta.path.is_ident("since") {
                                let value = meta.value()?;
                                let s: Lit = value.parse()?;
                                if let Lit::Int(lit) = s {
                                    if let Ok(v) = lit.base10_parse::<i32>() {
                                        field_since = v;
                                        latest_version = latest_version.max(v);
                                    }
                                }
                            } else if meta.path.is_ident("until") {
                                let value = meta.value()?;
                                let s: Lit = value.parse()?;
                                if let Lit::Int(lit) = s {
                                    if let Ok(v) = lit.base10_parse::<i32>() {
                                        field_until = Some(v);
                                        latest_version = latest_version.max(v);
                                    }
                                }
                            }
                            Ok(())
                        });
                    }
                }

                if force_integer {
                    col_type = quote! { d1_orm::ColumnType::Integer };
                }

                if primary_key == "id" && field_name == "id" && primary_key_ident.is_none() {
                    primary_key_ident = Some(ident.clone());
                    is_primary_key = true;
                }

                if !(is_primary_key && is_auto_increment) {
                    insert_setters.push(quote! {
                        insert = insert.set(#field_name, d1_orm::to_js_value(&self.#ident)?);
                    });
                }

                if !is_primary_key {
                    update_setters.push(quote! {
                        update = update.set(#field_name, d1_orm::to_js_value(&self.#ident)?);
                    });
                }

                let field_until_check = if let Some(until) = field_until {
                    quote! { version <= #until }
                } else {
                    quote! { true }
                };

                columns_at_setup.push(quote! {
                    if version >= #field_since && #field_until_check {
                        columns.push(d1_orm::Column {
                            name: #field_name.to_string(),
                            col_type: #col_type,
                            constraints: vec![#(#constraints),*],
                        });
                    }
                });

                if has_index {
                    indexes_at_setup.push(quote! {
                        if version >= #field_since && #field_until_check {
                            indexes.push(
                                d1_orm::Index::new(
                                    &format!("idx_{}_{}", #table_name, #field_name),
                                    #table_name,
                                )
                                .column(#field_name),
                            );
                        }
                    });
                }
                if has_unique_index {
                    indexes_at_setup.push(quote! {
                        if version >= #field_since && #field_until_check {
                            indexes.push(
                                d1_orm::Index::new(
                                    &format!("idx_{}_{}", #table_name, #field_name),
                                    #table_name,
                                )
                                .column(#field_name)
                                .unique(),
                            );
                        }
                    });
                }

                if select_by {
                    let get_fn = format_ident!("get_{}_by_{}", model_snake, field_name);
                    let list_fn = format_ident!("list_{}_by_{}", model_snake, field_name);
                    let update_fn = format_ident!("update_{}_by_{}", model_snake, field_name);
                    let delete_fn = format_ident!("delete_{}_by_{}", model_snake, field_name);
                    let get_by_fn = format_ident!("get_by_{}", field_name);
                    let list_by_fn = format_ident!("list_by_{}", field_name);
                    let update_by_fn = format_ident!("update_by_{}", field_name);
                    let delete_by_fn = format_ident!("delete_by_{}", field_name);
                    select_by_helpers.push(quote! {
                        pub async fn #get_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                        ) -> d1_orm::Result<Option<Self>> {
                            d1_orm::Repository::<Self>::new(db)
                                .find_one(d1_orm::Select::new(#table_name).where_eq(#field_name, value))
                                .await
                        }

                        pub async fn #list_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                        ) -> d1_orm::Result<Vec<Self>> {
                            d1_orm::Repository::<Self>::new(db)
                                .find_all(d1_orm::Select::new(#table_name).where_eq(#field_name, value))
                                .await
                        }

                        pub async fn #update_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                            set: &[(&str, d1_orm::JsValue)],
                        ) -> d1_orm::Result<d1_orm::D1Result> {
                            let mut update = d1_orm::Update::new(#table_name);
                            for (column, bind_value) in set {
                                update = update.set(column, bind_value.clone());
                            }
                            update = update.where_eq(#field_name, value);
                            d1_orm::Repository::<Self>::new(db).execute(update).await
                        }

                        pub async fn #delete_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                        ) -> d1_orm::Result<()> {
                            d1_orm::Repository::<Self>::new(db)
                                .execute(d1_orm::Delete::new(#table_name).where_eq(#field_name, value))
                                .await
                                .map(|_| ())
                        }

                        pub async fn #get_by_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                        ) -> d1_orm::Result<Option<Self>> {
                            Self::#get_fn(db, value).await
                        }

                        pub async fn #list_by_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                        ) -> d1_orm::Result<Vec<Self>> {
                            Self::#list_fn(db, value).await
                        }

                        pub async fn #update_by_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                            set: &[(&str, d1_orm::JsValue)],
                        ) -> d1_orm::Result<d1_orm::D1Result> {
                            Self::#update_fn(db, value, set).await
                        }

                        pub async fn #delete_by_fn(
                            db: &d1_orm::D1Database,
                            value: impl Into<d1_orm::JsValue> + Send,
                        ) -> d1_orm::Result<()> {
                            Self::#delete_fn(db, value).await
                        }
                    });
                }
            }
        }
    }

    let update_query_body = if let Some(pk_ident) = primary_key_ident {
        quote! {
            let mut update = d1_orm::Update::new(#table_name);
            #(#update_setters)*
            update = update.where_eq(#primary_key, d1_orm::to_js_value(&self.#pk_ident)?);
            Ok(update)
        }
    } else {
        quote! {
            Err(d1_orm::Error::Database(
                "model does not expose a primary key field for update".to_string(),
            ))
        }
    };
    let table_constraints_setup = table_constraints
        .iter()
        .map(|c| quote! { #c.to_string() })
        .collect::<Vec<_>>();
    let table_until_check = if let Some(until) = table_until {
        quote! { version <= #until }
    } else {
        quote! { true }
    };

    let expanded = quote! {
        impl d1_orm::Model for #name {
            const TABLE: &'static str = #table_name;
            fn primary_key() -> &'static str {
                #primary_key
            }
            fn schema_at(version: i32) -> Option<d1_orm::Table> {
                if version < #table_since || !(#table_until_check) {
                    return None;
                }
                let mut columns = Vec::new();
                #(#columns_at_setup)*
                Some(d1_orm::Table {
                    name: #table_name.to_string(),
                    columns,
                    constraints: vec![
                        #(#table_constraints_setup),*
                    ],
                })
            }

            fn indexes_at(version: i32) -> Vec<d1_orm::Index> {
                if version < #table_since || !(#table_until_check) {
                    return Vec::new();
                }
                let mut indexes = Vec::new();
                #(#indexes_at_setup)*
                indexes
            }

            fn latest_version() -> i32 {
                #latest_version
            }
        }

        impl #name {
            pub fn schema_at(version: i32) -> Option<d1_orm::Table> {
                <Self as d1_orm::Model>::schema_at(version)
            }

            pub fn indexes_at(version: i32) -> Vec<d1_orm::Index> {
                <Self as d1_orm::Model>::indexes_at(version)
            }

            pub async fn find_by_pk(
                db: &d1_orm::D1Database,
                id: impl Into<d1_orm::JsValue> + Send,
            ) -> d1_orm::Result<Option<Self>> {
                d1_orm::Repository::<Self>::new(db).find_by_id(id).await
            }

            fn insert_query(&self) -> d1_orm::Result<d1_orm::Insert> {
                let mut insert = d1_orm::Insert::new(#table_name);
                #(#insert_setters)*
                Ok(insert)
            }

            fn update_query(&self) -> d1_orm::Result<d1_orm::Update> {
                #update_query_body
            }

            pub async fn insert(&self, db: &d1_orm::D1Database) -> d1_orm::Result<d1_orm::D1Result> {
                d1_orm::Repository::<Self>::new(db)
                    .execute(self.insert_query()?)
                    .await
            }

            pub async fn create(&self, db: &d1_orm::D1Database) -> d1_orm::Result<d1_orm::D1Result> {
                self.insert(db).await
            }

            pub async fn insert_returning(&self, db: &d1_orm::D1Database) -> d1_orm::Result<Option<Self>> {
                let insert = self.insert_query()?.returning("*");
                d1_orm::Repository::<Self>::new(db).insert_one(insert).await
            }

            pub async fn create_returning(&self, db: &d1_orm::D1Database) -> d1_orm::Result<Option<Self>> {
                self.insert_returning(db).await
            }

            pub async fn update(&self, db: &d1_orm::D1Database) -> d1_orm::Result<d1_orm::D1Result> {
                d1_orm::Repository::<Self>::new(db)
                    .execute(self.update_query()?)
                    .await
            }

            #(#select_by_helpers)*
        }
    };

    TokenStream::from(expanded)
}
