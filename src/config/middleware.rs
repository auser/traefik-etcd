use convert_case::{Case, Casing};
use std::collections::{HashMap, HashSet};

use crate::{
    config::core_traits::{EtcdPair, ToEtcdPairs, Validate},
    error::{TraefikError, TraefikResult},
};

use super::base_structs::{HeadersConfig, MiddlewareConfig};

impl ToEtcdPairs for MiddlewareConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let middleware_key = format!("{}/{}", base_key, self.name);

        if let Some(headers) = &self.headers {
            // Handle header configurations
            self.add_header_pairs(&middleware_key, headers, &mut pairs)?;
        }

        Ok(pairs)
    }
}

impl MiddlewareConfig {
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn add_header_pairs(
        &self,
        middleware_key: &str,
        headers: &HeadersConfig,
        pairs: &mut Vec<EtcdPair>,
    ) -> TraefikResult<()> {
        // Process custom request headers
        for (key, value) in &headers.custom_request_headers {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/headers/customRequestHeaders/{}",
                    middleware_key,
                    format_header_key(key)
                ),
                value.clone(),
            ));
        }

        // Process custom response headers
        for (key, value) in &headers.custom_response_headers {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/headers/customResponseHeaders/{}",
                    middleware_key,
                    format_header_key(key)
                ),
                value.clone(),
            ));
        }

        // Process access control headers
        if !headers.access_control_allow_methods.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlAllowMethods", middleware_key),
                format_list_value(&headers.access_control_allow_methods),
            ));
        }

        if !headers.access_control_allow_headers.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlAllowHeaders", middleware_key),
                format_list_value(&headers.access_control_allow_headers),
            ));
        }

        if !headers.access_control_expose_headers.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlExposeHeaders", middleware_key),
                format_list_value(&headers.access_control_expose_headers),
            ));
        }

        if !headers.access_control_allow_origin_list.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlAllowOriginList", middleware_key),
                format_list_value(&headers.access_control_allow_origin_list),
            ));
        }

        if headers.add_vary_header {
            pairs.push(EtcdPair::new(
                format!("{}/headers/addVaryHeader", middleware_key),
                "true".to_string(),
            ));
        }

        Ok(())
    }
}

impl Validate for MiddlewareConfig {
    fn validate(&self) -> TraefikResult<()> {
        // Validate middleware name
        if self.name.is_empty() {
            return Err(TraefikError::MiddlewareConfig(
                "Middleware name cannot be empty".into(),
            ));
        }

        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(TraefikError::MiddlewareConfig(format!(
                "Invalid middleware name '{}': must contain only alphanumeric characters, hyphens, or underscores",
                self.name
            )));
        }

        // Validate headers if present
        if let Some(headers) = &self.headers {
            headers.validate()?;
        }

        Ok(())
    }
}

impl Validate for HeadersConfig {
    fn validate(&self) -> TraefikResult<()> {
        // Validate HTTP methods
        let valid_methods: HashSet<&str> = vec![
            "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE", "CONNECT",
        ]
        .into_iter()
        .collect();

        for method in &self.access_control_allow_methods {
            if !valid_methods.contains(method.as_str()) {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Invalid HTTP method: {}",
                    method
                )));
            }
        }

        // Validate custom headers
        self.validate_header_names(&self.custom_request_headers)?;
        self.validate_header_names(&self.custom_response_headers)?;

        // Validate header lists
        self.validate_header_list(
            &self.access_control_allow_headers,
            "Access-Control-Allow-Headers",
        )?;
        self.validate_header_list(
            &self.access_control_expose_headers,
            "Access-Control-Expose-Headers",
        )?;

        // Validate header values
        for (name, value) in &self.custom_request_headers {
            self.validate_header_value(name, value)?;
        }
        for (name, value) in &self.custom_response_headers {
            self.validate_header_value(name, value)?;
        }

        Ok(())
    }
}

