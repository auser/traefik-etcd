#![allow(dead_code)]
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    error::{TraefikError, TraefikResult},
    etcd::{util::get_safe_key, Etcd, EtcdConfig, EtcdPair, ToEtcdPairs},
};

#[derive(Debug, Serialize, Deserialize, Default)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct HostConfig {
    pub domain: String,
    #[serde(default)]
    pub paths: Vec<PathConfig>,
    pub deployments: HashMap<String, DeploymentConfig>,
    #[serde(default)]
    pub middlewares: Vec<String>,
    #[serde(default)]
    pub pass_through: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedirectRegexMiddleware {
    pub regex: String,
    pub replacement: String,
    #[serde(default = "default_permanent")]
    pub permanent: bool,
}

fn default_permanent() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HeadersConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_through: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PathConfig {
    pub path: String,
    pub deployments: HashMap<String, DeploymentConfig>,
    #[serde(default)]
    pub middlewares: Vec<String>,
    #[serde(default)]
    pub strip_prefix: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentConfig {
    pub ip: String,
    pub port: u16,
    #[serde(default)]
    pub weight: u8,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedirectorConfig {
    #[serde(default = "default_redirector_url")]
    pub url: String,
    #[serde(default)]
    pub health_check: HealthCheckConfig,
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

impl ToEtcdPairs for TraefikConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        // Add redirector service configuration
        self.add_redirector_service(&mut pairs, base_key)?;

        // Add www redirect configuration if enabled
        self.add_www_redirect(&mut pairs, base_key)?;

        // Add middleware configurations
        for (_name, middleware) in &self.middlewares {
            let middleware_pairs =
                middleware.to_etcd_pairs(&format!("{}/middlewares", base_key))?;
            pairs.extend(middleware_pairs);
        }

        // Add host configurations
        for host in &self.hosts {
            let host_pairs = host.to_etcd_pairs(base_key)?;
            pairs.extend(host_pairs);

            // Add www redirect router for non-www domains if enabled
            if self.www_redirect.unwrap_or(false) && !host.domain.starts_with("www.") {
                self.add_www_redirect_router(&mut pairs, base_key, &host.domain)?;
            }
        }

        Ok(pairs)
    }
}

impl TraefikConfig {
    pub async fn clean_etcd(&self, etcd: &mut Etcd, all: bool) -> TraefikResult<()> {
        if all {
            etcd.delete_with_prefix("traefik/http").await?;
        } else {
            for host in &self.hosts {
                host.clean_etcd(etcd).await?;
            }
        }
        Ok(())
    }

    pub async fn apply_to_etcd(&mut self, etcd: &mut Etcd, dry_run: bool) -> TraefikResult<()> {
        // First validate the configuration
        self.validate()?;

        // Generate all etcd pairs
        let pairs = self.to_etcd_pairs("traefik/http")?;

        if dry_run {
            // Just print what would be applied
            for pair in pairs {
                println!("Would set: {} = {}", pair.key(), pair.value());
            }
            return Ok(());
        }

        // Clean existing configuration first
        self.clean_etcd(etcd, false).await?;

        // Apply all pairs to etcd
        for pair in pairs {
            etcd.put(pair.key(), pair.value(), None)
                .await
                .map_err(|e| TraefikError::EtcdError(e.into()))?;
        }

        Ok(())
    }

    fn add_www_redirect(&self, pairs: &mut Vec<EtcdPair>, base_key: &str) -> TraefikResult<()> {
        if self.www_redirect.unwrap_or(false) {
            // Add the www redirect middleware configuration
            pairs.push(EtcdPair::new(
                format!("{}/middlewares/add-www/redirectregex/permanent", base_key),
                "true".to_string(),
            ));

            pairs.push(EtcdPair::new(
                format!("{}/middlewares/add-www/redirectregex/regex", base_key),
                "^https://([^.]+\\.[^.]+\\.[^.]+)(.*)".to_string(),
            ));

            pairs.push(EtcdPair::new(
                format!("{}/middlewares/add-www/redirectregex/replacement", base_key),
                "https://www.${1}${2}".to_string(),
            ));
        }
        Ok(())
    }

    fn add_www_redirect_router(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        domain: &str,
    ) -> TraefikResult<()> {
        let safe_name = format!("to-www-{}", get_safe_key(domain));

        // Add router to catch non-www version
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/rule", base_key, safe_name),
            format!("Host(`{}`)", domain),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/entrypoints/0", base_key, safe_name),
            "websecure".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/middlewares/0", base_key, safe_name),
            "add-www".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/tls", base_key, safe_name),
            "true".to_string(),
        ));

        // Set higher priority for the redirect router
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/priority", base_key, safe_name),
            "200".to_string(),
        ));

        Ok(())
    }

    fn add_redirector_service(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
    ) -> TraefikResult<()> {
        // Basic service configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/servers/0/url",
                base_key
            ),
            self.redirector.url.clone(),
        ));

        // passHostHeader configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/passHostHeader",
                base_key
            ),
            "true".to_string(),
        ));

        // Response forwarding configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/responseForwarding/flushInterval",
                base_key
            ),
            "100ms".to_string(),
        ));

        // Health check configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/healthCheck/path",
                base_key
            ),
            self.redirector.health_check.path.clone(),
        ));

        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/healthCheck/interval",
                base_key
            ),
            self.redirector.health_check.interval.clone(),
        ));

        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/healthCheck/timeout",
                base_key
            ),
            self.redirector.health_check.timeout.clone(),
        ));

        Ok(())
    }
}

