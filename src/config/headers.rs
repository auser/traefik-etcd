use std::collections::{HashMap, HashSet};

use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        etcd_trait::{EtcdPair, ToEtcdPairs},
        util::format_list_value,
        Validate,
    },
    error::{TraefikError, TraefikResult},
};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct HeadersConfig {
    #[serde(default)]
    pub headers: HashMap<String, String>,
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

impl ToEtcdPairs for HeadersConfig {
    /// Convert the headers configuration to etcd pairs
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        // let headers_base_key = format!("{}/headers", base_key);
        let headers_base_key = "headers".to_string();

        for (key, value) in self.headers.iter() {
            pairs.push(EtcdPair::new(
                format!("{}/{}", headers_base_key, key),
                value.to_string(),
            ));
        }

        // process custom request headers
        for (key, value) in self.custom_request_headers.iter() {
            pairs.push(EtcdPair::new(
                format!("{}/customRequestHeaders/{}", headers_base_key, key),
                value.to_string(),
            ));
        }

        // process custom response headers
        for (key, value) in self.custom_response_headers.iter() {
            pairs.push(EtcdPair::new(
                format!("{}/customResponseHeaders/{}", headers_base_key, key),
                value.to_string(),
            ));
        }

        // Process access control allow methods
        if !self.access_control_allow_methods.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/accessControlAllowMethods", headers_base_key),
                format_list_value(&self.access_control_allow_methods),
            ));
        }

        // Process access control allow headers
        if !self.access_control_allow_headers.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/accessControlAllowHeaders", headers_base_key),
                format_list_value(&self.access_control_allow_headers),
            ));
        }

        // Process access control expose headers
        if !self.access_control_expose_headers.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/accessControlExposeHeaders", headers_base_key),
                format_list_value(&self.access_control_expose_headers),
            ));
        }

        // Process access control allow origin list
        if !self.access_control_allow_origin_list.is_empty() {
            pairs.push(EtcdPair::new(
                format!("{}/accessControlAllowOriginList", headers_base_key),
                format_list_value(&self.access_control_allow_origin_list),
            ));
        }

        // Process add vary header
        if self.add_vary_header {
            pairs.push(EtcdPair::new(
                format!("{}/addVaryHeader", headers_base_key),
                "true".to_string(),
            ));
        }
        Ok(pairs)
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

#[derive(Debug, Clone, Default)]
pub struct HeadersConfigBuilder {
    pub custom_request_headers: HashMap<String, String>,
    pub custom_response_headers: HashMap<String, String>,
    pub access_control_allow_methods: Vec<String>,
    pub access_control_allow_headers: Vec<String>,
    pub access_control_expose_headers: Vec<String>,
    pub access_control_allow_origin_list: Vec<String>,
    pub auth_response_headers: Vec<String>,
    pub auth_response_headers_regex: String,
    pub add_vary_header: bool,
    pub headers: HashMap<String, String>,
}

impl HeadersConfigBuilder {
    pub fn add_custom_request_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.custom_request_headers
            .insert(name.to_string(), value.to_string());
        self
    }

    pub fn add_custom_response_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.custom_response_headers
            .insert(name.to_string(), value.to_string());
        self
    }

    pub fn add_access_control_allow_method(&mut self, method: &str) -> &mut Self {
        self.access_control_allow_methods.push(method.to_string());
        self
    }

    pub fn add_access_control_allow_header(&mut self, header: &str) -> &mut Self {
        self.access_control_allow_headers.push(header.to_string());
        self
    }

    pub fn add_access_control_expose_header(&mut self, header: &str) -> &mut Self {
        self.access_control_expose_headers.push(header.to_string());
        self
    }

    pub fn add_access_control_allow_origin(&mut self, origin: &str) -> &mut Self {
        self.access_control_allow_origin_list
            .push(origin.to_string());
        self
    }

    pub fn add_vary_header(&mut self, add_vary_header: bool) -> &mut Self {
        self.add_vary_header = add_vary_header;
        self
    }

    pub fn add_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn build(&self) -> HeadersConfig {
        HeadersConfig {
            headers: self.headers.clone(),
            custom_request_headers: self.custom_request_headers.clone(),
            custom_response_headers: self.custom_response_headers.clone(),
            access_control_allow_methods: self.access_control_allow_methods.clone(),
            access_control_allow_headers: self.access_control_allow_headers.clone(),
            access_control_expose_headers: self.access_control_expose_headers.clone(),
            access_control_allow_origin_list: self.access_control_allow_origin_list.clone(),
            add_vary_header: self.add_vary_header,
        }
    }
}

impl HeadersConfig {
    pub fn builder() -> HeadersConfigBuilder {
        HeadersConfigBuilder::default()
    }

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

    #[test]
    fn test_to_etcd_pairs_with_custom_request_headers() {
        let headers = HeadersConfig::builder()
            .add_custom_request_header("X-Forwarded-Proto", "https")
            .add_custom_request_header("X-Forwarded-Port", "80")
            .build();
        let pairs = headers.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(
            pair_strs.contains(&"headers/customRequestHeaders/X-Forwarded-Proto https".to_string())
        );
        assert!(pair_strs.contains(&"headers/customRequestHeaders/X-Forwarded-Port 80".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_allow_methods() {
        let headers = HeadersConfig::builder()
            .add_access_control_allow_method("GET")
            .add_access_control_allow_method("POST")
            .add_access_control_allow_method("PUT")
            .build();
        let pairs = headers.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"headers/accessControlAllowMethods GET, POST, PUT".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_allow_headers() {
        let headers = HeadersConfig::builder()
            .add_access_control_allow_header("Content-Type")
            .add_access_control_allow_header("Content-Length")
            .build();
        let pairs = headers.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        println!("pair_strs: {:?}", pair_strs);
        assert!(pair_strs.contains(
            &"headers/accessControlAllowHeaders Content-Type, Content-Length".to_string()
        ));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_expose_headers() {
        let headers = HeadersConfig::builder()
            .add_access_control_expose_header("Content-Type")
            .add_access_control_expose_header("Content-Length")
            .build();
        let pairs = headers.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(
            &"headers/accessControlExposeHeaders Content-Type, Content-Length".to_string()
        ));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_allow_origin_list() {
        let headers = HeadersConfig::builder()
            .add_access_control_allow_origin("example.com")
            .build();
        let pairs = headers.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"headers/accessControlAllowOriginList example.com".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_add_vary_header() {
        let headers = HeadersConfig::builder().add_vary_header(true).build();
        let pairs = headers.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"headers/addVaryHeader true".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_custom_response_headers() {
        let headers = HeadersConfig::builder()
            .add_custom_response_header("Content-Type", "application/json")
            .build();
        let pairs = headers.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs
            .contains(&"headers/customResponseHeaders/Content-Type application/json".to_string()));
    }
}
