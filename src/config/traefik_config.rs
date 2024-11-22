use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    core::Validate,
    error::{TraefikError, TraefikResult},
    features::etcd,
};

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
        // Validate middlewares
        let mut middlewares = self.middlewares.clone();
        for (name, middleware) in middlewares.iter_mut() {
            // Validate middleware isn't already used
            if self.middlewares.contains_key(name) {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "middleware {} already exists",
                    name
                )));
            }

            middleware.set_name(name);
            middleware.validate()?;
        }

        // Validate hosts
        let mut domain_set: HashSet<String> = HashSet::new();
        for host in self.hosts.iter() {
            if !domain_set.insert(host.domain.clone()) {
                return Err(TraefikError::HostConfig(format!(
                    "duplicate host: {}",
                    host.domain
                )));
            }

            // Validate host
            host.validate()?;

            // Validate host middleware references
            self.validate_middleware_references(host)?;
        }

        Ok(())
    }
}

impl TraefikConfig {
    pub fn validate_middleware_references(&self, host: &HostConfig) -> TraefikResult<()> {
        // Validate host middleware references
        for middleware in host.middlewares.iter() {
            self.validate_middleware_references_in_host(middleware)?;
        }
        Ok(())
    }

    fn validate_middleware_references_in_host(&self, middleware_name: &str) -> TraefikResult<()> {
        if !self.middlewares.contains_key(middleware_name) {
            return Err(TraefikError::MiddlewareConfig(format!(
                "middleware {} not found",
                middleware_name
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_middleware_references() {
        let config = TraefikConfig::default();
        config.validate().unwrap();
    }

    #[test]
    fn test_validate_middleware_references_duplicate_host() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        config.hosts.push(HostConfig::default());
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_duplicate_middleware() {
        let mut config = TraefikConfig::default();
        config
            .middlewares
            .insert("test".to_string(), MiddlewareConfig::default());
        config
            .middlewares
            .insert("test".to_string(), MiddlewareConfig::default());
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_duplicate_middleware_in_host() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_middleware_not_found() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_middleware_not_found_in_host() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_middleware_already_exists() {
        let mut config = TraefikConfig::default();
        config
            .middlewares
            .insert("test".to_string(), MiddlewareConfig::default());
        let result = config.validate();
        assert!(result.is_err());
    }
}
