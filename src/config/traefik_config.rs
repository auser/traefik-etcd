use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    core::{
        etcd_trait::{EtcdPair, ToEtcdPairs},
        Validate,
    },
    error::{TraefikError, TraefikResult},
    features::etcd,
};

use super::{host::HostConfig, middleware::MiddlewareConfig};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TraefikConfig {
    #[serde(default = "default_rule_prefix")]
    pub rule_prefix: String,
    #[cfg(feature = "etcd")]
    pub etcd: etcd::EtcdConfig,
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
    #[serde(default)]
    pub middlewares: HashMap<String, MiddlewareConfig>,
}

fn default_rule_prefix() -> String {
    "traefik".to_string()
}

impl ToEtcdPairs for TraefikConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        // Start with middleware rules
        for (name, middleware) in self.middlewares.iter() {
            let mw_prefix = format!("{}/{}/middlewares/{}", base_key, middleware.protocol, name);
            let new_rules = middleware.to_etcd_pairs(&mw_prefix)?;
            pairs.extend(new_rules);
        }
        pairs.push(EtcdPair::new(
            format!("{}/rule_prefix", base_key),
            self.rule_prefix.clone(),
        ));
        Ok(pairs)
    }
}

impl Validate for TraefikConfig {
    fn validate(&self) -> TraefikResult<()> {
        // Validate middlewares
        let mut middlewares = self.middlewares.clone();
        for (name, middleware) in middlewares.iter_mut() {
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
    use crate::config::host::HostConfigBuilder;

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
    fn test_validate_middleware_references_middleware_found_in_host() {
        let mut config = TraefikConfig::default();
        config
            .middlewares
            .insert("test".to_string(), MiddlewareConfig::default());
        config.hosts.push(
            HostConfigBuilder::default()
                .middleware("test".to_string())
                .build()
                .unwrap(),
        );
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_with_complex_config() {
        let config_str = r#"
        etcd:
            endpoints: ["https://0.0.0.0:2379"]
            timeout: 2000
            keep_alive: 300
            tls:
                cert: "./config/tls/etcd-peer.pem"
                key: "./config/tls/etcd-peer-key.pem"
                ca: "./config/tls/ca.pem"
                domain: herringbank.com
        middlewares:
        enable-headers:
            headers:
            custom_request_headers:
                X-Forwarded-Proto: "https"
                X-Forwarded-Port: "443"
                Location: ""

        hosts:
            - domain: ari.io
              paths:
                - path: /
                  deployments:
                    green_with_cookie:
                        ip: 10.0.0.1
                        port: 80
                        weight: 100
              deployments:
                green:
                    protocol: http
                    port: 80
                    weight: 50
                blue:
                    protocol: http
                    port: 80
                    weight: 50
        "#;
        let config: TraefikConfig = serde_yaml::from_str(config_str).unwrap();
        let validation_result = config.validate();
        assert!(validation_result.is_ok());
        assert_eq!(config.hosts.len(), 1);
        assert_eq!(config.hosts[0].domain, "ari.io");
        assert_eq!(config.hosts[0].deployments.len(), 2);
        assert_eq!(config.hosts[0].deployments["green"].port, 80);
        assert_eq!(config.hosts[0].deployments["blue"].port, 80);
        assert_eq!(config.hosts[0].deployments["green"].weight, 50);
        assert_eq!(config.hosts[0].deployments["blue"].weight, 50);
        assert_eq!(config.hosts[0].deployments["green"].protocol, "http");
        assert_eq!(config.hosts[0].deployments["blue"].protocol, "http");
        let paths = config.hosts[0].paths.iter().find(|p| p.path == "/");
        assert!(paths.is_some());
        let path = paths.unwrap();
        assert_eq!(path.deployments.len(), 1);
        assert_eq!(path.deployments["green_with_cookie"].port, 80);
        assert_eq!(path.deployments["green_with_cookie"].weight, 100);
        assert_eq!(path.deployments["green_with_cookie"].ip, "10.0.0.1");
    }

    #[test]
    fn test_validate_middleware_references_www_redirect() {
        let config_str = include_str!("../../config/config.yml");
        let config: TraefikConfig = serde_yaml::from_str(config_str).unwrap();
        let validation_result = config.validate();
        assert!(validation_result.is_ok());
    }
}
