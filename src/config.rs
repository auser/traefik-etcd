use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::{
    common::{
        error::{ConfigError, TraefikResult},
        etcd::{Etcd, EtcdConfig, EtcdPair, ToEtcdPairs},
        rollback::get_current_config,
    },
    etcd::{host::HostConfig, middleware::MiddlewareConfig},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TraefikConfig {
    pub etcd: EtcdConfig,
    pub middlewares: HashMap<String, MiddlewareConfig>,
    pub hosts: Vec<HostConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca: PathBuf,
    pub domain: String,
}

impl TraefikConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub async fn clean_etcd(&self, etcd: &mut Etcd) -> TraefikResult<()> {
        Ok(())
    }

    pub async fn apply_to_etcd(&mut self, etcd: &mut Etcd, dry_run: bool) -> TraefikResult<()> {
        let actions = self.construct_global_middlewares();

        Ok(())
    }

    pub fn construct_global_middlewares(&mut self) -> Vec<(String, Vec<EtcdPair>)> {
        let mut actions = vec![];

        let base_key = self.get_base_key("middlewares");

        for (name, middleware) in self.middlewares.iter_mut() {
            middleware.name = name.clone();
            for pair in middleware.to_etcd_pairs(&base_key).iter() {
                actions.push((name.clone(), pair.clone()));
            }
        }
        actions
    }

    fn construct_hosts(&mut self) -> Vec<(String, Vec<EtcdPair>)> {
        let mut actions = vec![];

        let base_key = self.get_base_key("");
        for host in self.hosts.iter_mut() {
            let host_actions = host
                .to_etcd_pairs(&base_key)
                .expect("Failed to convert host to etcd pairs");
            actions.push((host.domain.clone(), host_actions));
        }
        actions
    }

    fn get_base_key(&self, path: &str) -> String {
        format!("traefik/http/{}", path)
    }

    pub async fn save_backup(&self, client: &mut Etcd) -> TraefikResult<String> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_key = format!("traefik/backups/{}", timestamp);

        // Get all current config
        let config = get_current_config(client).await?;

        // Save as backup
        client
            .put(&*backup_key, serde_json::to_string(&config)?, None)
            .await?;

        Ok(timestamp.to_string())
    }

    pub fn validate(&self) -> TraefikResult<()> {
        // Validate middleware references
        for host in &self.hosts {
            for middleware in &host.middlewares {
                if !self.middlewares.contains_key(middleware) {
                    return Err(ConfigError::MiddlewareConfig(format!(
                        "Middleware '{}' referenced in host '{}' does not exist",
                        middleware, host.domain
                    )));
                }
            }
        }

        // Validate deployment weights
        for host in &self.hosts {
            let total_weight = host.deployments.iter().fold(0, |acc, (_, deployment)| {
                acc + deployment.weight.unwrap_or(0)
            });
            if total_weight != 100 {
                if total_weight != 100 {
                    return Err(ConfigError::DeploymentWeight(format!(
                        "Total deployment weight for host '{}' must be 100, got {}",
                        host.domain, total_weight
                    )));
                }
            }
        }

        // Validate paths
        for host in &self.hosts {
            let mut path_set = std::collections::HashSet::new();
            for path in &host.paths {
                if !path.path.starts_with('/') {
                    return Err(ConfigError::PathConfig(format!(
                        "Path '{}' in host '{}' must start with '/'",
                        path.path, host.domain
                    )));
                }
                if !path_set.insert(&path.path) {
                    return Err(ConfigError::DuplicatePath(format!(
                        "Duplicate path '{}' in host '{}'",
                        path.path, host.domain
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_construct_global_middleware() {
        let mut traefik_config = TraefikConfig::load("test/configs/simple.yml").unwrap();
        let actions = traefik_config.construct_global_middlewares();
        assert_eq!(actions.len(), 4);
        let action_names: Vec<String> = actions.iter().map(|a| a.0.clone()).collect();
        vec![
            "handle-redirects",
            "add-www",
            "pass-through",
            "handle-redirects",
        ]
        .iter()
        .all(|name| action_names.contains(&name.to_string()));
    }

    #[test]
    fn test_is_invalid_if_deployment_weights_do_not_sum_to_100() {
        let config = r#"
        etcd:
          endpoints:
            - http://localhost:2379
        middlewares:
        hosts:
          - domain: "ibs.collegegreen.net"
            deployments:
              blue:
                ip: redirector
                port: 3000
                weight: 50
        "#;
        let traefik_config: TraefikConfig = serde_yaml::from_str(config).unwrap();
        assert!(traefik_config.validate().is_err());
    }

    #[test]
    fn test_is_invalid_if_duplicate_path() {
        let config = r#"
        etcd:
          endpoints:
            - http://localhost:2379
        middlewares:
        hosts:
          - domain: "ibs.collegegreen.net"
            paths:
              - path: "/css"
              - path: "/css"
            deployments:
              blue:
                ip: redirector
                port: 3000
                weight: 100
        "#;
        let traefik_config: TraefikConfig = serde_yaml::from_str(config).unwrap();
        assert!(traefik_config.validate().is_err());
    }
}
