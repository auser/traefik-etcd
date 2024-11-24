use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::config::deployment::DeploymentProtocol;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NewDeploymentRequest {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub weight: usize,
    pub protocol: DeploymentProtocol,
}
