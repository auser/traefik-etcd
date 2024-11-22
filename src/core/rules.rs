use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    config::{
        deployment::DeploymentConfig,
        host::{HostConfig, PathConfig},
        selections::SelectionConfig,
    },
    error::TraefikResult,
    TraefikConfig,
};

use super::etcd_trait::EtcdPair;

// use super::etcd_trait::EtcdPair;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Rule {
    key: String,
    value: String,
    rule_type: RuleType,
}

impl Rule {
    fn new(key: &str, value: &str, rule_type: RuleType) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            rule_type,
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let rule_str = match self.rule_type {
            RuleType::Other => format!("{}(`{}`)", self.key, self.value),
            RuleType::Header => format!("HeaderRegexp(`{}`, `{}`)", self.key, self.value),
            RuleType::Host => format!("Host(`{}`)", self.value),
            RuleType::ClientIp => format!("ClientIP(`{}`)", self.value),
            RuleType::TcpHost => format!("HostSNI(`{}`)", self.value),
        };
        write!(f, "{}", rule_str)
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self::new("", "", RuleType::Other)
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RuleEntry {
    weight: usize,
    rule: String,
}

impl RuleEntry {
    pub fn new(weight: usize, rule: String) -> Self {
        Self { weight, rule }
    }

    pub fn get_weight(&self) -> usize {
        self.weight
    }

