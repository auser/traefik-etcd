pub static NAME: &str = "traefikctl";

pub mod config;
pub mod core;
pub mod error;
pub mod features;

#[cfg(test)]
pub(crate) mod test_helpers;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "tracing")]
pub mod tracing;

#[cfg(feature = "cli")]
pub use cli::run;

pub use config::traefik_config::TraefikConfig;
