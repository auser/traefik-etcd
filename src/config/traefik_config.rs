use std::collections::{HashMap, HashSet};

use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::{
    core::{
        client::StoreClient,
        etcd_trait::{EtcdPair, ToEtcdPairs},
        rules::{add_deployment_rules, get_sorted_deployments, RouterRule},
        Validate,
    },
    error::{TraefikError, TraefikResult},
    features::etcd::{self, Etcd},
};

use super::{
    deployment::DeploymentConfig,
    host::{HostConfig, PathConfig},
    middleware::MiddlewareConfig,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "frontend/src/types")]
pub struct TraefikConfig {
    #[serde(default = "default_rule_prefix")]
    pub rule_prefix: String,
    #[cfg(feature = "etcd")]
    #[serde(default = "default_etcd_config")]
    pub etcd: etcd::EtcdConfig,
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
    #[serde(default)]
    pub middlewares: HashMap<String, MiddlewareConfig>,
}

fn default_etcd_config() -> etcd::EtcdConfig {
    etcd::EtcdConfig::default()
}

fn default_rule_prefix() -> String {
    "traefik".to_string()
}

impl ToEtcdPairs for TraefikConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        // self.add_defaults(&mut pairs, base_key)?;
        // Start with middleware rules
        for (name, middleware) in self.middlewares.clone().iter_mut() {
            debug!("Applying middleware: {}", name);
            middleware.set_name(name);
            let new_rules = middleware.to_etcd_pairs(base_key)?;
            pairs.extend(new_rules);
        }

        let sorted_hosts = get_sorted_deployments(self)?;
        for deployment_config in sorted_hosts.iter() {
            let mut rules = deployment_config.rules.clone();
            let host = deployment_config.host_config.clone();
            add_deployment_rules(
                &host,
                &[deployment_config.clone()],
                &mut pairs,
                base_key,
                &mut rules,
            )?;
        }

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

impl TraefikConfig {
    pub async fn clean_etcd(&self, client: &StoreClient<Etcd>, _all: bool) -> TraefikResult<()> {
        client.delete_with_prefix(self.rule_prefix.as_str()).await?;
        Ok(())
    }

    pub async fn apply_to_etcd(
        &mut self,
        client: &StoreClient<Etcd>,
        dry_run: bool,
        show_rules: bool,
        should_clean: bool,
    ) -> TraefikResult<()> {
        self.validate()?;
        let pairs = self.to_etcd_pairs(&self.rule_prefix)?;
        let rules = RouterRule::from_pairs(&pairs);

        let mut rule_to_priority: HashMap<String, i32> = HashMap::new();

        // Build priority map
        for rule in &rules {
            rule_to_priority.insert(rule.get_rule().clone(), rule.get_priority());
        }

        if dry_run {
            if show_rules {
                for rule in rules.iter() {
                    println!(
                        "Rule = {} (priority: {})",
                        rule.get_rule(),
                        rule.get_priority()
                    );
                }
            } else {
                for pair in &pairs {
                    println!("Would set: {} = {}", pair.key(), pair.value());
                }
            }
            return Ok(());
        }

        if should_clean {
            self.clean_etcd(client, false).await?;
        }

        let mut client = client.actor.client.clone();

        for pair in pairs.iter() {
            debug!("applying: {:#?}", pair.to_string());
            match client.put(pair.key(), pair.value(), None).await {
                Ok(_kv) => {}
                Err(e) => error!("error: {:?}", e),
            }
        }

        Ok(())
    }
}

impl TraefikConfig {
    pub fn generate_config(domain: Option<String>) -> TraefikConfig {
        let domain = domain.unwrap_or_else(|| "your-domain.com".to_string());

        let host_config = HostConfig::builder()
            .domain(domain)
            .path(
                "/api".to_string(),
                PathConfig::builder()
                    .path("/api".to_string())
                    .deployment(
                        "blue".to_string(),
                        DeploymentConfig::builder()
                            .ip("10.0.0.1".to_string())
                            .port(80)
                            .weight(100)
                            .build(),
                    )
                    .build(),
            )
            .deployment(
                "default".to_string(),
                DeploymentConfig::builder()
                    .ip("10.0.0.1".to_string())
                    .port(80)
                    .weight(100)
                    .build(),
            )
            .build()
            .unwrap();

        // Demo
        let host_configs = vec![host_config];

        TraefikConfig {
            #[cfg(feature = "etcd")]
            etcd: Default::default(),
            middlewares: HashMap::new(),
            hosts: host_configs,
            rule_prefix: "test".to_string(),
        }
    }
}

impl TraefikConfig {
    pub fn builder() -> TraefikConfigBuilder {
        TraefikConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct TraefikConfigBuilder {
    pub rule_prefix: String,
    pub hosts: Vec<HostConfig>,
    pub middlewares: HashMap<String, MiddlewareConfig>,
}

impl TraefikConfigBuilder {
    pub fn rule_prefix(mut self, rule_prefix: String) -> Self {
        self.rule_prefix = rule_prefix;
        self
    }

    pub fn hosts(mut self, hosts: Vec<HostConfig>) -> Self {
        self.hosts = hosts;
        self
    }

    pub fn middlewares(mut self, middlewares: HashMap<String, MiddlewareConfig>) -> Self {
        self.middlewares = middlewares;
        self
    }

    pub fn build(&self) -> TraefikResult<TraefikConfig> {
        Ok(TraefikConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{deployment::DeploymentProtocol, host::HostConfigBuilder};

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
        assert_eq!(
            config.hosts[0].deployments["green"].protocol,
            DeploymentProtocol::Http
        );
        assert_eq!(
            config.hosts[0].deployments["blue"].protocol,
            DeploymentProtocol::Http
        );
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

    #[test]
    fn test_config_can_be_serialized() {
        let config = TraefikConfig::generate_config(None);
        let serialized = serde_yaml::to_string(&config).unwrap();
        assert!(!serialized.is_empty());
        assert!(serialized.contains("domain: your-domain.com"));
    }
}
