use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    core::Validate,
    error::{TraefikError, TraefikResult},
};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
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
                if value.parse::<u16>().is_err() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_header_value() {
        let headers = HeadersConfig::default();
        assert!(headers.validate().is_ok());
    }

    #[test]
    fn test_validate_header_names() {
        let mut headers = HeadersConfig::default();
        headers
            .custom_request_headers
            .insert("X-Forwarded-Proto".to_string(), "http".to_string());
        assert!(headers.validate().is_ok());
    }
}
