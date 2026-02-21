use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Lit};

#[proc_macro_derive(Model, attributes(d1))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut table_name = name.to_string().to_lowercase();
    let mut primary_key = "id".to_string();
    let mut primary_key_ident: Option<Ident> = None;
    let mut table_constraints = Vec::<String>::new();

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
                } else {
                    Ok(())
                }
            });
        }
    }

    let mut columns_setup = Vec::new();
    let mut indexes_setup = Vec::new();
    let mut insert_setters = Vec::new();
    let mut update_setters = Vec::new();

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
                                let idx_name = format!("idx_{}_{}", table_name, field_name);
                                indexes_setup.push(quote! {
                                    d1_orm::Index::new(#idx_name, #table_name).column(#field_name)
                                });
                            }
                            Ok(())
                        });
                    }
                }

                columns_setup.push(quote! {
                    d1_orm::Column {
                        name: #field_name.to_string(),
                        col_type: #col_type,
                        constraints: vec![#(#constraints),*],
                    }
                });

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

    let expanded = quote! {
        impl d1_orm::Model for #name {
            const TABLE: &'static str = #table_name;
            fn primary_key() -> &'static str {
                #primary_key
            }
        }

        impl #name {
            pub fn schema() -> d1_orm::Table {
                d1_orm::Table {
                    name: #table_name.to_string(),
                    columns: vec![
                        #(#columns_setup),*
                    ],
                    constraints: vec![
                        #(#table_constraints_setup),*
                    ],
                }
            }

            pub fn indexes() -> Vec<d1_orm::Index> {
                vec![
                    #(#indexes_setup),*
                ]
            }

            pub fn repo<'a>(db: &'a d1_orm::D1Database) -> d1_orm::Repository<'a, Self> {
                d1_orm::Repository::new(db)
            }

            pub async fn find_by_pk(
                db: &d1_orm::D1Database,
                id: impl Into<d1_orm::JsValue> + Send,
            ) -> d1_orm::Result<Option<Self>> {
                Self::repo(db).find_by_id(id).await
            }

            pub async fn delete_by_pk(
                db: &d1_orm::D1Database,
                id: impl Into<d1_orm::JsValue> + Send,
            ) -> d1_orm::Result<()> {
                Self::repo(db).delete_by_id(id).await
            }

            pub fn insert_query(&self) -> d1_orm::Result<d1_orm::Insert> {
                let mut insert = d1_orm::Insert::new(#table_name);
                #(#insert_setters)*
                Ok(insert)
            }

            pub fn update_query(&self) -> d1_orm::Result<d1_orm::Update> {
                #update_query_body
            }

            pub async fn insert(&self, db: &d1_orm::D1Database) -> d1_orm::Result<d1_orm::D1Result> {
                Self::repo(db).execute(self.insert_query()?).await
            }

            pub async fn insert_returning(&self, db: &d1_orm::D1Database) -> d1_orm::Result<Option<Self>> {
                let insert = self.insert_query()?.returning("*");
                Self::repo(db).insert_one(insert).await
            }

            pub async fn update(&self, db: &d1_orm::D1Database) -> d1_orm::Result<d1_orm::D1Result> {
                Self::repo(db).execute(self.update_query()?).await
            }
        }
    };

    TokenStream::from(expanded)
}
