use ordermap::OrderMap;
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
    #[serde(default)]
    pub with_cookie: Option<WithCookieConfig>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            domain: "test.example.com".to_string(),
            paths: vec![],
            deployments: HashMap::new(),
            middlewares: vec![],
            with_cookie: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
    #[serde(default)]
    pub with_cookie: Option<WithCookieConfig>,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 80,
            weight: 100,
            with_cookie: None,
        }
    }
}

#[allow(dead_code)]
impl DeploymentConfig {
    pub fn builder() -> DeploymentConfigBuilder {
        DeploymentConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct DeploymentConfigBuilder {
    pub ip: String,
    pub port: u16,
    pub weight: u8,
    pub with_cookie: Option<WithCookieConfig>,
}

#[allow(dead_code)]
impl DeploymentConfigBuilder {
    pub fn ip(mut self, ip: String) -> Self {
        self.ip = ip;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn weight(mut self, weight: u8) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_cookie(mut self, with_cookie: WithCookieConfig) -> Self {
        self.with_cookie = Some(with_cookie);
        self
    }

    pub fn build(self) -> DeploymentConfig {
        DeploymentConfig {
            ip: self.ip,
            port: self.port,
            weight: self.weight,
            with_cookie: self.with_cookie,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InternalDeploymentConfig {
    pub deployment: DeploymentConfig,
    pub name: String,
}

impl DeploymentConfig {
    pub fn get_weight(&self) -> usize {
        self.get_rules().len()
    }

    pub fn get_rules(&self) -> Vec<String> {
        let mut rules = Vec::new();
        if let Some(with_cookie) = &self.with_cookie {
            rules.push(format!(
                "{}={}",
                with_cookie.name,
                with_cookie.value.as_ref().unwrap_or(&"true".to_string())
            ));
        }
        rules
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WithCookieConfig {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
}

impl Default for WithCookieConfig {
    fn default() -> Self {
        Self {
            name: "TEST_COOKIE".to_string(),
            value: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RuleType {
    Host,
    Header,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuleConfig {
    pub rules: OrderMap<String, (RuleType, String)>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            rules: OrderMap::new(),
        }
    }
}

impl RuleConfig {
    pub fn get_weight(&self) -> usize {
        self.rules.values().count()
    }

    #[allow(dead_code)]
    pub fn add_rule(&mut self, key: &str, value: &str) {
        self.rules
            .insert(key.to_string(), (RuleType::Other, value.to_string()));
    }

    pub fn add_host_rule(&mut self, value: &str) {
        self.rules
            .insert(String::from("Host"), (RuleType::Host, value.to_string()));
    }

    pub fn add_header_rule(&mut self, key: &str, value: &str) {
        self.rules
            .insert(key.to_string(), (RuleType::Header, value.to_string()));
    }

    pub fn rule_str(&self) -> String {
        self.rules
            .iter()
            .map(|(k, (rule_type, v))| match rule_type {
                RuleType::Other => format!("{}(`{}`)", k, v),
                RuleType::Header => format!("HeaderRegexp(`{}`, `{}`)", k, v),
                RuleType::Host => format!("Host(`{}`)", v),
            })
            .collect::<Vec<String>>()
            .join(" && ")
    }
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
