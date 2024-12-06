use std::fmt::Display;

use crate::{
    core::{
        util::{validate_hostname, validate_ip, validate_is_alphanumeric, validate_port},
        Validate,
    },
    error::{TraefikError, TraefikResult},
};
use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::selections::SelectionConfig;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::Type))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub enum DeploymentProtocol {
    #[default]
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "https")]
    Https,
    #[serde(rename = "tcp")]
    Tcp,
    #[serde(other)]
    Invalid,
}

impl From<DeploymentProtocol> for String {
    fn from(protocol: DeploymentProtocol) -> Self {
        match protocol {
            DeploymentProtocol::Http => "http".to_string(),
            DeploymentProtocol::Https => "https".to_string(),
            DeploymentProtocol::Tcp => "tcp".to_string(),
            DeploymentProtocol::Invalid => "invalid".to_string(),
        }
    }
}

impl From<&str> for DeploymentProtocol {
    fn from(value: &str) -> Self {
        match value {
            "http" => DeploymentProtocol::Http,
            "https" => DeploymentProtocol::Https,
            "tcp" => DeploymentProtocol::Tcp,
            _ => DeploymentProtocol::Invalid,
        }
    }
}

impl From<i32> for DeploymentProtocol {
    fn from(protocol: i32) -> Self {
        match protocol {
            1 => DeploymentProtocol::Http,
            2 => DeploymentProtocol::Https,
            3 => DeploymentProtocol::Tcp,
            _ => DeploymentProtocol::Invalid,
        }
    }
}

impl Display for DeploymentProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentProtocol::Http => write!(f, "http"),
            DeploymentProtocol::Https => write!(f, "https"),
            DeploymentProtocol::Tcp => write!(f, "tcp"),
            DeploymentProtocol::Invalid => write!(f, "invalid"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "api", derive(sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct IPAndPort {
    #[serde(default = "default_ip")]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for IPAndPort {
    fn default() -> Self {
        IPAndPort {
            ip: default_ip(),
            port: default_port(),
        }
    }
}

impl Validate for IPAndPort {
    fn validate(&self) -> TraefikResult<()> {
        validate_port(self.port)?;
        validate_ip(&self.ip)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(untagged)]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub enum DeploymentTarget {
    IpAndPort { ip: String, port: u16 },
    Service { service_name: String },
}

impl Default for DeploymentTarget {
    fn default() -> Self {
        DeploymentTarget::IpAndPort {
            ip: default_ip(),
            port: default_port(),
        }
    }
}

impl Display for DeploymentTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentTarget::IpAndPort { ip, port } => write!(f, "{}:{}", ip, port),
            DeploymentTarget::Service { service_name } => write!(f, "{}", service_name),
        }
    }
}

