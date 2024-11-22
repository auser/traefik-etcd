use std::collections::HashMap;

use super::{deployment::DeploymentConfig, selections::SelectionConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HostConfig {
    /// The domain of the host
    pub domain: String,
    /// The paths of the host
    #[serde(default)]
    pub paths: Vec<PathConfig>,
    /// The deployments of the host
    #[serde(default)]
    pub deployments: HashMap<String, DeploymentConfig>,
    /// The middlewares of the host
    #[serde(default)]
    pub middlewares: Vec<String>,
    /// The selection configuration of the host
    /// This is flattened to allow for more complex selection configurations
    /// such as weighted selections.
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PathConfig {
    /// The path of the host
    pub path: String,
    /// The deployments of the path
    pub deployments: HashMap<String, DeploymentConfig>,
    /// The middlewares of the path
    #[serde(default)]
    pub middlewares: Vec<String>,
    /// Whether to strip the prefix from the path
    #[serde(default)]
    pub strip_prefix: bool,
    /// Whether to pass through the path to the backend
    #[serde(default)]
    pub pass_through: bool,
}