impl TraefikConfig {
    pub fn validate(&mut self) -> TraefikResult<()> {
        // Validate middlewares
        for (name, middleware) in self.middlewares.iter_mut() {
            if name.is_empty() {
                return Err(TraefikError::MiddlewareConfig(
                    "Middleware name cannot be empty".to_string(),
                ));
            }
            middleware.set_name(name);
            middleware.validate()?;
        }

        // Validate hosts
        let mut domain_set = HashSet::new();
        for host in &self.hosts {
            // Check for duplicate domains
            if !domain_set.insert(&host.domain) {
                return Err(TraefikError::ConfigError(format!(
                    "Duplicate domain: {}",
                    host.domain
                )));
            }

            // Validate host configuration
            host.validate()?;

            // Validate middleware references
            for middleware_name in &host.middlewares {
                if !self.middlewares.contains_key(middleware_name) {
                    return Err(TraefikError::MiddlewareConfig(format!(
                        "Undefined middleware '{}' referenced in host '{}'",
                        middleware_name, host.domain
                    )));
                }
            }
        }

        Ok(())
    }
}
impl ToEtcdPairs for MiddlewareConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let middleware_key = format!("{}/{}", base_key, self.name);

        if let Some(headers) = &self.headers {
            // Handle header configurations
            self.add_header_pairs(&middleware_key, headers, &mut pairs)?;
        }

        if let Some(pass_through) = self.pass_through {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/headers/customRequestHeaders/X-Pass-Through",
                    middleware_key
                ),
                pass_through.to_string(),
            ));
        }

        Ok(pairs)
    }
}

impl HostConfig {
    pub fn validate(&self) -> TraefikResult<()> {
        // Validate domain
        if self.domain.is_empty() {
            return Err(TraefikError::ConfigError(
                "Domain cannot be empty".to_string(),
            ));
        }

        // Validate root deployments
        self.validate_deployments(&self.deployments, "root")?;

        // Validate paths
        let mut path_set = HashSet::new();
        for path in &self.paths {
            // Validate path format
            if !path.path.starts_with('/') {
                return Err(TraefikError::PathConfig(format!(
                    "Path '{}' must start with '/'",
                    path.path
                )));
            }

            // Check for duplicate paths
            if !path_set.insert(&path.path) {
                return Err(TraefikError::PathConfig(format!(
                    "Duplicate path '{}'",
                    path.path
                )));
            }

            // Validate path deployments
            self.validate_deployments(&path.deployments, &path.path)?;
        }

        Ok(())
    }

    fn validate_deployments(
        &self,
        deployments: &HashMap<String, DeploymentConfig>,
        context: &str,
    ) -> TraefikResult<()> {
        if deployments.is_empty() {
            return Err(TraefikError::DeploymentError(format!(
                "No deployments defined for {}",
                context
            )));
        }

        let total_weight: u8 = deployments.values().map(|d| d.weight).sum();
        if total_weight > 0 && total_weight != 100 {
            return Err(TraefikError::DeploymentWeight(format!(
                "Deployment weights for {} must sum to 100, got {}",
                context, total_weight
            )));
        }

        for (color, deployment) in deployments {
            if deployment.port == 0 {
                return Err(TraefikError::DeploymentError(format!(
                    "Invalid port 0 for deployment {} in {}",
                    color, context
                )));
            }
        }

        Ok(())
    }

