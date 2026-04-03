use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Subsection {
    pub parent_section_id: String,
    pub child_section_id: String,
}
