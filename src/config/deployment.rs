use serde::{Deserialize, Serialize};

use crate::{
    core::Validate,
    error::{TraefikError, TraefikResult},
};

use super::selections::SelectionConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentConfig {
    #[serde(default = "default_ip")]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub weight: u8,
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
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

impl Validate for DeploymentConfig {
    fn validate(&self) -> TraefikResult<()> {
        if self.protocol != "http" && self.protocol != "https" {
            return Err(TraefikError::DeploymentConfig(format!(
                "protocol must be http, https, or tcp, got {}",
                self.protocol
            )));
        }

        if !(1..=65535).contains(&self.port) {
            return Err(TraefikError::DeploymentConfig(format!(
                "port must be between 1 and 65535, got {}",
                self.port
            )));
        }

        if self.weight > 100 {
            return Err(TraefikError::DeploymentConfig(format!(
                "weight must be between 0 and 100, got {}",
                self.weight
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
