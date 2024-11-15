use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
pub struct FromClientIpConfig {
    pub range: Option<String>,
    pub ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SelectionConfig {
    #[serde(default)]
    pub with_cookie: Option<WithCookieConfig>,
    #[serde(default)]
    pub from_client_ip: Option<FromClientIpConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostConfig {
    pub domain: String,
    #[serde(default)]
    pub paths: Vec<PathConfig>,
    pub deployments: HashMap<String, DeploymentConfig>,
    #[serde(default)]
    pub middlewares: Vec<String>,
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
}

impl Into<Option<SelectionConfig>> for HostConfig {
    fn into(self) -> Option<SelectionConfig> {
        self.selection
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            domain: "test.example.com".to_string(),
            paths: vec![],
            deployments: HashMap::new(),
            middlewares: vec![],
            selection: None,
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
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
}

impl Into<Option<SelectionConfig>> for DeploymentConfig {
    fn into(self) -> Option<SelectionConfig> {
        self.selection
    }
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            port: 80,
            weight: 100,
            selection: None,
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
    pub selection: Option<SelectionConfig>,
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

    pub fn selection(mut self, selection: SelectionConfig) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn build(self) -> DeploymentConfig {
        DeploymentConfig {
            ip: self.ip,
            port: self.port,
            weight: self.weight,
            selection: self.selection,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InternalDeploymentConfig {
    pub deployment: DeploymentConfig,
    pub name: String,
    pub weight: usize,
}

#[allow(dead_code)]
impl DeploymentConfig {
    pub fn get_weight(&self) -> usize {
        self.get_rules().len()
    }

    pub fn get_rules(&self) -> Vec<String> {
        let mut rules = Vec::new();
        if let Some(selection) = &self.selection {
            if let Some(with_cookie) = &selection.with_cookie {
                rules.push(format!(
                    "{}={}",
                    with_cookie.name,
                    with_cookie.value.as_ref().unwrap_or(&"true".to_string())
                ));
            }
            if let Some(from_client_ip) = &selection.from_client_ip {
                rules.push(format!(
                    "FromClientIP(`{}`)",
                    from_client_ip.ip.as_ref().unwrap_or(&"".to_string())
                ));
            }
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

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum RuleType {
    Host,
    Header,
    ClientIp,
    Other,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct RuleConfig {
//     pub rules: OrderMap<String, (RuleType, String)>,
// }

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rule {
    key: String,
    value: String,
    rule_type: RuleType,
}

impl Rule {
    fn new(key: &str, value: &str, rule_type: RuleType) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            rule_type,
        }
    }

    fn to_string(&self) -> String {
        match self.rule_type {
            RuleType::Other => format!("{}(`{}`)", self.key, self.value),
            RuleType::Header => format!("HeaderRegexp(`{}`, `{}`)", self.key, self.value),
            RuleType::Host => format!("Host(`{}`)", self.value),
            RuleType::ClientIp => format!("ClientIP(`{}`)", self.value),
        }
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self::new("", "", RuleType::Other)
    }
}

#[derive(Debug)]
pub struct RuleConfig {
    rules: HashSet<Rule>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            rules: HashSet::new(),
        }
    }
}

impl RuleConfig {
    pub fn add_rule(&mut self, key: &str, value: &str, rule_type: RuleType) {
        let rule = Rule::new(key, value, rule_type);
        self.rules.insert(rule);
    }

    pub fn add_default_rule(&mut self, key: &str, value: &str) {
        self.add_rule(key, value, RuleType::Other);
    }

    pub fn add_header_rule(&mut self, header: &str, value: &str) {
        let rule = Rule::new(header, value, RuleType::Header);
        self.rules.insert(rule);
    }

    pub fn add_client_ip_rule(&mut self, ip: Option<&str>, range: Option<&str>) {
        if let Some(ip) = ip {
            self.rules.insert(Rule::new("ip", ip, RuleType::ClientIp));
        }
        if let Some(range) = range {
            self.rules
                .insert(Rule::new("range", range, RuleType::ClientIp));
        };
    }

    pub fn add_host_rule(&mut self, domain: &str) {
        self.add_rule("Host", domain, RuleType::Host);
    }

    pub fn rule_str(&self) -> String {
        let rules: Vec<_> = self.rules.iter().map(|r| r.to_string()).collect();
        rules.join(" && ")
    }

    // Weight is now determined by the number of rules
    pub fn get_weight(&self) -> usize {
        self.rules.len()
    }
}

// impl RuleConfig {
//     pub fn get_weight(&self) -> usize {
//         self.rules.values().count()
//     }

//     #[allow(dead_code)]
//     pub fn add_rule(&mut self, key: &str, value: &str) {
//         self.rules
//             .insert(key.to_string(), (RuleType::Other, value.to_string()));
//     }

//     pub fn add_host_rule(&mut self, value: &str) {
//         self.rules
//             .insert(String::from("Host"), (RuleType::Host, value.to_string()));
//     }

//     pub fn add_header_rule(&mut self, key: &str, value: &str) {
//         self.rules
//             .insert(key.to_string(), (RuleType::Header, value.to_string()));
//     }

//     pub fn rule_str(&self) -> String {
//         self.rules
//             .iter()
//             .map(|(k, (rule_type, v))| match rule_type {
//                 RuleType::Other => format!("{}(`{}`)", k, v),
//                 RuleType::Header => format!("HeaderR(`{}`, `{}`)", k, v),
//                 RuleType::Host => format!("Host(`{}`)", v),
//             })
//             .collect::<Vec<String>>()
//             .join(" && ")
//     }
// }

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
