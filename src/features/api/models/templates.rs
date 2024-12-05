use chrono::{DateTime, Utc};
use export_type::ExportType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, ExportType)]
#[export_type(name = "TemplateInfo", path = "generated/types")]
pub struct TemplateInfo {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    #[serde(default, rename = "fileTemplate")]
    #[export_type(rename = "fileTemplate")]
    pub file_template: bool,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
