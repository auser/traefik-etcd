pub static NAME: &str = "traefikctl";

pub mod config;
pub mod core;
pub mod error;
pub mod features;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "cli")]
pub use cli::run;

pub use config::traefik_config::TraefikConfig;
