use ordermap::OrderMap;
use tracing::debug;

use crate::{
    config::core_traits::{EtcdPair, ToEtcdPairs, Validate},
    error::{TraefikError, TraefikResult},
    etcd::{util::get_safe_key, Etcd},
};
use std::collections::{HashMap, HashSet};

use super::{
    base_structs::{DeploymentConfig, HostConfig, PathConfig},
    util::add_selection_rules,
    InternalDeploymentConfig, RuleConfig,
};

impl ToEtcdPairs for HostConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let safe_name = format!("host-{}", get_safe_key(&self.domain));

        let mut rule = self.get_host_rule();

        add_selection_rules(self, &mut rule);

        // Add custom request headers middleware for root
        self.add_host_headers(&mut pairs, base_key, &safe_name)?;

        // Set up root path router and service
        self.add_deployment_pairs(
            &mut pairs,
            &safe_name,
            base_key,
            "",
            &self.deployments,
            &mut rule,
        )?;

        // Add middlewares for root path
        self.add_middlewares(&mut pairs, base_key, &safe_name, &self.middlewares, None)?;

        // Root router configuration
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
            self.add_path_configuration(&mut pairs, base_key, &safe_name, idx, path_config)?;
        }

        Ok(pairs)
    }
}

// Host configuration methods
impl HostConfig {
    pub fn get_host_rule(&self) -> RuleConfig {
        let mut rule = RuleConfig::default();
        rule.add_host_rule(&self.domain);

        add_selection_rules(self, &mut rule);
        rule
    }

