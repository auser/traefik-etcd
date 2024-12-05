use export_type::ExportType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, ExportType)]
#[export_type(name = "TemplateInfo", path = "generated/types")]
pub struct TemplateInfo {
    pub name: String,
    pub path: String,
    pub description: Option<String>, // Could be extracted from a comment in the file
}
