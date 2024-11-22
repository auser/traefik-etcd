use crate::error::TraefikResult;

pub mod client;
pub mod rules;
pub mod util;

pub static TCP_BASE_KEY: &str = "traefik/tcp";
pub static HTTP_BASE_KEY: &str = "traefik/http";

#[cfg(feature = "etcd")]
pub mod etcd_trait;

/// Validate the config file
pub trait Validate {
    fn validate(&self) -> TraefikResult<()>;
}

pub type ClientBuildResult = (String, String);

// TODO: implement this trait for all config types?
pub trait Build {
    fn build(
        &self,
        rule_prefix: &str,
        builder: &impl Build,
    ) -> TraefikResult<Vec<ClientBuildResult>>;
}
