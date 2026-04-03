use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct SectionIdentity {
    pub section_id: String,
    pub document_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Section {
    pub id: i64,
    pub section_id: String,
    pub layout_order: f64,
    pub name: String,
    pub revision_date: NaiveDateTime,
    pub content: String,
}
