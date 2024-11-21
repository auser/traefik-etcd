use std::collections::BTreeMap;

use super::{deployment::DeploymentConfig, selections::SelectionConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostConfig {
    pub domain: String,
    #[serde(default)]
    pub paths: Vec<PathConfig>,
    pub deployments: BTreeMap<String, DeploymentConfig>,
    #[serde(default)]
    pub middlewares: Vec<String>,
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PathConfig {
    pub path: String,
    pub deployments: BTreeMap<String, DeploymentConfig>,
    #[serde(default)]
    pub middlewares: Vec<String>,
    #[serde(default)]
    pub strip_prefix: bool,
    #[serde(default)]
    pub pass_through: bool,
}
