use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthCheckConfig {
    pub path: String,
    pub check_interval: String,
    pub check_timeout: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewHealthCheckRequest {
    pub deployment_id: String,
    pub path: String,
    pub check_interval: String,
    pub check_timeout: String,
}
