use crate::error::TraefikResult;

pub mod client;

#[cfg(feature = "etcd")]
pub mod etcd_trait;

/// Validate the config file
///
/// # Example
///
/// ```rust
/// use traefikctl::config::TraefikConfig;
///
/// let config = TraefikConfig::new();
/// config.validate().unwrap();
/// ```
pub trait Validate {
    fn validate(&self) -> TraefikResult<()>;
}