    fn add_host_headers(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        safe_name: &str,
    ) -> TraefikResult<()> {
        // Add custom request headers
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Forwarded-Proto",
                base_key, safe_name
            ),
            "https".to_string(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Forwarded-Host",
                base_key, safe_name
            ),
            self.domain.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Original-Host",
                base_key, safe_name
            ),
            self.domain.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Real-IP",
                base_key, safe_name
            ),
            "true".to_string(),
        ));

        if self.pass_through {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Pass-Through",
                    base_key, safe_name
                ),
                "true".to_string(),
            ));
        }

        Ok(())
    }

    fn add_middlewares(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        router_name: &str,
        additional_middlewares: &[String],
        strip_prefix_name: Option<&str>,
    ) -> TraefikResult<()> {
        let mut middleware_idx = 0;

        // Always add headers middleware first
        pairs.push(EtcdPair::new(
            format!(
                "{}/routers/{}/middlewares/{}",
                base_key, router_name, middleware_idx
            ),
            format!("{}-headers", router_name),
        ));
        middleware_idx += 1;

        // Add strip prefix if provided
        if let Some(strip_name) = strip_prefix_name {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/routers/{}/middlewares/{}",
                    base_key, router_name, middleware_idx
                ),
                strip_name.to_string(),
            ));
            middleware_idx += 1;
        }

        // Add redirect handler if not in pass-through mode
        if !self.pass_through {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/routers/{}/middlewares/{}",
                    base_key, router_name, middleware_idx
                ),
                "redirect-handler@file".to_string(),
            ));
            middleware_idx += 1;
        }

        // Add additional middlewares
        for middleware in additional_middlewares {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/routers/{}/middlewares/{}",
                    base_key, router_name, middleware_idx
                ),
                middleware.clone(),
            ));
            middleware_idx += 1;
        }

        Ok(())
    }

    pub async fn clean_etcd(&self, etcd: &mut Etcd) -> TraefikResult<()> {
        let safe_name = format!("host-{}", get_safe_key(&self.domain));
        let base_key = "traefik/http";

        // Delete root configuration
        etcd.delete_with_prefix(format!("{}/routers/{}", base_key, safe_name))
            .await
            .map_err(|e| TraefikError::EtcdError(e.into()))?;

        etcd.delete_with_prefix(format!("{}/services/{}", base_key, safe_name))
            .await
            .map_err(|e| TraefikError::EtcdError(e.into()))?;

        // Delete path-specific configurations
        for (idx, _) in self.paths.iter().enumerate() {
            let path_safe_name = format!("{}-path-{}", safe_name, idx);

            // Clean up path routers
            etcd.delete_with_prefix(format!("{}/routers/{}", base_key, path_safe_name))
                .await
                .map_err(|e| TraefikError::EtcdError(e.into()))?;

            // Clean up path services
            etcd.delete_with_prefix(format!("{}/services/{}", base_key, path_safe_name))
                .await
                .map_err(|e| TraefikError::EtcdError(e.into()))?;

            // Clean up path middlewares (strip prefix)
            etcd.delete_with_prefix(format!("{}/middlewares/{}-strip", base_key, path_safe_name))
                .await
                .map_err(|e| TraefikError::EtcdError(e.into()))?;
        }

        Ok(())
    }

    fn add_deployment_pairs(
        &self,
        pairs: &mut Vec<EtcdPair>,
        safe_name: &str,
        base_key: &str,
        _path: &str,
        deployments: &HashMap<String, DeploymentConfig>,
    ) -> TraefikResult<()> {
        // Set up services for each deployment
        for (color, deployment) in deployments {
            let service_name = format!("{}-{}", safe_name, color);

            // Basic URL configuration
            pairs.push(EtcdPair::new(
                format!(
                    "{}/services/{}/loadBalancer/servers/0/url",
                    base_key, service_name
                ),
                format!("http://{}:{}", deployment.ip, deployment.port),
            ));

            // Add passHostHeader configuration
            pairs.push(EtcdPair::new(
                format!(
                    "{}/services/{}/loadBalancer/passHostHeader",
                    base_key, service_name
                ),
                "true".to_string(),
            ));

            // Add response forwarding flush interval
            pairs.push(EtcdPair::new(
                format!(
                    "{}/services/{}/loadBalancer/responseForwarding/flushInterval",
                    base_key, service_name
                ),
                "100ms".to_string(),
            ));
        }

        // Calculate active deployments (weight > 0)
        let active_deployments: Vec<_> = deployments.iter().filter(|(_, d)| d.weight > 0).collect();

        // Set up weighted service if we have multiple active deployments
        if active_deployments.len() > 1 {
            let weighted_name = format!("{}-weighted", safe_name);

            // Add loadBalancer configurations for weighted service
            pairs.push(EtcdPair::new(
                format!(
                    "{}/services/{}/loadBalancer/passHostHeader",
                    base_key, weighted_name
                ),
                "true".to_string(),
            ));

            pairs.push(EtcdPair::new(
                format!(
                    "{}/services/{}/loadBalancer/responseForwarding/flushInterval",
                    base_key, weighted_name
                ),
                "100ms".to_string(),
            ));

            for (idx, (color, deployment)) in active_deployments.into_iter().enumerate() {
                // Add weighted service configuration
                pairs.push(EtcdPair::new(
                    format!(
                        "{}/services/{}/weighted/services/{}/name",
                        base_key, weighted_name, idx
                    ),
                    format!("{}@internal", format!("{}-{}", safe_name, color)),
                ));
                pairs.push(EtcdPair::new(
                    format!(
                        "{}/services/{}/weighted/services/{}/weight",
                        base_key, weighted_name, idx
                    ),
                    deployment.weight.to_string(),
                ));
            }

            // Set router service to weighted service
            pairs.push(EtcdPair::new(
                format!("{}/routers/{}/service", base_key, safe_name),
                weighted_name,
            ));
        } else {
            // Single deployment - use it directly
            if let Some((color, _)) = deployments.iter().next() {
                pairs.push(EtcdPair::new(
                    format!("{}/routers/{}/service", base_key, safe_name),
                    format!("{}-{}", safe_name, color),
                ));
            }
        }

        Ok(())
    }
}

