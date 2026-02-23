#[macro_export]
macro_rules! migration_info_helper {
    (@table($name:expr)) => {
        $crate::MigrationInfo::Table($name)
    };
    (@index($name:expr)) => {
        $crate::MigrationInfo::Index($name)
    };
    (@column($t:expr, $c:expr)) => {
        $crate::MigrationInfo::Column {
            table: $t,
            column: $c,
        }
    };
    (@adhoc($info:expr)) => {
        *$info
    };
}

#[macro_export]
macro_rules! sql_params {
    ($p:ident sql) => {};
    ($p:ident info) => {};
    ($p:ident $field:ident [skip_primary_key]) => {
        for u in $field
            .iter()
            .filter(|u| !$crate::FieldMeta::is_primary_key(*u))
        {
            $p.push($crate::FieldUpdate::to_value(u));
        }
    };
    ($p:ident $field:ident) => {
        $crate::ToParams::add_params($field, &mut $p);
    };
}

#[macro_export]
macro_rules! define_sql {
    (
        $enum_name:ident
        $(
            $( @$mtype:ident ( $($margs:tt)* ) )?
            $name:ident $( { $($field:ident : $ftype:ty $( [ $mode:ident ] )? ),* $(,)? } )? => $sql:expr
        ),* $(,)?
    ) => {
        #[derive(Clone, Debug)]
        pub enum $enum_name<'a> {
            $(
                #[allow(dead_code)]
                $name $( { $($field : $ftype),* } )?,
            )*
        }

        impl<'a> $crate::Query for $enum_name<'a> {
            fn build(&self) -> Result<(::std::borrow::Cow<'static, str>, Vec<$crate::DatabaseValue>), $crate::Error> {
                match self {
                    $(
                        $enum_name::$name $( { $($field,)* } )? => {
                            $( $(let _ = &$field;)* )?
                            let sql: Result<::std::borrow::Cow<'static, str>, _> =
                                $crate::IntoResultCow::into_result_cow($sql);

                            sql.map(|sql| {
                                #[allow(unused_mut)]
                                let mut v = Vec::new();
                                $(
                                    $(
                                        $crate::sql_params!(v $field $( [$mode] )? );
                                    )*
                                )?
                                (sql, v)
                            })
                        },
                    )*
                }
            }
        }

        impl<'a> $crate::MigrationMeta for $enum_name<'a> {
            fn migration_info(&self) -> Option<$crate::MigrationInfo> {
                match self {
                    $(
                        $enum_name::$name $( { $($field,)* } )? => {
                            $( $(let _ = $field;)* )?
                            None $( .or(Some($crate::migration_info_helper!(@$mtype($($margs)*)))) )?
                        }
                    ),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! is_pk_helper {
    (@pk) => {
        true
    };
}

#[macro_export]
macro_rules! define_model {
    ($(#[$struct_meta:meta])* $name:ident, $enum_name:ident, $update_enum:ident {
        $( $(#[$field_meta:meta])* $field:ident : $ftype:ty $( [ $mode:ident ] )? ),* $(,)?
    }) => {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
        $(#[$struct_meta])*
        pub struct $name {
            $( $(#[$field_meta])* pub $field : $ftype ),*
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[allow(non_camel_case_types, dead_code)]
        pub enum $enum_name {
            $( $field ),*
        }

        impl $enum_name {
            #[allow(dead_code)]
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$field => stringify!($field) ),*
                }
            }
        }

        #[allow(non_camel_case_types, dead_code)]
        #[derive(Debug, Clone)]
        pub enum $update_enum {
             $( $field($ftype) ),*
        }

        impl $crate::FieldUpdate for $update_enum {
            fn field(&self) -> &'static str {
                match self {
                    $( Self::$field(_) => stringify!($field) ),*
                }
            }
            fn to_value(&self) -> $crate::DatabaseValue {
                match self {
                    $( Self::$field(v) => v.clone().into() ),*
                }
            }
        }

        impl $crate::FieldMeta for $update_enum {
            fn is_primary_key(&self) -> bool {
                match self {
                    $( Self::$field(_) => {
                        false $( || $crate::is_pk_helper!(@$mode) )?
                    } ),*
                }
            }
        }

        impl $crate::ToParams for $update_enum {
            fn add_params(&self, params: &mut Vec<$crate::DatabaseValue>) {
                match self {
                    $( Self::$field(v) => params.push(v.clone().into()) ),*
                }
            }
        }
    };
}
