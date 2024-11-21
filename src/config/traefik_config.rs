use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{core::Validate, error::TraefikResult, features::etcd};

use super::{host::HostConfig, middleware::MiddlewareConfig};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TraefikConfig {
    #[cfg(feature = "etcd")]
    pub etcd: etcd::EtcdConfig,
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
    #[serde(default)]
    pub middlewares: HashMap<String, MiddlewareConfig>,
}

impl Validate for TraefikConfig {
    fn validate(&self) -> TraefikResult<()> {
        let mut middlewares = self.middlewares.clone();
        for (name, middleware) in middlewares.iter_mut() {
            middleware.set_name(name);
            middleware.validate()?;
        }
        Ok(())
    }
}