impl ToEtcdPairs for HostConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let safe_name = format!("host-{}", get_safe_key(&self.domain));

        // Add custom request headers middleware for root
        self.add_host_headers(&mut pairs, base_key, &safe_name)?;

        // Set up root path router and service
        self.add_deployment_pairs(&mut pairs, &safe_name, base_key, "", &self.deployments)?;

        // Add middlewares for root path
        self.add_middlewares(&mut pairs, base_key, &safe_name, &self.middlewares, None)?;

        // Root router configuration
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/rule", base_key, safe_name),
            format!("Host(`{}`)", self.domain),
        ));
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/entrypoints/0", base_key, safe_name),
            "websecure",
        ));
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/tls", base_key, safe_name),
            "true",
        ));

        // Set up path-specific routes
        for (idx, path_config) in self.paths.iter().enumerate() {
            let path_safe_name = format!("{}-path-{}", safe_name, idx);

            // Add custom request headers middleware for path
            self.add_host_headers(&mut pairs, base_key, &path_safe_name)?;

            let path_rule = format!(
                "Host(`{}`) && PathPrefix(`{}`)",
                self.domain, path_config.path
            );

            // Router configuration
            pairs.push(EtcdPair::new(
                format!("{}/routers/{}/rule", base_key, path_safe_name),
                path_rule,
            ));
            pairs.push(EtcdPair::new(
                format!("{}/routers/{}/entrypoints/0", base_key, path_safe_name),
                "websecure",
            ));
            pairs.push(EtcdPair::new(
                format!("{}/routers/{}/tls", base_key, path_safe_name),
                "true",
            ));

            // Add strip prefix middleware if enabled
            let strip_prefix_name = if path_config.strip_prefix {
                pairs.push(EtcdPair::new(
                    format!(
                        "{}/middlewares/{}-strip/stripPrefix/prefixes/0",
                        base_key, path_safe_name
                    ),
                    path_config.path.clone(),
                ));
                Some(format!("{}-strip", path_safe_name))
            } else {
                None
            };

            // Add middlewares for path
            self.add_middlewares(
                &mut pairs,
                base_key,
                &path_safe_name,
                &path_config.middlewares,
                strip_prefix_name.as_deref(),
            )?;

            // Set up deployments for this path
            self.add_deployment_pairs(
                &mut pairs,
                &path_safe_name,
                base_key,
                &path_config.path,
                &path_config.deployments,
            )?;
        }

        Ok(pairs)
    }
}

