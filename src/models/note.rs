use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: i64,
    pub author_id: i64,
    pub section_id: String,
    pub creation_id: i64,
    pub resolution_id: Option<i64>,
    pub note_date: NaiveDateTime,
    pub content: String,
}
