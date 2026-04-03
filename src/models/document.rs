use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}