impl MiddlewareConfig {
    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn add_header_pairs(
        &self,
        middleware_key: &str,
        headers: &HeadersConfig,
        pairs: &mut Vec<EtcdPair>,
    ) -> TraefikResult<()> {
        // Helper function to convert header keys to PascalCase for etcd
        fn format_header_key(key: &str) -> String {
            key.to_case(Case::Pascal)
        }

        // Helper function to format list values
        fn format_list_value(values: &[String]) -> String {
            values.join(", ")
        }

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

impl MiddlewareConfig {
    pub fn validate(&self) -> TraefikResult<()> {
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

impl HeadersConfig {
    pub fn validate(&self) -> TraefikResult<()> {
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

    fn validate_header_names(&self, headers: &HashMap<String, String>) -> TraefikResult<()> {
        for name in headers.keys() {
            if name.is_empty() {
                return Err(TraefikError::MiddlewareConfig(
                    "Header name cannot be empty".into(),
                ));
            }

            // Check for valid characters in header name
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
        // Check for duplicates (case-insensitive)
        let mut seen = HashSet::new();
        for header in headers {
            let header_lower = header.to_lowercase();
            if !seen.insert(header_lower) {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Duplicate header in {}: {}",
                    context, header
                )));
            }

            // Validate individual header name
            if header.is_empty() {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Empty header name in {}",
                    context
                )));
            }

            if !header
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
            {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Invalid header name '{}' in {}: must contain only alphanumeric characters, hyphens, underscores, dots, or forward slashes",
                    header, context
                )));
            }
        }
        Ok(())
    }

    fn validate_header_value(&self, name: &str, value: &str) -> TraefikResult<()> {
        // Check for control characters
        if value.chars().any(|c| c.is_control() && c != '\t') {
            return Err(TraefikError::MiddlewareConfig(format!(
                "Invalid value for header '{}': contains control characters",
                name
            )));
        }

        // Special validations for specific headers
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

impl RedirectorConfig {
    pub fn validate(&self) -> TraefikResult<()> {
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
    use super::*;

    fn create_test_middleware() -> HashMap<String, MiddlewareConfig> {
        HashMap::from([
            (
                "enable-headers".to_string(),
                MiddlewareConfig {
                    name: "enable-headers".to_string(),
                    headers: Some(HeadersConfig {
                        custom_request_headers: HashMap::from([
                            ("X-Forwarded-Proto".to_string(), "https".to_string()),
                            ("X-Forwarded-Port".to_string(), "443".to_string()),
                        ]),
                        custom_response_headers: HashMap::from([(
                            "Location".to_string(),
                            "".to_string(),
                        )]),
                        access_control_allow_methods: vec![
                            "GET".to_string(),
                            "POST".to_string(),
                            "OPTIONS".to_string(),
                        ],
                        access_control_allow_headers: vec![
                            "Content-Type".to_string(),
                            "Authorization".to_string(),
                        ],
                        access_control_allow_origin_list: vec![],
                        access_control_expose_headers: vec!["Location".to_string()],
                        add_vary_header: true,
                    }),
                    pass_through: None,
                },
            ),
            (
                "handle-redirects".to_string(),
                MiddlewareConfig {
                    name: "handle-redirects".to_string(),
                    headers: Some(HeadersConfig {
                        custom_request_headers: HashMap::from([(
                            "Location".to_string(),
                            "".to_string(),
                        )]),
                        ..Default::default()
                    }),
                    pass_through: Some(true),
                },
            ),
        ])
    }

    fn create_test_config() -> TraefikConfig {
        TraefikConfig {
            etcd: Default::default(),
            middlewares: create_test_middleware(),
            redirector: RedirectorConfig::default(),
            www_redirect: Some(true),
            hosts: vec![HostConfig {
                domain: "test1.example.com".to_string(),
                pass_through: false,
                paths: vec![PathConfig {
                    path: "/api".to_string(),
                    deployments: HashMap::from([(
                        "blue".to_string(),
                        DeploymentConfig {
                            ip: "10.0.0.1".to_string(),
                            port: 8080,
                            weight: 100,
                        },
                    )]),
                    middlewares: vec!["enable-headers".to_string()],
                    strip_prefix: true,
                }],
                deployments: HashMap::from([(
                    "blue".to_string(),
                    DeploymentConfig {
                        ip: "10.0.0.1".to_string(),
                        port: 80,
                        weight: 100,
                    },
                )]),
                middlewares: vec!["enable-headers".to_string()],
            }],
        }
    }

    #[test]
    fn test_validate_valid_config() {
        let mut config = create_test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_middleware() {
        let mut config = create_test_config();
        config.hosts[0].middlewares.push("non-existent".to_string());

        match config.validate() {
            Err(TraefikError::MiddlewareConfig(msg)) => {
                assert!(msg.contains("Undefined middleware"));
            }
            _ => panic!("Expected undefined middleware error"),
        }
    }
    #[test]
    fn test_validate_invalid_path_format() {
        let mut config = create_test_config();
        let invalid_path = PathConfig {
            path: "invalid-path".to_string(), // Missing leading slash
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 8080,
                    weight: 100,
                },
            )]),
            middlewares: vec![],
            strip_prefix: true,
        };
        config.hosts[0].paths.push(invalid_path);

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(TraefikError::PathConfig(msg)) => {
                assert!(
                    msg.contains("must start with '/'"),
                    "Error message was: {}",
                    msg
                );
            }
            err => panic!("Expected PathConfig error, got {:?}", err),
        }
    }

    #[test]
    fn test_validate_duplicate_paths() {
        let mut config = create_test_config();
        let duplicate_path = PathConfig {
            path: "/api".to_string(),
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 8080,
                    weight: 100,
                },
            )]),
            middlewares: vec![],
            strip_prefix: true,
        };
        config.hosts[0].paths.push(duplicate_path);

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(TraefikError::PathConfig(msg)) => {
                assert!(msg.contains("Duplicate path"), "Error message was: {}", msg);
            }
            err => panic!("Expected PathConfig error, got {:?}", err),
        }
    }

    #[test]
    fn test_validate_duplicate_domains() {
        let mut config = create_test_config();
        let duplicate_host = HostConfig {
            domain: "test1.example.com".to_string(), // Same as first host
            paths: vec![],
            pass_through: false,
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 80,
                    weight: 100,
                },
            )]),
            middlewares: vec!["enable-headers".to_string()],
        };
        config.hosts.push(duplicate_host);

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(TraefikError::ConfigError(msg)) => {
                assert!(
                    msg.contains("Duplicate domain"),
                    "Error message was: {}",
                    msg
                );
            }
            err => panic!("Expected ConfigError error, got {:?}", err),
        }
    }

    #[test]
    fn test_validate_empty_domain() {
        let mut config = create_test_config();
        config.hosts[0].domain = "".to_string();

        match config.validate() {
            Err(TraefikError::ConfigError(msg)) => {
                assert!(msg.contains("Domain cannot be empty"));
            }
            _ => panic!("Expected empty domain error"),
        }
    }

    #[test]
    fn test_validate_invalid_deployment_weights() {
        let mut config = create_test_config();
        config.hosts[0].deployments.insert(
            "green".to_string(),
            DeploymentConfig {
                ip: "10.0.0.2".to_string(),
                port: 80,
                weight: 50,
            },
        );

        match config.validate() {
            Err(TraefikError::DeploymentWeight(msg)) => {
                assert!(msg.contains("must sum to 100"));
            }
            _ => panic!("Expected invalid weight error"),
        }
    }

    #[test]
    fn test_validate_empty_deployments() {
        let mut config = create_test_config();
        config.hosts[0].deployments.clear();

        match config.validate() {
            Err(TraefikError::DeploymentError(msg)) => {
                assert!(msg.contains("No deployments defined"));
            }
            _ => panic!("Expected empty deployments error"),
        }
    }

    #[test]
    fn test_validate_invalid_port() {
        let mut config = create_test_config();
        config.hosts[0].deployments.get_mut("blue").unwrap().port = 0;

        match config.validate() {
            Err(TraefikError::DeploymentError(msg)) => {
                assert!(msg.contains("Invalid port"));
            }
            _ => panic!("Expected invalid port error"),
        }
    }

    #[test]
    fn test_path_strip_prefix_generation() {
        let config = create_test_config();
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify strip prefix middleware is created
        let has_strip_prefix = pairs
            .iter()
            .any(|p| p.key().contains("stripPrefix") && p.value() == "/api");
        assert!(has_strip_prefix, "Strip prefix middleware not found");
    }

    #[test]
    fn test_middleware_ordering() {
        let config = create_test_config();
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Find middleware entries for the path
        let middleware_keys: Vec<_> = pairs
            .iter()
            .filter(|p| p.key().contains("middlewares") && p.key().contains("path"))
            .collect();

        // Verify strip-prefix comes before other middlewares
        let strip_prefix_idx = middleware_keys
            .iter()
            .position(|p| p.value().contains("strip"));
        let other_middleware_idx = middleware_keys
            .iter()
            .position(|p| p.value().contains("enable-headers"));

        match (strip_prefix_idx, other_middleware_idx) {
            (Some(strip_idx), Some(other_idx)) => {
                assert!(
                    strip_idx < other_idx,
                    "Strip prefix should come before other middlewares"
                );
            }
            _ => panic!("Expected both strip prefix and other middlewares"),
        }
    }

    #[test]
    fn test_weighted_service_generation() {
        let mut config = create_test_config();
        // Add a green deployment with weight
        config.hosts[0].deployments.insert(
            "green".to_string(),
            DeploymentConfig {
                ip: "10.0.0.2".to_string(),
                port: 80,
                weight: 20,
            },
        );
        config.hosts[0].deployments.get_mut("blue").unwrap().weight = 80;

        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify weighted service configuration
        let has_weighted_service = pairs.iter().any(|p| {
            p.key().contains("weighted/services")
                && (p.value().contains("weight") || p.value() == "80" || p.value() == "20")
        });
        assert!(
            has_weighted_service,
            "Weighted service configuration not found"
        );
    }

    #[test]
    fn test_host_headers_generation() {
        let config = create_test_config();
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify host headers are present
        let has_forwarded_proto = pairs.iter().any(|p| {
            p.key().contains("customRequestHeaders/X-Forwarded-Proto") && p.value() == "https"
        });
        assert!(has_forwarded_proto, "X-Forwarded-Proto header not found");

        let has_forwarded_host = pairs.iter().any(|p| {
            p.key().contains("customRequestHeaders/X-Forwarded-Host")
                && p.value() == "test1.example.com"
        });
        assert!(has_forwarded_host, "X-Forwarded-Host header not found");

        let has_original_host = pairs.iter().any(|p| {
            p.key().contains("customRequestHeaders/X-Original-Host")
                && p.value() == "test1.example.com"
        });
        assert!(has_original_host, "X-Original-Host header not found");

        let has_real_ip = pairs
            .iter()
            .any(|p| p.key().contains("customRequestHeaders/X-Real-IP") && p.value() == "true");
        assert!(has_real_ip, "X-Real-IP header not found");
    }

    #[test]
    fn test_pass_through_configuration() {
        let host_config = HostConfig {
            domain: "test.example.com".to_string(),
            paths: vec![],
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 80,
                    weight: 100,
                },
            )]),
            middlewares: vec![],
            pass_through: true,
        };

        let pairs = host_config.to_etcd_pairs("traefik/http").unwrap();

        // Verify X-Pass-Through header is present
        let has_pass_through = pairs.iter().any(|p| {
            p.key().contains("customRequestHeaders/X-Pass-Through") && p.value() == "true"
        });
        assert!(has_pass_through, "X-Pass-Through header not found");

        // Verify redirect handler is NOT present
        let has_redirect_handler = pairs.iter().any(|p| p.value() == "redirect-handler@file");
        assert!(
            !has_redirect_handler,
            "Redirect handler should not be present"
        );
    }

    #[test]
    fn test_loadbalancer_configuration() {
        let host_config = HostConfig {
            domain: "test.example.com".to_string(),
            paths: vec![],
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 80,
                    weight: 100,
                },
            )]),
            middlewares: vec![],
            pass_through: false,
        };

        let pairs = host_config.to_etcd_pairs("traefik/http").unwrap();

        // Verify passHostHeader is set
        let has_pass_host_header = pairs
            .iter()
            .any(|p| p.key().contains("/loadBalancer/passHostHeader") && p.value() == "true");
        assert!(has_pass_host_header, "passHostHeader not found");

        // Verify flushInterval is set
        let has_flush_interval = pairs.iter().any(|p| {
            p.key()
                .contains("/loadBalancer/responseForwarding/flushInterval")
                && p.value() == "100ms"
        });
        assert!(has_flush_interval, "flushInterval not found");
    }

    #[test]
    fn test_weighted_loadbalancer_configuration() {
        let host_config = HostConfig {
            domain: "test.example.com".to_string(),
            paths: vec![],
            deployments: HashMap::from([
                (
                    "blue".to_string(),
                    DeploymentConfig {
                        ip: "10.0.0.1".to_string(),
                        port: 80,
                        weight: 80,
                    },
                ),
                (
                    "green".to_string(),
                    DeploymentConfig {
                        ip: "10.0.0.2".to_string(),
                        port: 80,
                        weight: 20,
                    },
                ),
            ]),
            middlewares: vec![],
            pass_through: false,
        };

        let pairs = host_config.to_etcd_pairs("traefik/http").unwrap();

        // Verify weighted service has loadBalancer configs
        let weighted_service_name = format!("host-{}-weighted", get_safe_key(&host_config.domain));

        let has_weighted_pass_host_header = pairs.iter().any(|p| {
            p.key().contains(&format!(
                "/services/{}/loadBalancer/passHostHeader",
                weighted_service_name
            )) && p.value() == "true"
        });
        assert!(
            has_weighted_pass_host_header,
            "Weighted service passHostHeader not found"
        );

        let has_weighted_flush_interval = pairs.iter().any(|p| {
            p.key().contains(&format!(
                "/services/{}/loadBalancer/responseForwarding/flushInterval",
                weighted_service_name
            )) && p.value() == "100ms"
        });
        assert!(
            has_weighted_flush_interval,
            "Weighted service flushInterval not found"
        );
    }

    #[test]
    fn test_service_url_configuration() {
        let host_config = HostConfig {
            domain: "test.example.com".to_string(),
            paths: vec![],
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "redirector".to_string(),
                    port: 3000,
                    weight: 100,
                },
            )]),
            middlewares: vec![],
            pass_through: false,
        };

        let pairs = host_config.to_etcd_pairs("traefik/http").unwrap();

        // Verify service URL is correct
        let has_correct_url = pairs.iter().any(|p| {
            p.key().contains("/loadBalancer/servers/0/url") && p.value() == "http://redirector:3000"
        });
        assert!(has_correct_url, "Service URL not configured correctly");
    }
    #[test]
    fn test_www_redirect_configuration() {
        let config = TraefikConfig {
            etcd: Default::default(),
            middlewares: HashMap::new(),
            redirector: RedirectorConfig::default(),
            hosts: vec![HostConfig {
                domain: "example.com".to_string(),
                paths: vec![],
                deployments: HashMap::from([(
                    "blue".to_string(),
                    DeploymentConfig {
                        ip: "10.0.0.1".to_string(),
                        port: 80,
                        weight: 100,
                    },
                )]),
                middlewares: vec![],
                pass_through: false,
            }],
            www_redirect: Some(true),
        };

        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify middleware configuration
        let has_regex = pairs.iter().any(|p| {
            p.key().contains("/middlewares/add-www/redirectregex/regex")
                && p.value() == "^https://([^.]+\\.[^.]+\\.[^.]+)(.*)"
        });
        assert!(has_regex, "WWW redirect regex not found");

        let has_replacement = pairs.iter().any(|p| {
            p.key()
                .contains("/middlewares/add-www/redirectregex/replacement")
                && p.value() == "https://www.${1}${2}"
        });
        assert!(has_replacement, "WWW redirect replacement not found");

        // Verify router configuration
        let has_router = pairs.iter().any(|p| {
            p.key().contains("/routers/to-www-example-com/rule")
                && p.value() == "Host(`example.com`)"
        });
        assert!(has_router, "WWW redirect router not found");

        let has_priority = pairs.iter().any(|p| {
            p.key().contains("/routers/to-www-example-com/priority") && p.value() == "200"
        });
        assert!(has_priority, "WWW redirect router priority not found");
    }

    #[test]
    fn test_www_redirect_not_applied_to_www_domains() {
        let config = TraefikConfig {
            etcd: Default::default(),
            middlewares: HashMap::new(),
            redirector: RedirectorConfig::default(),
            hosts: vec![HostConfig {
                domain: "www.example.com".to_string(),
                paths: vec![],
                deployments: HashMap::from([(
                    "blue".to_string(),
                    DeploymentConfig {
                        ip: "10.0.0.1".to_string(),
                        port: 80,
                        weight: 100,
                    },
                )]),
                middlewares: vec![],
                pass_through: false,
            }],
            www_redirect: Some(true),
        };

        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify no redirect router for www domains
        let has_redirect_router = pairs
            .iter()
            .any(|p| p.key().contains("/routers/to-www-") && p.value().contains("www.example.com"));
        assert!(
            !has_redirect_router,
            "WWW redirect router should not exist for www domains"
        );
    }

    #[test]
    fn test_www_redirect_disabled() {
        let config = TraefikConfig {
            etcd: Default::default(),
            middlewares: HashMap::new(),
            redirector: RedirectorConfig::default(),
            hosts: vec![HostConfig {
                domain: "example.com".to_string(),
                paths: vec![],
                deployments: HashMap::from([(
                    "blue".to_string(),
                    DeploymentConfig {
                        ip: "10.0.0.1".to_string(),
                        port: 80,
                        weight: 100,
                    },
                )]),
                middlewares: vec![],
                pass_through: false,
            }],
            www_redirect: Some(false),
        };

        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify no redirect configuration when disabled
        let has_redirect_config = pairs
            .iter()
            .any(|p| p.key().contains("/middlewares/add-www/"));
        assert!(
            !has_redirect_config,
            "WWW redirect middleware should not exist when disabled"
        );

        let has_redirect_router = pairs.iter().any(|p| p.key().contains("/routers/to-www-"));
        assert!(
            !has_redirect_router,
            "WWW redirect router should not exist when disabled"
        );
    }

    #[test]
    fn test_redirector_service_configuration() {
        let config = TraefikConfig {
            etcd: Default::default(),
            middlewares: HashMap::new(),
            hosts: vec![],
            www_redirect: None,
            redirector: RedirectorConfig {
                url: "http://redirector:3000".to_string(),
                health_check: HealthCheckConfig {
                    path: "/health".to_string(),
                    interval: "10s".to_string(),
                    timeout: "5s".to_string(),
                },
            },
        };

        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify service URL
        let has_service_url = pairs.iter().any(|p| {
            p.key()
                .contains("/services/redirector/loadBalancer/servers/0/url")
                && p.value() == "http://redirector:3000"
        });
        assert!(has_service_url, "Redirector service URL not found");

        // Verify passHostHeader
        let has_pass_host_header = pairs.iter().any(|p| {
            p.key()
                .contains("/services/redirector/loadBalancer/passHostHeader")
                && p.value() == "true"
        });
        assert!(has_pass_host_header, "passHostHeader not found");

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
        let valid_config = RedirectorConfig {
            url: "http://redirector:3000".to_string(),
            health_check: HealthCheckConfig {
                path: "/health".to_string(),
                interval: "10s".to_string(),
                timeout: "5s".to_string(),
            },
        };
        assert!(valid_config.validate().is_ok());

        // Test invalid URL
        let invalid_url = RedirectorConfig {
            url: "redirector:3000".to_string(),
            ..valid_config.clone()
        };
        assert!(invalid_url.validate().is_err());

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
            ..valid_config
        };
        assert!(invalid_interval.validate().is_err());
    }
}
