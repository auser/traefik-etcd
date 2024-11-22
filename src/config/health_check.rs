use serde::{Deserialize, Serialize};

use crate::{
    core::Validate,
    error::{TraefikError, TraefikResult},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthCheckConfig {
    pub path: String,
    pub interval: String,
    pub timeout: String,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            interval: "10s".to_string(),
            timeout: "5s".to_string(),
        }
    }
}

impl Validate for HealthCheckConfig {
    fn validate(&self) -> TraefikResult<()> {
        if self.interval.is_empty() || self.timeout.is_empty() || self.path.is_empty() {
            return Err(TraefikError::HealthCheckConfig(
                "interval, timeout and path must not be empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_config_is_invalid_if_interval_is_empty() {
        let health_check = HealthCheckConfig {
            interval: "".to_string(),
            ..Default::default()
        };
        assert!(health_check.validate().is_err());
    }

    #[test]
    fn test_health_check_config_is_invalid_if_timeout_is_empty() {
        let health_check = HealthCheckConfig {
            timeout: "".to_string(),
            ..Default::default()
        };
        assert!(health_check.validate().is_err());
    }

    #[test]
    fn test_health_check_config_is_invalid_if_path_is_empty() {
        let health_check = HealthCheckConfig {
            path: "".to_string(),
            ..Default::default()
        };
        assert!(health_check.validate().is_err());
    }

    #[test]
    fn test_health_check_config_is_valid() {
        let health_check = HealthCheckConfig::default();
        assert!(health_check.validate().is_ok());
    }
}