    // When sorting hosts, we sort by the number of rules they have
    pub fn get_host_weight(&self) -> usize {
        self.get_host_rule().get_weight()
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

        // Add headers middleware first
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

    fn add_path_configuration(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        safe_name: &str,
        idx: usize,
        path_config: &PathConfig,
    ) -> TraefikResult<()> {
        let path_safe_name = format!("{}-path-{}", safe_name, idx);

        // Router configuration
        let mut rule = RuleConfig::default();
        rule.add_host_rule(&self.domain);
        rule.add_default_rule("PathPrefix", &path_config.path);

        self.add_root_router(pairs, base_key, &path_safe_name, &rule)?;

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

        if path_config.pass_through {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Pass-Through",
                    base_key, safe_name
                ),
                "true".to_string(),
            ));
        } else {
            pairs.push(EtcdPair::new(
                format!(
                    "{}/middlewares/{}-headers/headers/customResponseHeaders/Location",
                    base_key, ""
                ),
                "false".to_string(),
            ));
        }

        // Add middlewares for path
        self.add_middlewares(
            pairs,
            base_key,
            &path_safe_name,
            &path_config.middlewares,
            strip_prefix_name.as_deref(),
        )?;

        // Set up deployments for this path
        self.add_deployment_pairs(
            pairs,
            &path_safe_name,
            base_key,
            &path_config.path,
            &path_config.deployments,
            &mut rule,
        )?;

        Ok(())
    }

    fn add_deployment_pairs(
        &self,
        pairs: &mut Vec<EtcdPair>,
        safe_name: &str,
        base_key: &str,
        _path: &str,
        deployments: &HashMap<String, DeploymentConfig>,
        rule: &mut RuleConfig,
    ) -> TraefikResult<()> {
        let sorted_deployments = self.get_sorted_deployments(deployments);
        for (idx, (color, deployment)) in sorted_deployments.into_iter().enumerate() {
            let service_name = format!("{}-{}-{}", safe_name, color, idx);

            if deployment.weight > 0 {
                add_selection_rules(&deployment, rule);
                // Always create the service
                self.add_base_service_configuration(pairs, base_key, &service_name, &deployment)?;

                // Only add router and weighted configuration for deployments with weight
                self.add_root_router(pairs, base_key, safe_name, rule)?;
                self.add_weighted_service_configuration(
                    pairs,
                    base_key,
                    safe_name,
                    &service_name, // Pass the correct service name
                    deployments,
                    rule,
                )?;
            }
        }

        Ok(())
    }

    fn get_sorted_deployments(
        &self,
        deployments: &HashMap<String, DeploymentConfig>,
    ) -> OrderMap<String, DeploymentConfig> {
        let mut internal_deployments = Vec::new();
        for (name, deployment) in deployments {
            // Count number of rules for this deployment
            let mut rules = RuleConfig::default();
            add_selection_rules(deployment, &mut rules);
            internal_deployments.push(InternalDeploymentConfig {
                deployment: deployment.clone(),
                name: name.clone(),
                weight: rules.get_weight(),
            });
        }
        // Sort by number of rules (descending)
        internal_deployments.sort_by_key(|d| d.weight);
        internal_deployments.reverse();

        internal_deployments
            .into_iter()
            .map(|d| (d.name, d.deployment))
            .collect()
    }

    fn add_base_service_configuration(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        service_name: &str,
        deployment: &DeploymentConfig,
    ) -> TraefikResult<()> {
        debug!("Adding base service configuration for {}", service_name);
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

        // Add response forwarding configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/{}/loadBalancer/responseForwarding/flushInterval",
                base_key, service_name
            ),
            "100ms".to_string(),
        ));
        Ok(())
    }

    fn add_root_router(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        safe_name: &str,
        rule: &RuleConfig,
    ) -> TraefikResult<()> {
        debug!("Adding root router for {}", safe_name);
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/rule", base_key, safe_name),
            rule.rule_str(),
        ));
        debug!("Added rule: {}", rule.rule_str());
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/entrypoints/0", base_key, safe_name),
            "websecure",
        ));
        debug!("Added entrypoint: websecure");
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/tls", base_key, safe_name),
            "true",
        ));
        debug!("Added tls: true");
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/{}/loadBalancer/passHostHeader",
                base_key, safe_name
            ),
            "true".to_string(),
        ));
        debug!("Added passHostHeader: true");
        Ok(())
    }

    fn add_weighted_service_configuration(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        safe_name: &str,
        service_name: &str,
        deployments: &HashMap<String, DeploymentConfig>,
        _rule: &mut RuleConfig,
    ) -> TraefikResult<()> {
        let active_deployments: Vec<_> = deployments.iter().filter(|(_, d)| d.weight > 0).collect();

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
            debug!("Added passHostHeader: true");

            pairs.push(EtcdPair::new(
                format!(
                    "{}/services/{}/loadBalancer/responseForwarding/flushInterval",
                    base_key, weighted_name
                ),
                "100ms".to_string(),
            ));
            debug!("Added flushInterval: 100ms");

            // Add weighted service entries
            for (idx, (color, deployment)) in active_deployments.into_iter().enumerate() {
                let weighted_service_name = format!("{}-{}-{}", safe_name, color, idx);
                self.add_base_service_configuration(
                    pairs,
                    base_key,
                    &weighted_service_name,
                    deployment,
                )?;

                pairs.push(EtcdPair::new(
                    format!(
                        "{}/services/{}/weighted/services/{}/name",
                        base_key, weighted_name, idx
                    ),
                    format!("{}@internal", weighted_service_name),
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
        } else if let Some((_color, _)) = deployments.iter().next() {
            // Single deployment - use it directly
            pairs.push(EtcdPair::new(
                format!("{}/routers/{}/service", base_key, safe_name),
                service_name.to_string(), // Use the service name passed in
            ));
        }

        Ok(())
    }
}

impl Validate for HostConfig {
    fn validate(&self) -> TraefikResult<()> {
        // Base validation
        // Validate domain
        if self.domain.is_empty() {
            return Err(TraefikError::ConfigError(
                "Domain cannot be empty".to_string(),
            ));
        }

        // Domain format validation
        if !self.validate_domain_format()? {
            return Err(TraefikError::ConfigError(format!(
                "Invalid domain format: {}",
                self.domain
            )));
        }

        // Path validation
        self.validate_paths_extended()?;

        // Deployment validation
        self.validate_deployments_extended()?;

        // Middleware validation
        self.validate_middleware_references()?;

        Ok(())
    }
}

impl HostConfig {
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
}

impl HostConfig {
    fn validate_domain_format(&self) -> TraefikResult<bool> {
        // Basic domain validation rules
        if self.domain.is_empty() || self.domain.len() > 255 {
            return Ok(false);
        }

        // Check for valid characters and format
        let valid_chars = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '.';

        if !self.domain.chars().all(valid_chars) {
            return Ok(false);
        }

        // Check parts
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() < 2 {
            return Ok(false);
        }

        // Validate each part
        for part in parts {
            if part.is_empty() || part.len() > 63 {
                return Ok(false);
            }
            if part.starts_with('-') || part.ends_with('-') {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn validate_paths_extended(&self) -> TraefikResult<()> {
        for path in &self.paths {
            // Check path format
            if !path
                .path
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.')
            {
                return Err(TraefikError::PathConfig(format!(
                    "Invalid characters in path: {}",
                    path.path
                )));
            }

            // Check for proper nesting
            if path.path.contains("//") {
                return Err(TraefikError::PathConfig(format!(
                    "Invalid path nesting: {}",
                    path.path
                )));
            }

            // Validate middleware references
            for middleware in &path.middlewares {
                if middleware.is_empty() {
                    return Err(TraefikError::MiddlewareConfig(
                        "Empty middleware reference in path configuration".to_string(),
                    ));
                }
            }
            if !path.path.starts_with('/') {
                return Err(TraefikError::PathConfig(format!(
                    "Path '{}' must start with '/'",
                    path.path
                )));
            }

            // Check for duplicate paths
            let mut path_set = HashSet::new();
            if !path_set.insert(&path.path) {
                return Err(TraefikError::PathConfig(format!(
                    "Duplicate path '{}'",
                    path.path
                )));
            }
        }

        Ok(())
    }

    fn validate_deployments_extended(&self) -> TraefikResult<()> {
        self.validate_has_deployments()?;
        self.validate_deployment_weights()?;
        self.validate_deployment_ports()?;

        // Validate root deployments
        self.validate_deployment_config(&self.deployments, "root")?;

        // Validate path deployments
        for path in &self.paths {
            self.validate_deployment_config(&path.deployments, &path.path)?;
        }

        Ok(())
    }

    fn validate_has_deployments(&self) -> TraefikResult<()> {
        if self.deployments.is_empty() {
            return Err(TraefikError::DeploymentError(format!(
                "No deployments defined for {}",
                "root"
            )));
        }

        Ok(())
    }

    fn validate_deployment_weights(&self) -> TraefikResult<()> {
        let total_weight: u8 = self.deployments.values().map(|d| d.weight).sum();
        if total_weight > 0 && total_weight != 100 {
            return Err(TraefikError::DeploymentWeight(format!(
                "Deployment weights for {} must sum to 100, got {}",
                "root", total_weight
            )));
        }

        Ok(())
    }

    fn validate_deployment_ports(&self) -> TraefikResult<()> {
        for deployment in self.deployments.values() {
            if deployment.port == 0 {
                return Err(TraefikError::DeploymentError(
                    "Invalid port 0 for deployment".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn validate_deployment_config(
        &self,
        deployments: &HashMap<String, DeploymentConfig>,
        context: &str,
    ) -> TraefikResult<()> {
        for (color, deployment) in deployments {
            // Validate deployment name
            if !color
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Err(TraefikError::DeploymentError(format!(
                    "Invalid deployment name '{}' in {}",
                    color, context
                )));
            }

            // Validate IP format
            if !self.is_valid_ip_or_hostname(&deployment.ip) {
                return Err(TraefikError::DeploymentError(format!(
                    "Invalid IP or hostname '{}' in {} deployment {}",
                    deployment.ip, context, color
                )));
            }

            // Validate port range
            if !(1..=65535).contains(&deployment.port) {
                return Err(TraefikError::DeploymentError(format!(
                    "Invalid port {} in {} deployment {}",
                    deployment.port, context, color
                )));
            }
        }

        Ok(())
    }

    fn is_valid_ip_or_hostname(&self, host: &str) -> bool {
        // IP address validation
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() == 4 {
            return parts.iter().all(|part| {
                if let Ok(_num) = part.parse::<u8>() {
                    !part.is_empty() && part.len() <= 3
                } else {
                    false
                }
            });
        }

        // Hostname validation
        if host.is_empty() {
            return false;
        }

        // if host
        //     .chars()
        //     .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        // {
        //     let parts: Vec<&str> = host.split('.').collect();
        //     !parts.is_empty()
        //         && parts.iter().all(|part| {
        //             !part.is_empty()
        //                 && part.len() <= 63
        //                 && !part.starts_with('-')
        //                 && !part.ends_with('-')
        //         })
        self.validate_valid_hostname(host)
    }

    fn validate_valid_hostname(&self, hostname: &str) -> bool {
        fn is_valid_char(byte: u8) -> bool {
            (b'a'..=b'z').contains(&byte)
                || (b'A'..=b'Z').contains(&byte)
                || (b'0'..=b'9').contains(&byte)
                || byte == b'-'
                || byte == b'.'
        }

        !(hostname.bytes().any(|byte| !is_valid_char(byte))
            || hostname.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.starts_with('-')
                    || label.ends_with('-')
            })
            || hostname.is_empty()
            || hostname.len() > 253)
    }

    fn validate_middleware_references(&self) -> TraefikResult<()> {
        // Validate root middleware references
        for middleware in &self.middlewares {
            if middleware.is_empty() {
                return Err(TraefikError::MiddlewareConfig(
                    "Empty middleware reference in host configuration".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{
            base_structs::DeploymentConfig, FromClientIpConfig, SelectionConfig, WithCookieConfig,
        },
        test_helpers::{create_test_config, create_test_host},
    };

    #[test]
    fn test_host_basic_configuration() {
        let host = create_test_host();
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify basic router configuration
        let has_router = pairs.iter().any(|p| {
            p.key().contains("/routers/host-test-example-com/rule")
                && p.value() == "Host(`test.example.com`)"
        });
        assert!(has_router, "Basic router configuration not found");

        // Verify TLS configuration
        let has_tls = pairs
            .iter()
            .any(|p| p.key().contains("/tls") && p.value() == "true");
        assert!(has_tls, "TLS configuration not found");

        // Verify entrypoint
        let has_entrypoint = pairs
            .iter()
            .any(|p| p.key().contains("/entrypoints/0") && p.value() == "websecure");
        assert!(has_entrypoint, "Websecure entrypoint not found");
    }

    #[test]
    fn test_host_headers() {
        let host = create_test_host();
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify custom request headers
        let headers = [
            ("X-Forwarded-Proto", "https"),
            ("X-Forwarded-Host", "test.example.com"),
            ("X-Original-Host", "test.example.com"),
            ("X-Real-IP", "true"),
        ];

        for (header, value) in headers {
            let has_header = pairs
                .iter()
                .any(|p| p.key().contains(header) && p.value() == value);
            assert!(has_header, "Header {} not found or incorrect", header);
        }
    }

    #[test]
    fn test_path_configuration() {
        let host = create_test_host();
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify path router rule
        let has_path_rule = pairs.iter().any(|p| {
            p.key().contains("path-0/rule")
                && p.value().contains("Host(`test.example.com`)")
                && p.value().contains("PathPrefix(`/api`)")
        });
        assert!(has_path_rule, "Path router rule not found");

        // Verify strip prefix
        let has_strip_prefix = pairs
            .iter()
            .any(|p| p.key().contains("stripPrefix/prefixes/0") && p.value() == "/api");
        assert!(has_strip_prefix, "Strip prefix configuration not found");
    }

    #[test]
    fn test_path_configuration_with_cookie() {
        let mut host = create_test_host();
        host.selection = Some(SelectionConfig {
            with_cookie: Some(WithCookieConfig {
                name: "TEST_COOKIE".to_string(),
                value: Some("test_value".to_string()),
            }),
            ..Default::default()
        });
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify path router rule
        let has_path_rule = pairs.iter().any(|p| {
            p.key().contains("rule")
                && p.value()
                    .contains("HeaderRegexp(`Cookie`, `TEST_COOKIE=test_value`)")
        });
        assert!(has_path_rule, "Path router rule not found");

        // Verify strip prefix
        let has_strip_prefix = pairs
            .iter()
            .any(|p| p.key().contains("stripPrefix/prefixes/0") && p.value() == "/api");
        assert!(has_strip_prefix, "Strip prefix configuration not found");
    }

    #[test]
    fn test_path_configuration_with_cookie_regex() {
        let mut host = create_test_host();
        host.selection = Some(SelectionConfig {
            with_cookie: Some(WithCookieConfig {
                name: "TEST".to_string(),
                value: Some("true".to_string()),
            }),
            ..Default::default()
        });
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify path router rule
        let has_path_rule = pairs.iter().any(|p| {
            p.key().contains("rule") && p.value().contains("HeaderRegexp(`Cookie`, `TEST=true`)")
        });
        assert!(has_path_rule, "Path router rule not found");

        // Verify strip prefix
        let has_strip_prefix = pairs
            .iter()
            .any(|p| p.key().contains("stripPrefix/prefixes/0") && p.value() == "/api");
        assert!(has_strip_prefix, "Strip prefix configuration not found");
    }

    #[test]
    fn test_weighted_deployment() {
        let mut host = create_test_host();
        host.deployments.insert(
            "green".to_string(),
            DeploymentConfig {
                ip: "10.0.0.2".to_string(),
                port: 80,
                weight: 20,
                selection: None,
            },
        );
        host.deployments.get_mut("blue").unwrap().weight = 80;

        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify weighted service configuration
        let has_weighted_service = pairs.iter().any(|p| {
            p.key().contains("weighted/services")
                && (p.value().contains("weight") || p.value() == "80" || p.value() == "20")
        });
        assert!(
            has_weighted_service,
            "Weighted service configuration not found"
        );

        // Verify load balancer settings for weighted service
        let service_name = "host-test-example-com-weighted";
        let has_lb_config = pairs.iter().any(|p| {
            p.key().contains(&format!(
                "services/{}/loadBalancer/passHostHeader",
                service_name
            )) && p.value() == "true"
        });
        assert!(
            has_lb_config,
            "Load balancer configuration for weighted service not found"
        );
    }

    #[test]
    fn test_rule_weights() {
        let mut host_config = HostConfig::default();
        host_config.domain = "example.com".to_string();
        assert_eq!(host_config.get_host_weight(), 1); // Just Host rule

        // Add cookie
        host_config.selection = Some(SelectionConfig {
            with_cookie: Some(WithCookieConfig {
                name: "TEST".to_string(),
                value: Some("true".to_string()),
            }),
            ..Default::default()
        });
        assert_eq!(host_config.get_host_weight(), 2); // Host + Cookie rule
    }

    #[test]
    fn test_deployment_sorting() {
        let mut config = HostConfig::default();

        // Deployment with cookie (2 rules)
        config.deployments.insert(
            "blue".to_string(),
            DeploymentConfig {
                selection: Some(SelectionConfig {
                    with_cookie: Some(WithCookieConfig {
                        name: "TEST".to_string(),
                        value: Some("true".to_string()),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        // Deployment without cookie (1 rule)
        config
            .deployments
            .insert("green".to_string(), DeploymentConfig::default());

        let sorted = config.get_sorted_deployments(&config.deployments);
        let deployments: Vec<_> = sorted.keys().collect();
        assert_eq!(deployments, vec!["blue", "green"]);
    }

    #[test]
    fn test_pass_through_configuration() {
        let host = create_test_host();
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify X-Pass-Through header
        let has_pass_through = pairs
            .iter()
            .any(|p| p.key().contains("X-Pass-Through") && p.value() == "true");

        assert!(has_pass_through, "Pass-through header not found");
    }

    #[test]
    fn test_host_deployment_weight_sorting() {
        // let mut host_config_with_cookie = HostConfig::default();
        // host_config_with_cookie.deployments.insert(
        //     "blue".to_string(),
        //     DeploymentConfig {
        //         with_cookie: Some(WithCookieConfig {
        //             name: "BLUEGREEN".to_string(),
        //             value: Some("true".to_string()),
        //         }),
        //         ..Default::default()
        //     },
        // );
        // host_config_with_cookie.deployments.insert(
        //     "green".to_string(),
        //     DeploymentConfig {
        //         with_cookie: Some(WithCookieConfig {
        //             name: "GREEN".to_string(),
        //             value: Some("true".to_string()),
        //         }),
        //         ..Default::default()
        //     },
        // );
        // host_config_with_cookie.paths.push(PathConfig {
        //     path: "/api".to_string(),

        //     ..Default::default()
        // });

        let mut config = create_test_config(None);
        config.hosts[0].paths[0].deployments.insert(
            "green".to_string(),
            DeploymentConfig::builder()
                .ip("8.8.8.8".to_string())
                .selection(SelectionConfig {
                    with_cookie: Some(WithCookieConfig {
                        name: "BLUEGREEN".to_string(),
                        value: Some("true".to_string()),
                    }),
                    ..Default::default()
                })
                .build(),
        );
        config.hosts[0].paths[0].deployments.insert(
            "blue".to_string(),
            DeploymentConfig::builder()
                .ip("1.1.1.1".to_string())
                .build(),
        );
        let deployments = config.hosts[0].paths[0].deployments.clone();
        let host_deployments = config.hosts[0].get_sorted_deployments(&deployments);

        assert_eq!(host_deployments["blue"].get_weight(), 0);
        assert_eq!(host_deployments["green"].get_weight(), 1);
    }

    #[test]
    fn test_non_pass_through_configuration() {
        let host = create_test_host();
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        // Verify X-Pass-Through header
        let has_pass_through = pairs
            .iter()
            .any(|p| p.key().contains("Location") && p.value() == "");

        assert!(!has_pass_through, "Location header should not be present");
    }

    #[test]
    fn test_middleware_ordering() {
        let host = create_test_host();
        let pairs = host.to_etcd_pairs("traefik/http").unwrap();

        let middleware_keys: Vec<_> = pairs
            .iter()
            .filter(|p| p.key().contains("/middlewares/") && p.key().contains("path"))
            .collect();

        let mut middleware_order = Vec::new();
        for pair in &middleware_keys {
            if pair.value().contains("-strip") {
                middleware_order.push("strip_prefix");
            } else if pair.value().contains("enable-headers") {
                middleware_order.push("headers");
            } else if pair.value().contains("redirect-handler") {
                middleware_order.push("redirect_handler");
            }
        }

        assert_eq!(
            middleware_order,
            vec!["strip_prefix", "headers", "redirect_handler"],
            "Middleware order is not as expected"
        );
    }

    #[test]
    fn test_validate_domain() {
        let mut host = create_test_host();
        host.domain = "".to_string();
        assert!(
            host.validate().is_err(),
            "Empty domain should fail validation"
        );
    }

    #[test]
    fn test_validate_paths() {
        let mut host = create_test_host();

        // Test invalid path format
        host.paths.push(PathConfig {
            path: "invalid-path".to_string(),
            deployments: host.paths[0].deployments.clone(),
            middlewares: vec![],
            strip_prefix: true,
            pass_through: true,
        });
        assert!(
            host.validate().is_err(),
            "Invalid path format should fail validation"
        );

        // Test duplicate paths
        host.paths.push(PathConfig {
            path: "/api".to_string(),
            deployments: host.paths[0].deployments.clone(),
            middlewares: vec![],
            strip_prefix: true,
            pass_through: true,
        });
        assert!(
            host.validate().is_err(),
            "Duplicate paths should fail validation"
        );
    }

    #[test]
    fn test_validate_deployments() {
        let mut host = create_test_host();

        // Test empty deployments
        host.deployments.clear();
        assert!(
            host.validate().is_err(),
            "Empty deployments should fail validation"
        );

        // Test invalid weights
        host.deployments.insert(
            "blue".to_string(),
            DeploymentConfig {
                ip: "10.0.0.1".to_string(),
                port: 80,
                weight: 50,
                selection: None,
            },
        );
        host.deployments.insert(
            "green".to_string(),
            DeploymentConfig {
                ip: "10.0.0.2".to_string(),
                port: 80,
                weight: 30,
                selection: None,
            },
        );
        assert!(
            host.validate().is_err(),
            "Invalid weights should fail validation"
        );
    }

    #[test]
    fn test_valid_ip_validation() {
        let host = create_test_host();
        assert!(host.is_valid_ip_or_hostname("192.168.1.1"));
        assert!(host.is_valid_ip_or_hostname("10.0.0.1"));
        assert!(host.is_valid_ip_or_hostname("172.16.0.1"));
    }

    #[test]
    fn test_invalid_ip_validation() {
        let host = create_test_host();
        assert!(!host.is_valid_ip_or_hostname("256.1.2.3"));
    }

    #[test]
    fn test_valid_hostname_validation() {
        let host = create_test_host();
        assert!(host.is_valid_ip_or_hostname("example.com"));
        assert!(host.is_valid_ip_or_hostname("sub.example.com"));
        assert!(host.is_valid_ip_or_hostname("my-service.example.com"));
        assert!(host.is_valid_ip_or_hostname("localhost"));
    }

    #[test]
    fn test_invalid_hostname_validation() {
        let host = create_test_host();
        assert!(!host.is_valid_ip_or_hostname(""));
        assert!(!host.is_valid_ip_or_hostname("."));
        assert!(!host.is_valid_ip_or_hostname("example..com"));
        assert!(!host.is_valid_ip_or_hostname("-example.com"));
        assert!(!host.is_valid_ip_or_hostname("example-.com"));
        assert!(!host.is_valid_ip_or_hostname("exam ple.com"));
    }

    #[test]
    fn test_cookie_selection() {
        let mut rule = RuleConfig::default();
        let config = DeploymentConfig {
            selection: Some(SelectionConfig {
                with_cookie: Some(WithCookieConfig {
                    name: "TEST".to_string(),
                    value: Some("true".to_string()),
                }),
                from_client_ip: None,
            }),
            ..Default::default()
        };

        add_selection_rules(&config, &mut rule);
        assert_eq!(rule.get_weight(), 1);
        assert!(rule
            .rule_str()
            .contains("HeaderRegexp(`Cookie`, `TEST=true`)"));
    }

    #[test]
    fn test_client_ip_selection() {
        let mut rule = RuleConfig::default();
        let config = DeploymentConfig {
            selection: Some(SelectionConfig {
                with_cookie: None,
                from_client_ip: Some(FromClientIpConfig {
                    ip: Some("192.168.1.1".to_string()),
                    range: None,
                }),
            }),
            ..Default::default()
        };

        add_selection_rules(&config, &mut rule);
        assert_eq!(rule.get_weight(), 1);
        assert!(rule.rule_str().contains("ClientIP(`192.168.1.1`)"));
    }

    #[test]
    fn test_client_range_selection() {
        let mut rule = RuleConfig::default();
        let config = DeploymentConfig {
            selection: Some(SelectionConfig {
                with_cookie: None,
                from_client_ip: Some(FromClientIpConfig {
                    ip: None,
                    range: Some("192.168.1.0/24".to_string()),
                }),
            }),
            ..Default::default()
        };

        add_selection_rules(&config, &mut rule);
        assert_eq!(rule.get_weight(), 1);
        assert!(rule.rule_str().contains("ClientIP(`192.168.1.0/24`)"));
    }

    #[test]
    fn test_client_prefers_ip_selection() {
        let mut rule = RuleConfig::default();
        let config = DeploymentConfig {
            selection: Some(SelectionConfig {
                with_cookie: None,
                from_client_ip: Some(FromClientIpConfig {
                    ip: Some("192.168.1.1".to_string()),
                    range: Some("192.168.1.0/24".to_string()),
                }),
            }),
            ..Default::default()
        };

        add_selection_rules(&config, &mut rule);
        assert_eq!(rule.get_weight(), 2);
        assert!(rule.rule_str().contains("ClientIP(`192.168.1.1`)"));
        assert!(rule.rule_str().contains("ClientIP(`192.168.1.0/24`)"));
    }

    #[test]
    fn test_combined_selection() {
        let mut rule = RuleConfig::default();
        let config = DeploymentConfig {
            selection: Some(SelectionConfig {
                with_cookie: Some(WithCookieConfig {
                    name: "TEST".to_string(),
                    value: Some("true".to_string()),
                }),
                from_client_ip: Some(crate::config::FromClientIpConfig {
                    ip: Some("192.168.1.1".to_string()),
                    range: None,
                }),
            }),
            ..Default::default()
        };

        add_selection_rules(&config, &mut rule);
        assert_eq!(rule.get_weight(), 2);
        let rule_str = rule.rule_str();
        assert!(rule_str.contains("HeaderRegexp(`Cookie`, `TEST=true`)"));
        assert!(rule_str.contains("ClientIP(`192.168.1.1`)"));
    }

    #[test]
    fn test_rule_ordering() {
        let mut host = HostConfig::default();

        // Deployment with 2 rules (cookie + IP)
        host.deployments.insert(
            "blue".to_string(),
            DeploymentConfig {
                selection: Some(SelectionConfig {
                    with_cookie: Some(WithCookieConfig {
                        name: "TEST".to_string(),
                        value: Some("true".to_string()),
                    }),
                    from_client_ip: Some(FromClientIpConfig {
                        ip: Some("192.168.1.1".to_string()),
                        range: None,
                    }),
                }),
                ..Default::default()
            },
        );

        // Deployment with 1 rule (cookie only)
        host.deployments.insert(
            "green".to_string(),
            DeploymentConfig {
                selection: Some(SelectionConfig {
                    with_cookie: Some(WithCookieConfig {
                        name: "GREEN".to_string(),
                        value: None,
                    }),
                    from_client_ip: None,
                }),
                ..Default::default()
            },
        );

        let sorted = host.get_sorted_deployments(&host.deployments);
        let deployments: Vec<_> = sorted.keys().collect();

        // Blue deployment should come first as it has more rules
        assert_eq!(deployments, vec!["blue", "green"]);
    }

    #[test]
    fn test_default_cookie_value() {
        let mut rule = RuleConfig::default();
        let config = DeploymentConfig {
            selection: Some(SelectionConfig {
                with_cookie: Some(WithCookieConfig {
                    name: "TEST".to_string(),
                    value: None,
                }),
                from_client_ip: None,
            }),
            ..Default::default()
        };

        add_selection_rules(&config, &mut rule);
        assert!(rule
            .rule_str()
            .contains("HeaderRegexp(`Cookie`, `TEST=true`)"));
    }
}