impl HeadersConfig {
    fn validate_header_names(&self, headers: &HashMap<String, String>) -> TraefikResult<()> {
        for name in headers.keys() {
            if name.is_empty() {
                return Err(TraefikError::MiddlewareConfig(
                    "Header name cannot be empty".into(),
                ));
            }

            if !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
            {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Invalid header name '{}': must contain only alphanumeric characters, hyphens, underscores, dots, or forward slashes",
                    name
                )));
            }
        }
        Ok(())
    }

    fn validate_header_list(&self, headers: &[String], context: &str) -> TraefikResult<()> {
        let mut seen = HashSet::new();
        for header in headers {
            let header_lower = header.to_lowercase();
            if !seen.insert(header_lower) {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Duplicate header in {}: {}",
                    context, header
                )));
            }

            if header.is_empty() {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Empty header name in {}",
                    context
                )));
            }
        }
        Ok(())
    }

    fn validate_header_value(&self, name: &str, value: &str) -> TraefikResult<()> {
        if value.chars().any(|c| c.is_control() && c != '\t') {
            return Err(TraefikError::MiddlewareConfig(format!(
                "Invalid value for header '{}': contains control characters",
                name
            )));
        }

        match name.to_lowercase().as_str() {
            "x-forwarded-proto" => {
                if !["http", "https"].contains(&value.to_lowercase().as_str()) {
                    return Err(TraefikError::MiddlewareConfig(format!(
                        "Invalid value for X-Forwarded-Proto: must be 'http' or 'https', got '{}'",
                        value
                    )));
                }
            }
            "x-forwarded-port" => {
                if !value.parse::<u16>().is_ok() {
                    return Err(TraefikError::MiddlewareConfig(format!(
                        "Invalid value for X-Forwarded-Port: must be a valid port number, got '{}'",
                        value
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

// Helper functions
fn format_header_key(key: &str) -> String {
    key.to_case(Case::Pascal)
}

fn format_list_value(values: &[String]) -> String {
    values.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_headers() -> HeadersConfig {
        HeadersConfig {
            custom_request_headers: HashMap::from([
                ("X-Forwarded-Proto".to_string(), "https".to_string()),
                ("X-Forwarded-Port".to_string(), "443".to_string()),
            ]),
            custom_response_headers: HashMap::from([("Location".to_string(), "".to_string())]),
            access_control_allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "OPTIONS".to_string(),
            ],
            access_control_allow_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
            ],
            access_control_expose_headers: vec!["Location".to_string()],
            access_control_allow_origin_list: vec!["*".to_string()],
            add_vary_header: true,
        }
    }

    #[test]
    fn test_middleware_headers_conversion() {
        let config = MiddlewareConfig {
            name: "test-headers".to_string(),
            headers: Some(create_test_headers()),
        };

        let pairs = config.to_etcd_pairs("traefik/http/middlewares").unwrap();

        // Verify header case conversion
        assert!(pairs.iter().any(|p| p.key().contains("/XForwardedProto")));
        assert!(pairs.iter().any(|p| p.key().contains("/XForwardedPort")));

        // Verify list formatting
        let methods = pairs
            .iter()
            .find(|p| p.key().contains("accessControlAllowMethods"))
            .unwrap();
        assert_eq!(methods.value(), "GET, POST, OPTIONS");
    }

    #[test]
    fn test_validate_valid_middleware() {
        let config = MiddlewareConfig {
            name: "test-headers".to_string(),
            headers: Some(create_test_headers()),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_middleware_name() {
        let config = MiddlewareConfig {
            name: "".to_string(),
            headers: Some(create_test_headers()),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_method() {
        let mut headers = create_test_headers();
        headers.access_control_allow_methods = vec!["INVALID".to_string()];

        let config = MiddlewareConfig {
            name: "test".to_string(),
            headers: Some(headers),
        };
        assert!(config.validate().is_err());
    }
}
