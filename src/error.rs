use std::{num::ParseIntError, str::ParseBoolError};

use thiserror::Error;
use tracing_subscriber::util::TryInitError;

pub type TraefikResult<T = (), E = TraefikError> = color_eyre::Result<T, E>;

#[derive(Error, Debug)]
pub enum TraefikError {
    #[error("IO error: {0}")]
    IOError(Box<dyn std::error::Error>),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Host config error: {0}")]
    HostConfig(String),

    #[error("Middleware config error: {0}")]
    MiddlewareConfig(String),

    #[error("Deployment config error: {0}")]
    DeploymentConfig(String),

    #[error("Selection config error: {0}")]
    SelectionConfig(String),

    #[error("Health check config error: {0}")]
    HealthCheckConfig(String),

    #[error("Parsing error: {0}")]
    ParsingError(#[from] color_eyre::Report),

    #[error("Tracing error: {0}")]
    TracingError(#[from] TryInitError),

    #[error("Config read error: {0}")]
    ConfigReadError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<Box<dyn std::error::Error>> for TraefikError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        TraefikError::IOError(e)
    }
}

impl From<ParseBoolError> for TraefikError {
    fn from(e: ParseBoolError) -> Self {
        TraefikError::ParsingError(e.into())
    }
}

impl From<ParseIntError> for TraefikError {
    fn from(e: ParseIntError) -> Self {
        TraefikError::ParsingError(e.into())
    }
}

impl From<std::io::Error> for TraefikError {
    fn from(e: std::io::Error) -> Self {
        TraefikError::IOError(e.into())
    }
}
