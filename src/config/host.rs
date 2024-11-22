use std::collections::HashMap;

use crate::{
    core::Validate,
    error::{TraefikError, TraefikResult},
};

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

impl Validate for HostConfig {
    fn validate(&self) -> TraefikResult<()> {
        // validate domain is not empty
        if self.domain.is_empty() {
            return Err(TraefikError::HostConfig("domain is empty".to_string()));
        }

        // validate paths if they exist
        for path in self.paths.iter() {
            path.validate()?;
        }

        // validate deployments if they exist
        for deployment in self.deployments.values() {
            deployment.validate()?;
        }

        if self.selection.is_some() {
            self.selection.as_ref().unwrap().validate()?;
        }

        Ok(())
    }
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

impl Validate for PathConfig {
    fn validate(&self) -> TraefikResult<()> {
        // validate path is not empty
        if self.path.is_empty() {
            return Err(TraefikError::HostConfig("path is empty".to_string()));
        }

        // validate deployments if they exist
        for deployment in self.deployments.values() {
            deployment.validate()?;
        }

        Ok(())
    }
}