impl Validate for DeploymentTarget {
    fn validate(&self) -> TraefikResult<()> {
        match self {
            DeploymentTarget::IpAndPort { ip, port } => {
                validate_port(*port)?;
                validate_ip(ip)?;
            }
            DeploymentTarget::Service { service_name } => validate_is_alphanumeric(service_name)?,
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "api", derive(sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct DeploymentConfig {
    #[serde(default)]
    pub name: String,
    /// The ip address of the deployment
    #[serde(flatten)]
    pub target: DeploymentTarget,
    /// The weight of the deployment
    #[serde(default = "default_weight")]
    pub weight: usize,
    /// The selection of the deployment
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
    /// The protocol of the deployment
    #[serde(default = "default_protocol")]
    pub protocol: DeploymentProtocol,
    /// The middlewares of the deployment
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub middlewares: Option<Vec<String>>,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            target: DeploymentTarget::default(),
            weight: default_weight(),
            selection: None,
            protocol: default_protocol(),
            middlewares: None,
        }
    }
}

fn default_protocol() -> DeploymentProtocol {
    DeploymentProtocol::Http
}

fn default_port() -> u16 {
    80
}

fn default_weight() -> usize {
    100
}

fn default_ip() -> String {
    "127.0.0.1".to_string()
}

impl DeploymentConfig {
    pub fn builder() -> DeploymentConfigBuilder {
        DeploymentConfigBuilder::default()
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }
}

impl From<DeploymentConfig> for Option<SelectionConfig> {
    fn from(deployment: DeploymentConfig) -> Self {
        deployment.selection
    }
}

#[derive(Default)]
pub struct DeploymentConfigBuilder {
    name: Option<String>,
    target: Option<DeploymentTarget>,
    weight: Option<usize>,
    selection: Option<SelectionConfig>,
    protocol: Option<DeploymentProtocol>,
    middlewares: Option<Vec<String>>,
}

impl DeploymentConfigBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn ip_and_port(mut self, ip: String, port: u16) -> Self {
        self.target = Some(DeploymentTarget::IpAndPort { ip, port });
        self
    }

    pub fn service_name(mut self, service_name: String) -> Self {
        self.target = Some(DeploymentTarget::Service { service_name });
        self
    }

    pub fn weight(mut self, weight: usize) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn selection(mut self, selection: SelectionConfig) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn protocol(mut self, protocol: DeploymentProtocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn middlewares(mut self, middlewares: Vec<String>) -> Self {
        self.middlewares = Some(middlewares);
        self
    }

    pub fn build(self) -> DeploymentConfig {
        let target = self.target.unwrap_or(DeploymentTarget::default());
        DeploymentConfig {
            target,
            weight: self.weight.unwrap_or(default_weight()),
            selection: self.selection,
            protocol: self.protocol.unwrap_or(default_protocol()),
            middlewares: self.middlewares,
            name: self.name.unwrap_or("deployment".to_string()),
        }
    }
}

impl Validate for DeploymentConfig {
    fn validate(&self) -> TraefikResult<()> {
        if self.protocol != DeploymentProtocol::Http
            && self.protocol != DeploymentProtocol::Https
            && self.protocol != DeploymentProtocol::Tcp
        {
            return Err(TraefikError::DeploymentConfig(format!(
                "protocol must be http, https, or tcp, got {}",
                self.protocol
            )));
        }

        // Validate IP format
        match &self.target {
            DeploymentTarget::IpAndPort { ip, port } => {
                validate_port(*port)?;
                self.is_valid_ip_or_hostname(ip)?;
            }
            DeploymentTarget::Service { service_name } => {
                validate_is_alphanumeric(service_name)?;
            }
        };

        if self.weight > 100 {
            return Err(TraefikError::DeploymentConfig(format!(
                "weight must be between 0 and 100, got {}",
                self.weight
            )));
        }

        Ok(())
    }
}

impl DeploymentConfig {
    pub fn validate_path(&self, path: &str) -> TraefikResult<()> {
        validate_is_alphanumeric(path)?;

        Ok(())
    }
    fn is_valid_ip_or_hostname(&self, host: &str) -> TraefikResult<()> {
        validate_ip(host)?;

        // Hostname validation
        if host.is_empty() {
            return Err(TraefikError::DeploymentConfig(format!(
                "Invalid IP or hostname '{}' in deployment",
                host
            )));
        }

        self.validate_valid_hostname(host)?;

        Ok(())
    }

    fn validate_valid_hostname(&self, hostname: &str) -> TraefikResult<()> {
        validate_hostname(hostname)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let deployment = DeploymentConfig::default();
        assert_eq!(deployment.target, DeploymentTarget::default());
        assert_eq!(deployment.weight, 100);
    }

    #[test]
    fn test_protocol_from_str() {
        assert_eq!(DeploymentProtocol::from("http"), DeploymentProtocol::Http);
        assert_eq!(DeploymentProtocol::from("https"), DeploymentProtocol::Https);
        assert_eq!(DeploymentProtocol::from("tcp"), DeploymentProtocol::Tcp);
        assert_eq!(
            DeploymentProtocol::from("invalid"),
            DeploymentProtocol::Invalid
        );
    }

    #[test]
    fn test_default_protocol() {
        let deployment = DeploymentConfig::default();
        assert_eq!(deployment.protocol, DeploymentProtocol::Http);
    }

    #[test]
    fn test_deployment_config_default_values() {
        let deployment = DeploymentConfig::default();
        assert_eq!(deployment.target, DeploymentTarget::default());
        assert_eq!(deployment.weight, 100);
    }

    #[test]
    fn test_deployment_config_is_invalid_if_protocol_is_not_http_or_https_or_tcp() {
        let deployment = DeploymentConfig {
            protocol: DeploymentProtocol::Invalid,
            ..Default::default()
        };
        assert!(deployment.validate().is_err());
    }

    #[test]
    fn test_deployment_config_is_invalid_if_port_is_not_between_1_and_65535() {
        let deployment = DeploymentConfig {
            target: DeploymentTarget::IpAndPort {
                ip: "127.0.0.1".to_string(),
                port: 0,
            },
            ..Default::default()
        };
        assert!(deployment.validate().is_err());
    }

    #[test]
    fn test_deployment_config_is_invalid_if_weight_is_greater_than_100() {
        let deployment = DeploymentConfig {
            weight: 101,
            ..Default::default()
        };
        assert!(deployment.validate().is_err());
    }

    #[test]
    fn test_deployment_config_accepts_protocol_as_http_or_https() {
        let deployment_config = r#"
        ip: redirector
        port: 3000
        weight: 100
        protocol: http
        "#;
        let deployment: DeploymentConfig = serde_yaml::from_str(deployment_config).unwrap();
        assert!(deployment.validate().is_ok());
    }

    #[test]
    fn test_deployment_config_accepts_protocol_as_https() {
        let deployment_config = r#"
        ip: redirector
        port: 3000
        weight: 100
        protocol: https
        "#;
        let deployment: DeploymentConfig = serde_yaml::from_str(deployment_config).unwrap();
        assert!(deployment.validate().is_ok());
    }

    #[test]
    fn test_deployment_config_accepts_protocol_as_tcp() {
        let deployment_config = r#"
        ip: redirector
        port: 3000
        weight: 100
        protocol: tcp
        "#;
        let deployment: DeploymentConfig = serde_yaml::from_str(deployment_config).unwrap();
        let validation = deployment.validate();
        assert!(validation.is_ok());
    }

    #[test]
    fn test_deployment_config_does_not_accept_invalid_protocol() {
        let deployment_config = r#"
        ip: redirector
        port: 3000
        weight: 100
        protocol: csat
        "#;
        let deployment: DeploymentConfig = serde_yaml::from_str(deployment_config).unwrap();
        let validation = deployment.validate();
        assert!(validation.is_err());
    }
}
