use crate::error::{TraefikError, TraefikResult};

use super::{
    base_structs::RedirectorConfig,
    core_traits::{EtcdPair, ToEtcdPairs, Validate},
};

impl ToEtcdPairs for RedirectorConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let service_key = format!("{}/services/redirector", base_key);

        // Basic service configuration
        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/servers/0/url", service_key),
            self.url.clone(),
        ));

        // passHostHeader configuration
        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/passHostHeader", service_key),
            "true".to_string(),
        ));

        // Response forwarding configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/loadBalancer/responseForwarding/flushInterval",
                service_key
            ),
            "100ms".to_string(),
        ));

        // Health check configuration
        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/healthCheck/path", service_key),
            self.health_check.path.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/healthCheck/interval", service_key),
            self.health_check.interval.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/healthCheck/timeout", service_key),
            self.health_check.timeout.clone(),
        ));

        Ok(pairs)
    }
}

impl Validate for RedirectorConfig {
    fn validate(&self) -> TraefikResult<()> {
        // Validate URL
        if self.url.is_empty() {
            return Err(TraefikError::ConfigError(
                "Redirector URL cannot be empty".to_string(),
            ));
        }

        // Basic URL format validation
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(TraefikError::ConfigError(
                "Redirector URL must start with http:// or https://".to_string(),
            ));
        }

        // Validate health check
        self.validate_health_check()?;

        Ok(())
    }
}

impl RedirectorConfig {
    fn validate_health_check(&self) -> TraefikResult<()> {
        // Validate health check path
        if !self.health_check.path.starts_with('/') {
            return Err(TraefikError::ConfigError(
                "Health check path must start with /".to_string(),
            ));
        }

        // Validate time durations (basic format check)
        if !self.health_check.interval.ends_with('s') {
            return Err(TraefikError::ConfigError(
                "Health check interval must be specified in seconds (e.g., '10s')".to_string(),
            ));
        }

        if !self.health_check.timeout.ends_with('s') {
            return Err(TraefikError::ConfigError(
                "Health check timeout must be specified in seconds (e.g., '5s')".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::base_structs::HealthCheckConfig;

    use super::*;

    fn create_test_config() -> RedirectorConfig {
        RedirectorConfig {
            url: "http://redirector:3000".to_string(),
            health_check: HealthCheckConfig {
                path: "/health".to_string(),
                interval: "10s".to_string(),
                timeout: "5s".to_string(),
            },
        }
    }

    #[test]
    fn test_redirector_service_configuration() {
        let config = create_test_config();
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify service URL
        let has_service_url = pairs.iter().any(|p| {
            p.key().contains("/loadBalancer/servers/0/url") && p.value() == "http://redirector:3000"
        });
        assert!(has_service_url, "Redirector service URL not found");

        // Verify passHostHeader
        let has_pass_host_header = pairs
            .iter()
            .any(|p| p.key().contains("/loadBalancer/passHostHeader") && p.value() == "true");
        assert!(has_pass_host_header, "passHostHeader not found");

        // Verify flushInterval
        let has_flush_interval = pairs.iter().any(|p| {
            p.key()
                .contains("/loadBalancer/responseForwarding/flushInterval")
                && p.value() == "100ms"
        });
        assert!(has_flush_interval, "flushInterval not found");

        // Verify health check configuration
        let has_health_check_path = pairs
            .iter()
            .any(|p| p.key().contains("/healthCheck/path") && p.value() == "/health");
        assert!(has_health_check_path, "Health check path not found");

        let has_health_check_interval = pairs
            .iter()
            .any(|p| p.key().contains("/healthCheck/interval") && p.value() == "10s");
        assert!(has_health_check_interval, "Health check interval not found");

        let has_health_check_timeout = pairs
            .iter()
            .any(|p| p.key().contains("/healthCheck/timeout") && p.value() == "5s");
        assert!(has_health_check_timeout, "Health check timeout not found");
    }

    #[test]
    fn test_redirector_validation() {
        let valid_config = create_test_config();
        assert!(valid_config.validate().is_ok());

        // Test invalid URL
        let invalid_url = RedirectorConfig {
            url: "redirector:3000".to_string(),
            ..valid_config.clone()
        };
        assert!(invalid_url.validate().is_err());

        // Test empty URL
        let empty_url = RedirectorConfig {
            url: "".to_string(),
            ..valid_config.clone()
        };
        assert!(empty_url.validate().is_err());

        // Test invalid health check path
        let invalid_path = RedirectorConfig {
            health_check: HealthCheckConfig {
                path: "health".to_string(),
                ..valid_config.health_check.clone()
            },
            ..valid_config.clone()
        };
        assert!(invalid_path.validate().is_err());

        // Test invalid interval format
        let invalid_interval = RedirectorConfig {
            health_check: HealthCheckConfig {
                interval: "10".to_string(),
                ..valid_config.health_check.clone()
            },
            ..valid_config.clone()
        };
        assert!(invalid_interval.validate().is_err());

        // Test invalid timeout format
        let invalid_timeout = RedirectorConfig {
            health_check: HealthCheckConfig {
                timeout: "5".to_string(),
                ..valid_config.health_check
            },
            ..valid_config
        };
        assert!(invalid_timeout.validate().is_err());
    }
}
