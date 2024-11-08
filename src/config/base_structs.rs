use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{error::TraefikResult, etcd::EtcdConfig};

use super::core_traits::{EtcdPair, ToEtcdPairs};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TraefikConfig {
    pub etcd: EtcdConfig,
    #[serde(default)]
    pub middlewares: HashMap<String, MiddlewareConfig>,
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
    #[serde(default)]
    pub www_redirect: Option<bool>,
    #[serde(default)]
    pub redirector: RedirectorConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostConfig {
    pub domain: String,
    #[serde(default)]
    pub paths: Vec<PathConfig>,
    pub deployments: HashMap<String, DeploymentConfig>,
    #[serde(default)]
    pub middlewares: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathConfig {
    pub path: String,
    pub deployments: HashMap<String, DeploymentConfig>,
    #[serde(default)]
    pub middlewares: Vec<String>,
    #[serde(default)]
    pub strip_prefix: bool,
    #[serde(default)]
    pub pass_through: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentConfig {
    pub ip: String,
    pub port: u16,
    #[serde(default)]
    pub weight: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MiddlewareConfig {
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HeadersConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct HeadersConfig {
    #[serde(default)]
    pub custom_request_headers: HashMap<String, String>,
    #[serde(default)]
    pub custom_response_headers: HashMap<String, String>,
    #[serde(default)]
    pub access_control_allow_methods: Vec<String>,
    #[serde(default)]
    pub access_control_allow_headers: Vec<String>,
    #[serde(default)]
    pub access_control_expose_headers: Vec<String>,
    #[serde(default)]
    pub access_control_allow_origin_list: Vec<String>,
    #[serde(default)]
    pub add_vary_header: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthCheckConfig {
    pub path: String,
    pub interval: String,
    pub timeout: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedirectorConfig {
    #[serde(default = "default_redirector_url")]
    pub url: String,
    #[serde(default)]
    pub health_check: HealthCheckConfig,
}

// Default implementations
impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            interval: "10s".to_string(),
            timeout: "5s".to_string(),
        }
    }
}

fn default_redirector_url() -> String {
    "http://redirector:3000".to_string()
}

impl Default for RedirectorConfig {
    fn default() -> Self {
        Self {
            url: default_redirector_url(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

impl ToEtcdPairs for HealthCheckConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        // Health check configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/healthCheck/path",
                base_key
            ),
            "/health".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/healthCheck/interval",
                base_key
            ),
            "10s".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/healthCheck/timeout",
                base_key
            ),
            "5s".to_string(),
        ));

        Ok(pairs)
    }
}
