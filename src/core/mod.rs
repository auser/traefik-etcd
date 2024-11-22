use crate::error::TraefikResult;

pub mod client;
pub mod util;

#[cfg(feature = "etcd")]
pub mod etcd_trait;

/// Validate the config file
pub trait Validate {
    fn validate(&self) -> TraefikResult<()>;
}
