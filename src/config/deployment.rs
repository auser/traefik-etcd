use serde::{Deserialize, Serialize};

use crate::{
    core::{
        util::{validate_is_alphanumeric, validate_port},
        Validate,
    },
    error::{TraefikError, TraefikResult},
};

use super::selections::SelectionConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentConfig {
    /// The ip address of the deployment
    #[serde(default = "default_ip")]
    pub ip: String,
    /// The port of the deployment
    #[serde(default = "default_port")]
    pub port: u16,
    /// The weight of the deployment
    #[serde(default)]
    pub weight: u8,
    /// The selection of the deployment
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
    /// The protocol of the deployment
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            ip: default_ip(),
            port: default_port(),
            weight: 100,
            selection: None,
            protocol: default_protocol(),
        }
    }
}

fn default_protocol() -> String {
    "http".to_string()
}

fn default_port() -> u16 {
    80
}

fn default_ip() -> String {
    "127.0.0.1".to_string()
}

impl DeploymentConfig {
    pub fn builder() -> DeploymentConfigBuilder {
        DeploymentConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct DeploymentConfigBuilder {
    ip: String,
    port: u16,
    weight: u8,
    selection: Option<SelectionConfig>,
    protocol: String,
}

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

    pub fn protocol(mut self, protocol: String) -> Self {
        self.protocol = protocol;
        self
    }

    pub fn build(self) -> DeploymentConfig {
        DeploymentConfig {
            ip: self.ip,
            port: self.port,
            weight: self.weight,
            selection: self.selection,
            protocol: self.protocol,
        }
    }
}

impl Validate for DeploymentConfig {
    fn validate(&self) -> TraefikResult<()> {
        if self.protocol != "http" && self.protocol != "https" {
            return Err(TraefikError::DeploymentConfig(format!(
                "protocol must be http, https, or tcp, got {}",
                self.protocol
            )));
        }

        validate_port(self.port)?;

        // Validate IP format
        self.is_valid_ip_or_hostname(&self.ip)?;

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
        // IP address validation
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() == 4 {
            let valid = parts.iter().all(|part| {
                if let Ok(_num) = part.parse::<u8>() {
                    !part.is_empty() && part.len() <= 3
                } else {
                    false
                }
            });
            if !valid {
                return Err(TraefikError::DeploymentConfig(format!(
                    "Invalid IP or hostname '{}' in deployment",
                    host
                )));
            }
        }

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
        fn is_valid_char(byte: u8) -> bool {
            byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.'
        }

        if hostname.bytes().any(|byte| !is_valid_char(byte))
            || hostname
                .split('.')
                .any(|label| label.is_empty() || label.len() > 63 || label.starts_with('-'))
            || hostname.is_empty()
            || hostname.len() > 255
        {
            return Err(TraefikError::DeploymentConfig(format!(
                "Invalid hostname '{}' in deployment",
                hostname
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_protocol() {
        let deployment = DeploymentConfig::default();
        assert_eq!(deployment.protocol, "http".to_string());
    }

    #[test]
    fn test_default_values() {
        let deployment = DeploymentConfig::default();
        assert_eq!(deployment.ip, "127.0.0.1".to_string());
        assert_eq!(deployment.port, 80);
        assert_eq!(deployment.weight, 100);
    }

    #[test]
    fn test_deployment_config_is_invalid_if_protocol_is_not_http_or_https() {
        let deployment = DeploymentConfig {
            protocol: "invalid".to_string(),
            ..Default::default()
        };
        assert!(deployment.validate().is_err());
    }

    #[test]
    fn test_deployment_config_is_invalid_if_port_is_not_between_1_and_65535() {
        let deployment = DeploymentConfig {
            port: 0,
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
}
