# d1-orm-derive

Procedural macros for `d1-orm`.

Most users should depend on `d1-orm` directly, which re-exports `#[derive(Model)]`.

## Installation

```toml
[dependencies]
d1-orm = "0.1"
```

## Derive Macro

`#[derive(Model)]` generates:

- `impl d1_orm::Model`
- schema helpers: `schema_at(version)`, `indexes_at(version)`
- convenience methods: `find_by_pk`, `insert`, `insert_returning`, `update`

## Complete Derive Example

```rust
use d1_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[d1(table_name = "episodes", constraint = "UNIQUE(show_id, number)")]
#[d1(since = 1)]
struct Episode {
    #[d1(primary_key, auto_increment)]
    id: i64,
    #[d1(not_null, index)]
    show_id: i64,
    #[d1(not_null)]
    number: i32,
    #[d1(not_null)]
    title: String,
    #[d1(since = 2)]
    summary: Option<String>,
}

fn demo() {
    let v1_table = Episode::schema_at(1).expect("table exists in v1");
    let v2_table = Episode::schema_at(2).expect("table exists in v2");
    let v2_indexes = Episode::indexes_at(2);
    let v1_setup_sql = d1_orm::model_setup_sql::<Episode>(1);
    let v1_to_v2_sql = d1_orm::model_diff_sql::<Episode>(1, 2);

    assert!(v1_table.columns.len() < v2_table.columns.len());
    assert!(!v2_indexes.is_empty());
    assert!(!v1_setup_sql.is_empty());
    assert!(!v1_to_v2_sql.is_empty());
}
```

## Supported Attributes

Container attributes:

- `#[d1(table_name = "...")]`
- `#[d1(constraint = "...")]` (repeatable)
- `#[d1(since = N)]`
- `#[d1(until = N)]`

Field attributes:

- `#[d1(primary_key)]`
- `#[d1(auto_increment)]`
- `#[d1(not_null)]`
- `#[d1(unique)]`
- `#[d1(index)]`
- `#[d1(unique_index)]`
- `#[d1(select_by)]` (generate both `get_*_by_*` and short `get_by_*` style methods, plus `list/update/delete`)
- `#[d1(integer)]` (force SQLite integer mapping)
- `#[d1(since = N)]`
- `#[d1(until = N)]`
