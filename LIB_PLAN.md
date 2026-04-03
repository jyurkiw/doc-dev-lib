# doc-dev-lib: Rust CRUD Crate Plan

## Purpose

This crate wraps all CRUD operations over the `doc-dev-lib` SQLite schema. It is designed
to be consumed as a Cargo dependency by two downstream projects:

- An **MCP server** (async, stdio transport)
- A **graphical front-end** (calls async ops via `spawn_blocking` or directly from async context)

---

## Technology Choices

| Concern | Choice | Rationale |
|---------|--------|-----------|
| SQLite driver | `sqlx` (SQLite feature) | Async-native; strong type mapping; works with tokio |
| Async runtime | `tokio` (downstream) | Library itself has no direct tokio dep |
| Error type | `thiserror` | Lightweight, composable |
| UUID | `uuid` crate (`v4`) | Section identities use UUID strings |
| Datetime | `chrono` + sqlx chrono feature | Maps to DATETIME columns |
| Serialization | `serde` | All models derive Serialize/Deserialize for MCP JSON |

---

## Crate Layout

```
src/
├── lib.rs               — public re-exports
├── db.rs                — Db struct, pool init, schema bootstrap
├── error.rs             — DbError enum
├── models/
│   ├── mod.rs
│   ├── author.rs        — Author
│   ├── document.rs      — Document
│   ├── section.rs       — SectionIdentity + Section
│   ├── subsection.rs    — Subsection
│   └── note.rs          — Note
└── ops/
    ├── mod.rs
    ├── authors.rs        — Author CRUD
    ├── documents.rs      — Document CRUD
    ├── sections.rs       — Section + SectionIdentity ops
    ├── subsections.rs    — Hierarchy ops
    └── notes.rs          — Note ops
```

---

## Models

All structs derive `Clone`, `Debug`, `serde::Serialize`, `serde::Deserialize`, `sqlx::FromRow`.

| Struct | Table | Key fields |
|--------|-------|------------|
| `Author` | Authors | id, name, description? |
| `Document` | Documents | id, name, description? |
| `SectionIdentity` | SectionIdentities | section_id (UUID), document_id |
| `Section` | Sections | id, section_id, layout_order, name, revision_date, content |
| `Subsection` | Subsections | parent_section_id, child_section_id |
| `Note` | Notes | id, author_id, section_id, creation_id, resolution_id?, note_date, content |

---

## `Db` struct (`db.rs`)

```rust
pub struct Db { pool: SqlitePool }

impl Db {
    pub async fn open(path: &str) -> Result<Self, DbError>
    pub async fn open_in_memory() -> Result<Self, DbError>
    pub async fn initialize(&self) -> Result<(), DbError>  // executes schema.sql
}
```

Downstream usage:
```rust
let db = Db::open("my.sqlite").await?;
db.initialize().await?;
```

---

## Operations

### Authors (`ops::authors`)
- `create(db, name, description?) → Author`
- `get(db, id) → Option<Author>`
- `list(db) → Vec<Author>`
- `update(db, id, name, description?) → Author`
- `delete(db, id) → ()`

### Documents (`ops::documents`)
- `create(db, name, description?) → Document`
- `get(db, id) → Option<Document>`
- `list(db) → Vec<Document>`
- `update(db, id, name, description?) → Document`
- `delete(db, id) → ()`

### Sections (`ops::sections`)

New section creation is **transactional** — wraps INSERT into SectionIdentities,
INSERT into Sections, and optional INSERT into Subsections:

```rust
pub async fn create_section(
    db: &Db,
    document_id: i64,
    parent_section_id: Option<&str>,
    layout_order: f64,
    name: &str,
    content: &str,
) -> Result<(SectionIdentity, Section), DbError>
```

Revisions are **append-only** — never UPDATE:
```rust
pub async fn revise_section(
    db: &Db,
    section_id: &str,
    name: &str,
    layout_order: f64,
    content: &str,
) -> Result<Section, DbError>
```

Queries:
- `get_current(db, section_id) → Option<Section>` — uses `idx_sections_current`
- `get_history(db, section_id) → Vec<Section>` — all revisions, newest first
- `list_current_for_document(db, document_id) → Vec<Section>`

### Subsections (`ops::subsections`)
- `add_child(db, parent_id, child_id) → ()`
- `remove_child(db, parent_id, child_id) → ()`
- `get_children(db, parent_id) → Vec<Section>` — joins current version, ordered by `layout_order`
- `get_parent(db, child_id) → Option<Section>`
- `get_top_level(db, document_id) → Vec<Section>` — sections with no parent row in Subsections

### Notes (`ops::notes`)
- `create(db, author_id, section_id, creation_id, content) → Note`
- `resolve(db, note_id, resolution_id) → Note`
- `list_for_section(db, section_id) → Vec<Note>` — uses `idx_notes_section`
- `list_unresolved(db) → Vec<Note>` — uses `idx_notes_unresolved` partial index

---

## Error Type (`error.rs`)

```rust
#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("sqlx error: {0}")]  Sqlx(#[from] sqlx::Error),
    #[error("not found")]        NotFound,
    #[error("invalid uuid")]     InvalidUuid,
}
```

---

## Schema Bootstrap

`Db::initialize()` executes `schema.sql` against the pool using sqlx. Idempotent via
`CREATE TABLE IF NOT EXISTS` could be added later; for now the function is safe to call
on a fresh database.

---

## Verification

```bash
cargo build     # must compile cleanly, no warnings
cargo check     # fast type-check
```

Smoke test (in `#[cfg(test)]` inside `db.rs`):
1. `Db::open_in_memory().await?.initialize().await?`
2. Create Author, Document, Section (top-level), child Section
3. Create Note, then resolve it
4. Assert `ops::notes::list_unresolved(&db).await?.len() == 0`
