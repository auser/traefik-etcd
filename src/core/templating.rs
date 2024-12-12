use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct ServiceContext {
    ip: String,
    port: u16,
    name: String,
}

impl ServiceContext {
    pub fn new(ip: String, port: u16, name: String) -> Self {
        Self { ip, port, name }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct DeploymentContext {
    service: ServiceContext,
    name: String,
}

impl DeploymentContext {
    pub fn new(service: ServiceContext, name: String) -> Self {
        Self { service, name }
    }
}
