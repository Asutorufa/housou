use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit};

#[proc_macro_derive(Model, attributes(d1))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut table_name = name.to_string().to_lowercase();
    let mut primary_key = "id".to_string();

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
                } else {
                    Ok(())
                }
            });
        }
    }

    let mut columns_setup = Vec::new();
    let mut indexes_setup = Vec::new();

    if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            for field in fields.named {
                let ident = field.ident.unwrap();
                let field_name = ident.to_string();
                let ty = field.ty;

                let mut col_type = quote! { d1_orm::ColumnType::Text }; // Default
                let mut constraints = Vec::new();

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
                            } else if meta.path.is_ident("auto_increment") {
                                constraints.push(quote! { d1_orm::Constraint::AutoIncrement });
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
            }
        }
    }

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
                    constraints: vec![],
                }
            }

            pub fn indexes() -> Vec<d1_orm::Index> {
                vec![
                    #(#indexes_setup),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}
