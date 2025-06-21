use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

mod csv;
mod db;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct RawIssueFeaturesRecord {
    pub issue_id: String,
    pub operation: Option<String>,
    pub phenomenon: Option<String>,
    pub expected_behavior: Option<String>,
    pub actual_behavior: Option<String>,
}
