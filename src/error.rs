use std::{num::ParseIntError, str::ParseBoolError};

use thiserror::Error;
use tracing_subscriber::util::TryInitError;

pub type TraefikResult<T = (), E = TraefikError> = color_eyre::Result<T, E>;

#[derive(Error, Debug)]
pub enum TraefikError {
    #[error("IO error: {0}")]
    IOError(Box<dyn std::error::Error>),

    #[error("anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Codegen error: {0}")]
    CodegenError(String),

    #[error("Etcd error: {0}")]
    EtcdError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Host config error: {0}")]
    HostConfig(String),

    #[error("Deployment config error: {0}")]
    DeploymentError(String),

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

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Tracing error: {0}")]
    TracingError(#[from] TryInitError),

    #[error("Config read error: {0}")]
    ConfigReadError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[cfg(feature = "api")]
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[cfg(feature = "api")]
    #[error("Database already exists")]
    DatabaseAlreadyExists,

    #[cfg(feature = "api")]
    #[error("Database creation failed")]
    DatabaseCreationFailed,

    #[cfg(feature = "api")]
    #[error("Database drop failed: {0}")]
    DatabaseDropFailed(String),

    #[cfg(feature = "api")]
    #[error("Migration error: {0}")]
    MigrationError(String),
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

impl From<sqlx::migrate::MigrateError> for TraefikError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        TraefikError::MigrationError(e.to_string())
    }
}

impl From<sqlx::Error> for TraefikError {
    fn from(e: sqlx::Error) -> Self {
        TraefikError::DatabaseError(e.to_string())
    }
}

impl From<url::ParseError> for TraefikError {
    fn from(e: url::ParseError) -> Self {
        TraefikError::ParsingError(e.into())
    }
}