    pub fn get_rule(&self) -> String {
        self.rule.clone()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleConfig {
    rules: HashSet<Rule>,
}

impl RuleConfig {
    pub fn add_rule(&mut self, key: &str, value: &str, rule_type: RuleType) {
        let rule = Rule::new(key, value, rule_type);
        self.rules.insert(rule);
    }

    pub fn add_default_rule(&mut self, key: &str, value: &str) {
        self.add_rule(key, value, RuleType::Other);
    }

    pub fn add_default_rule_from_optional_path(&mut self, key: &str, path: Option<&PathConfig>) {
        if let Some(path) = path {
            self.add_default_rule(key, &path.path);
        }
    }

    pub fn add_header_rule(&mut self, header: &str, value: &str) -> &mut Self {
        self.add_rule(header, value, RuleType::Header);
        self
    }

    pub fn add_client_ip_rule(&mut self, ip: Option<&str>, range: Option<&str>) -> &mut Self {
        if let Some(ip) = ip {
            self.add_rule("ip", ip, RuleType::ClientIp);
        }
        if let Some(range) = range {
            self.add_rule("range", range, RuleType::ClientIp);
        };
        self
    }

    pub fn add_host_rule(&mut self, domain: &str) -> &mut Self {
        self.add_rule("Host", domain, RuleType::Host);
        self
    }

    pub fn add_tcp_rule(&mut self, service: &str) -> &mut Self {
        self.add_rule("HostSNI", service, RuleType::TcpHost);
        self
    }

    pub fn rule_str(&self) -> String {
        // Sort rules to ensure consistent ordering
        let mut rules: Vec<_> = self.rules.iter().collect();
        rules.sort_by_key(|rule| {
            match rule.rule_type {
                RuleType::Host => 0,     // Host rules first
                RuleType::Header => 1,   // Then Header rules
                RuleType::ClientIp => 2, // Then ClientIP rules
                RuleType::TcpHost => 3,  // TCP Host rules
                RuleType::Other => 4,    // Other rules last
            }
        });

        rules
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<String>>()
            .join(" && ")
    }

    // Weight is now determined by the number of rules
    pub fn get_weight(&self) -> usize {
        self.rules.len()
    }
}

/// Rules can be of different types
/// Host rules are used to match the host of the request
/// Header rules are used to match the headers of the request
/// ClientIP rules are used to match the client IP of the request
/// TcpHost rules are used to match the SNI of the request
/// Other rules are used to match other types of rules
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum RuleType {
    Host,
    Header,
    ClientIp,
    TcpHost,
    Other,
}

pub fn add_selection_rules<T>(with_selection: &T, rules: &mut RuleConfig)
where
    T: Into<Option<SelectionConfig>> + Clone,
{
    let selection_rules: Option<SelectionConfig> = (*with_selection).clone().into();
    if let Some(selection) = &selection_rules {
        if let Some(with_cookie) = &selection.with_cookie {
            rules.add_header_rule(
                "Cookie",
                &format!(
                    "{}={}",
                    with_cookie.name,
                    with_cookie.value.as_deref().unwrap_or("true")
                ),
            );
        }
        if let Some(from_client_ip) = &selection.from_client_ip {
            rules.add_client_ip_rule(
                from_client_ip.ip.as_deref(),
                from_client_ip.range.as_deref(),
            );
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InternalDeploymentConfig {
    /// The deployment configuration
    #[serde(default)]
    pub deployment: DeploymentConfig,
    /// The name of the deployment
    #[serde(default)]
    pub name: String,
    /// The weight of the deployment
    #[serde(default = "default_weight")]
    pub weight: usize,
    /// The path of the deployment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_config: Option<PathConfig>,
    /// The host of the deployment
    pub host_config: HostConfig,
    /// The rules of the deployment
    pub rules: RuleConfig,
}

fn default_weight() -> usize {
    100
}

pub fn get_sorted_deployments(
    traefik_config: &TraefikConfig,
) -> TraefikResult<Vec<InternalDeploymentConfig>> {
    let mut internal_deployments = collect_all_deployments(traefik_config)?;

    internal_deployments.sort_by(|a, b| {
        // First by number of selections
        let a_has_selection = a.deployment.selection.is_some();
        let b_has_selection = b.deployment.selection.is_some();
        let selection_cmp = b_has_selection.cmp(&a_has_selection);
        if selection_cmp != std::cmp::Ordering::Equal {
            return selection_cmp;
        }

        // Then by rules weight
        let rule_weight_cmp = b.rules.get_weight().cmp(&a.rules.get_weight());
        if rule_weight_cmp != std::cmp::Ordering::Equal {
            return rule_weight_cmp;
        }

        // Finally by original order (lower weight = earlier in file)
        a.weight.cmp(&b.weight)
    });

    Ok(internal_deployments)
}

fn collect_all_deployments(
    traefik_config: &TraefikConfig,
) -> TraefikResult<Vec<InternalDeploymentConfig>> {
    let mut internal_deployments = Vec::new();

    for host in traefik_config.hosts.iter() {
        // First get all the deployments from the host
        internal_deployments.extend(get_all_internal_deployments(host, &host.deployments, None)?);

        // next get all the deployments from the paths
        for path in host.paths.iter() {
            internal_deployments.extend(get_all_internal_deployments(
                host,
                &path.deployments,
                Some(path),
            )?);
        }
    }

    // Adjust the weights of the deployments based upon the order
    // in which they were added to the internal_deployments vector
    // This is done to ensure that the weights are unique and to make
    // the rule complexity more predictable as well as give an edge to
    // the first deployments added
    // let total_number_of_deployments = internal_deployments.len();
    let mut internal_deployments_result = Vec::new();
    for internal_deployment in internal_deployments.iter_mut() {
        // Add the host rule
        let mut rules = internal_deployment.rules.clone();
        rules.add_host_rule(&internal_deployment.host_config.domain);
        // Add the path rule if it exists
        rules.add_default_rule_from_optional_path(
            "PathPrefix",
            internal_deployment.path_config.as_ref(),
        );
        // Add the selection rules
        add_selection_rules(&internal_deployment.deployment, &mut rules);
        let mut internal_deployment = internal_deployment.clone();
        internal_deployment.rules = rules;
        // Add the weight of the rules to the weight of the deployment
        internal_deployment.weight += 1000 + internal_deployment.rules.get_weight();
        internal_deployments_result.push(internal_deployment.clone());
    }

    Ok(internal_deployments_result)
}

/// Get all the internal deployments for a given host or path
fn get_all_internal_deployments(
    host_config: &HostConfig,
    deployments: &HashMap<String, DeploymentConfig>,
    path: Option<&PathConfig>,
) -> TraefikResult<Vec<InternalDeploymentConfig>> {
    let mut internal_deployments = Vec::new();

    // Get deployments in sorted order by key
    let mut deployment_keys: Vec<&String> = deployments.keys().collect();
    deployment_keys.sort(); // Sort keys for deterministic order

    for (idx, key) in deployment_keys.iter().enumerate() {
        let deployment = &deployments[*key];
        if deployment.weight > 0 || deployment.selection.is_some() {
            internal_deployments.push(InternalDeploymentConfig {
                deployment: deployment.clone(),
                name: (*key).clone(),
                host_config: host_config.clone(),
                weight: idx,
                path_config: path.cloned(),
                rules: RuleConfig::default(),
            });
        }
    }
    Ok(internal_deployments)
}

pub fn add_deployment_rules(
    host: &HostConfig,
    sorted_deployments: &[InternalDeploymentConfig],
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    rules: &mut RuleConfig,
) -> TraefikResult<()> {
    for deployment in sorted_deployments.iter() {
        let router_name = format!("{}-router", deployment.name);
        let rule = rules.clone();

        debug!("Adding deployment middlewares for {}", router_name);
        let additional_middlewares = host.middlewares.clone();
        let strip_prefix_name = add_strip_prefix_middleware(
            pairs,
            base_key,
            &router_name,
            deployment.path_config.clone(),
        )?;
        add_middlewares(
            pairs,
            base_key,
            &router_name,
            &additional_middlewares,
            strip_prefix_name.as_deref(),
        )?;

        debug!("Adding deployment rules for {}", router_name);
        add_root_router(pairs, base_key, &router_name, &rule)?;

        let service_name = format!("{}-service", router_name);
        add_base_service_configuration(pairs, base_key, &service_name, deployment)?;
    }

    Ok(())
}

pub fn add_root_router(
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    router_name: &str,
    rule: &RuleConfig,
) -> TraefikResult<()> {
    debug!("Adding root router for {}", router_name);
    pairs.push(EtcdPair::new(
        format!("{}/routers/{}/rule", base_key, router_name),
        rule.rule_str(),
    ));
    debug!("Added rule: {}", rule.rule_str());
    pairs.push(EtcdPair::new(
        format!("{}/routers/{}/entrypoints/0", base_key, router_name),
        "websecure",
    ));
    debug!("Added entrypoint: websecure");
    pairs.push(EtcdPair::new(
        format!("{}/routers/{}/tls", base_key, router_name),
        "true",
    ));
    debug!("Added tls: true");
    pairs.push(EtcdPair::new(
        format!(
            "{}/services/{}/loadBalancer/passHostHeader",
            base_key, router_name
        ),
        "true".to_string(),
    ));
    debug!("Added passHostHeader: true");

    // Set priority based on rule complexity
    pairs.push(EtcdPair::new(
        format!("{}/routers/{}/priority", base_key, router_name),
        (1000 + rule.get_weight() * 10).to_string(),
    ));
    Ok(())
}

fn add_base_service_configuration(
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    service_name: &str,
    internal_deployment_config: &InternalDeploymentConfig,
) -> TraefikResult<()> {
    let deployment = internal_deployment_config.deployment.clone();
    debug!(
        "Adding service {} pointing to http://{}:{}",
        service_name, deployment.ip, deployment.port
    );

    pairs.push(EtcdPair::new(
        format!(
            "{}/services/{}/loadBalancer/servers/0/url",
            base_key, service_name
        ),
        format!("http://{}:{}", deployment.ip, deployment.port),
    ));

    pairs.push(EtcdPair::new(
        format!(
            "{}/services/{}/loadBalancer/passHostHeader",
            base_key, service_name
        ),
        "true".to_string(),
    ));

    pairs.push(EtcdPair::new(
        format!(
            "{}/services/{}/loadBalancer/responseForwarding/flushInterval",
            base_key, service_name
        ),
        "100ms".to_string(),
    ));

    Ok(())
}

pub fn add_middlewares(
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

pub fn add_strip_prefix_middleware(
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    path_safe_name: &str,
    path_config: Option<PathConfig>,
) -> TraefikResult<Option<String>> {
    let strip_prefix_name = if let Some(path_config) = path_config {
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
    Ok(strip_prefix_name)
}

pub fn add_pass_through_middleware(
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    path_safe_name: &str,
    path_config: PathConfig,
) -> TraefikResult<()> {
    if path_config.pass_through {
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Pass-Through",
                base_key, path_safe_name
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

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_helpers::{
        assert_contains_pair, create_complex_test_config, create_test_deployment, create_test_host,
        read_test_config,
    };

    use super::*;

    #[test]
    fn test_collect_all_deployments_from_config() {
        let config = read_test_config();
        let deployments = collect_all_deployments(&config).unwrap();
        assert_eq!(deployments.len(), 3);
        let deployment_names: Vec<String> = deployments.iter().map(|d| d.name.clone()).collect();
        for name in ["catch-all", "green", "blue"] {
            assert!(deployment_names.contains(&name.to_string()));
        }
    }

    #[test]
    fn test_catch_all_deep_deployments() {
        let config = create_complex_test_config();
        let deployments = collect_all_deployments(&config).unwrap();
        assert_eq!(deployments.len(), 7);
        let deployment_names: Vec<String> = deployments.iter().map(|d| d.name.clone()).collect();
        for name in ["catch-all", "green", "blue", "test-1", "test-2", "test-3"] {
            assert!(deployment_names.contains(&name.to_string()));
        }
    }

    #[test]
    fn test_get_sorted_deployments_with_a_deployment_weight_of_zero() {
        let mut config = create_complex_test_config();
        config.hosts[1].deployments.get_mut("bingo").unwrap().weight = 0;
        let deployments = get_sorted_deployments(&config).unwrap();
        assert_eq!(deployments.len(), 6);
    }

    #[test]
    fn test_collect_all_deployments_from_config_with_zero_hosts() {
        let config = TraefikConfig::default();
        let deployments = collect_all_deployments(&config).unwrap();
        assert_eq!(deployments.len(), 0);
    }

    #[test]
    fn test_get_all_sorted_deployments() {
        let config = create_complex_test_config();
        let deployments = get_sorted_deployments(&config).unwrap();
        assert_eq!(deployments.len(), 7);
        let deployment_names: Vec<String> = deployments.iter().map(|d| d.name.clone()).collect();
        assert_eq!(
            deployment_names,
            [
                "test-1",
                "green",
                "test-2",
                "test-3",
                "blue",
                "bingo",
                "catch-all",
            ]
        );
    }

    #[test]
    fn test_selection_based_deployment_ordering() {
        let config = create_complex_test_config();
        let deployments = get_sorted_deployments(&config).unwrap();
        let deployment_names: Vec<String> = deployments.iter().map(|d| d.name.clone()).collect();

        // green and test-1 have selections, should be first
        let green_pos = deployment_names.iter().position(|x| x == "green").unwrap();
        let test1_pos = deployment_names.iter().position(|x| x == "test-1").unwrap();

        // Verify selections come first
        assert!(green_pos < deployment_names.len() - 2);
        assert!(test1_pos < deployment_names.len() - 2);

        // Verify non-selection deployments come after
        for name in ["blue", "catch-all", "test-2", "test-3", "bingo"] {
            let pos = deployment_names.iter().position(|x| x == name).unwrap();
            assert!(pos > green_pos || pos > test1_pos);
        }
    }

    #[test]
    fn test_rule_config_host_valid_with_host_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_host_rule("example.com");
        assert_eq!(rule_config.rule_str(), "Host(`example.com`)");
    }

    #[test]
    fn test_rule_config_client_ip_valid_with_ip_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_client_ip_rule(Some("192.168.1.1"), None);
        assert_eq!(rule_config.rule_str(), "ClientIP(`192.168.1.1`)");
    }

    #[test]
    fn test_rule_config_client_ip_valid_with_range_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(rule_config.rule_str(), "ClientIP(`192.168.1.1/24`)");
    }

    #[test]
    fn test_rule_config_host_and_client_ip_valid() {
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_host_rule("example.com")
            .add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(
            rule_config.rule_str(),
            "Host(`example.com`) && ClientIP(`192.168.1.1/24`)"
        );
    }

    #[test]
    fn test_rule_config_weight_with_two_rules() {
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_host_rule("example.com")
            .add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(rule_config.get_weight(), 2);
    }

    #[test]
    fn test_rule_config_valid_with_header_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_header_rule("X-Forwarded-Proto", "https");
        assert_eq!(
            rule_config.rule_str(),
            "HeaderRegexp(`X-Forwarded-Proto`, `https`)"
        );
    }

    #[test]
    fn test_rule_config_with_valid_header_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_header_rule("X-Forwarded-Proto", "https");
        assert_eq!(
            rule_config.rule_str(),
            "HeaderRegexp(`X-Forwarded-Proto`, `https`)"
        );
    }

    #[test]
    fn test_rule_config_valid_with_other_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_default_rule("key", "value");
        assert_eq!(rule_config.rule_str(), "key(`value`)");
    }

    #[test]
    fn test_rule_config_with_valid_tcp_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_tcp_rule("service");
        assert_eq!(rule_config.rule_str(), "HostSNI(`service`)");
    }

    #[test]
    fn test_rule_get_weight() {
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_host_rule("example.com")
            .add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(rule_config.get_weight(), 2);
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_header_rule("X-Forwarded-Proto", "https")
            .add_client_ip_rule(None, Some("192.168.1.1/24"))
            .add_header_rule("X-Forwarded-Port", "443");
        assert_eq!(rule_config.get_weight(), 3);
    }

    #[test]
    fn test_add_deployment_rules() {
        let host = create_test_host();
        let base_key = "test";
        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        // Create test deployments
        let deployment1 = InternalDeploymentConfig {
            name: "test1".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            path_config: Some(PathConfig {
                path: "/api".to_string(),
                strip_prefix: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        let deployment2 = InternalDeploymentConfig {
            name: "test2".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            path_config: None,
            ..Default::default()
        };

        let deployments = vec![deployment1, deployment2];

        add_deployment_rules(&host, &deployments, &mut pairs, base_key, &mut rules).unwrap();

        // Verify router configurations
        assert_contains_pair(
            &pairs,
            "test/routers/test1-router/entrypoints/0",
            "websecure",
        );
        assert_contains_pair(
            &pairs,
            "test/routers/test2-router/entrypoints/0",
            "websecure",
        );

        // Verify strip prefix middleware for path deployment
        assert_contains_pair(
            &pairs,
            "test/middlewares/test1-router-strip/stripPrefix/prefixes/0",
            "/api",
        );

        // Verify service configurations
        assert_contains_pair(
            &pairs,
            "test/services/test1-router-service/loadBalancer/servers/0/url",
            "http://10.0.0.1:8080",
        );
        assert_contains_pair(
            &pairs,
            "test/services/test2-router-service/loadBalancer/servers/0/url",
            "http://10.0.0.1:8080",
        );
    }

    #[test]
    fn test_add_deployment_rules_with_middlewares() {
        let mut host = create_test_host();
        host.middlewares = vec!["test-middleware".to_string()];

        let deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };

        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        add_deployment_rules(&host, &[deployment], &mut pairs, "test", &mut rules).unwrap();

        // Verify middleware configuration
        assert_contains_pair(
            &pairs,
            "test/routers/test-router/middlewares/0",
            "test-router-headers",
        );
        assert_contains_pair(
            &pairs,
            "test/routers/test-router/middlewares/1",
            "test-middleware",
        );
    }

    #[test]
    fn test_add_deployment_rules_empty_deployments() {
        let host = create_test_host();
        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        let result = add_deployment_rules(&host, &[], &mut pairs, "test", &mut rules);
        assert!(result.is_ok());
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_deployment_rules_and_services() {
        let mut host = create_test_host();
        host.domain = "domain.com".to_string();
        let base_key = "traefik/http";
        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();
        rules.add_host_rule(&host.domain);

        let deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };

        add_deployment_rules(&host, &[deployment], &mut pairs, base_key, &mut rules).unwrap();

        let router_name = "test-router";
        let service_name = "test-router-service";

        // Verify router rule exists
        assert_contains_pair(
            &pairs,
            &format!("{}/routers/{}/rule", base_key, router_name),
            "Host(`domain.com`)",
        );

        // Verify service exists with correct URL
        assert_contains_pair(
            &pairs,
            &format!(
                "{}/services/{}/loadBalancer/servers/0/url",
                base_key, service_name
            ),
            "http://10.0.0.1:8080",
        );

        // Verify router is linked to service
        assert_contains_pair(
            &pairs,
            &format!(
                "{}/services/{}/loadBalancer/passHostHeader",
                base_key, service_name
            ),
            "true",
        );
    }
}
