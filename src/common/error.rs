use std::{num::ParseIntError, str::ParseBoolError};

use thiserror::Error;
use tracing_subscriber::util::TryInitError;

pub type TraefikResult<T = (), E = ConfigError> = color_eyre::Result<T, E>;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IOError(Box<dyn std::error::Error>),

    #[error("Deployment weight error: {0}")]
    DeploymentWeight(String),

    #[error("Etcd TLS incorrectly configured: {0}")]
    EtcdConfig(String),

    #[error("Invalid middleware configuration: {0}")]
    MiddlewareConfig(String),

    #[error("Etcd error: {0}")]
    Etcd(#[from] etcd_client::Error),

    #[error("Path error: {0}")]
    PathConfig(String),

    #[error("Duplicate path: {0}")]
    DuplicatePath(String),

    #[error("Backend error: {0}")]
    BackendConfig(String),

    #[error("Health check error: {0}")]
    HealthCheckConfig(String),

    #[error("TryInitError: {0}")]
    TryInitError(#[from] TryInitError),

    #[error("Etcd error: {0}")]
    EtcdError(#[from] color_eyre::Report),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl From<Box<dyn std::error::Error>> for ConfigError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ConfigError::IOError(e)
    }
}

impl From<ParseBoolError> for ConfigError {
    fn from(e: ParseBoolError) -> Self {
        ConfigError::EtcdError(e.into())
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self {
        ConfigError::EtcdError(e.into())
    }
}
