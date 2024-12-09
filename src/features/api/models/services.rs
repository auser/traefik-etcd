use chrono::{DateTime, Utc};
use export_type::ExportType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, ExportType)]
#[export_type(name = "ServiceInfo", path = "generated/types")]
pub struct ServiceInfo {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
