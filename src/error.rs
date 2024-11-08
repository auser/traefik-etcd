use std::{num::ParseIntError, str::ParseBoolError};

use thiserror::Error;
use tracing_subscriber::util::TryInitError;

pub type TraefikResult<T = (), E = TraefikError> = color_eyre::Result<T, E>;

#[derive(Error, Debug)]
pub enum TraefikError {
    #[error("IO error: {0}")]
    IOError(Box<dyn std::error::Error>),

    #[error("Middleware config error: {0}")]
    MiddlewareConfig(String),

    #[error("Deployment weight error: {0}")]
    DeploymentWeight(String),

    #[error("Deployment error: {0}")]
    DeploymentError(String),

    #[allow(dead_code)]
    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Etcd error: {0}")]
    Etcd(#[from] etcd_client::Error),

    #[error("Path error: {0}")]
    PathConfig(String),

    #[error("TryInitError: {0}")]
    TryInitError(#[from] TryInitError),

    #[error("Etcd error: {0}")]
    EtcdError(#[from] color_eyre::Report),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl From<Box<dyn std::error::Error>> for TraefikError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        TraefikError::IOError(e)
    }
}

impl From<ParseBoolError> for TraefikError {
    fn from(e: ParseBoolError) -> Self {
        TraefikError::EtcdError(e.into())
    }
}

impl From<ParseIntError> for TraefikError {
    fn from(e: ParseIntError) -> Self {
        TraefikError::EtcdError(e.into())
    }
}
