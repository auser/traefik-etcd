use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tera::{Context, Tera};
use tracing::{debug, error};

use crate::config::deployment::{DeploymentProtocol, DeploymentTarget};
use crate::config::headers::HeadersConfig;
use crate::config::middleware::MiddlewareConfig;
use crate::config::services::ServiceConfig;
use crate::core::util::get_safe_key;
use crate::error::TraefikError;
use crate::{
    config::{
        deployment::DeploymentConfig,
        host::{HostConfig, PathConfig},
        selections::SelectionConfig,
    },
    error::TraefikResult,
    TraefikConfig,
};

use super::etcd_trait::{EtcdPair, ToEtcdPairs};

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

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RouterRule {
    rule: String,
    priority: i32,
    router_name: String,
}

impl RouterRule {
    pub fn new(rule: String, priority: i32, router_name: String) -> Self {
        Self {
            rule,
            priority,
            router_name,
        }
    }

    pub fn from_pairs(pairs: &[EtcdPair]) -> Vec<Self> {
        let rules = parse_pairs(pairs);
        let mut router_rules: Vec<RouterRule> = rules.into_iter().map(|r| r.into()).collect();

        // Sort rules by priority (highest first)
        router_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        router_rules
    }

    pub fn get_rule(&self) -> String {
        self.rule.clone()
    }

    pub fn get_priority(&self) -> i32 {
        self.priority
    }
}

struct RuleLine {
    rule: String,
    router_name: String,
    priority: usize,
}

impl From<RuleLine> for RouterRule {
    fn from(rule_line: RuleLine) -> Self {
        RouterRule::new(
            rule_line.rule,
            rule_line.priority as i32,
            rule_line.router_name,
        )
    }
}

fn parse_pairs(pairs: &[EtcdPair]) -> Vec<RuleLine> {
    let mut rule_lines = Vec::new();

    for (i, pair) in pairs.iter().enumerate() {
        if pair.key().ends_with("/rule") {
            // Extract router name from the key
            let parts = pair.key().split('/').collect::<Vec<&str>>();
            let router_name = parts[parts.len() - 2].to_string();

            // Look ahead for matching priority
            let priority_key = format!("traefik/http/routers/{}/priority", router_name);
            if let Some(priority_pair) = pairs[i..].iter().find(|p| p.key() == priority_key) {
                if let Ok(priority) = priority_pair.value().parse::<usize>() {
                    rule_lines.push(RuleLine {
                        rule: pair.value().to_string(),
                        router_name,
                        priority,
                    });
                }
            }
        }
    }

    rule_lines
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
    /// The traefik configuration
    pub traefik_config: TraefikConfig,
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

impl ToEtcdPairs for InternalDeploymentConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut internal_deployment = self.clone();
        let internal_deployment = internal_deployment.init();
        let mut pairs = Vec::new();

        let mut rules = internal_deployment.rules.clone();

        // Process middleware templates and add them to pairs
        let processed_middlewares = self.process_middleware_templates()?;
        debug!("Processed middlewares: {:#?}", processed_middlewares);
        let mut traefik_config = internal_deployment.traefik_config.clone();

        for (name, middleware) in processed_middlewares {
            debug!(
                "Adding middleware In InternalDeploymentConfig: {} => {:#?}",
                name, middleware
            );
            println!("base_key: {}", base_key);

            traefik_config.middlewares.insert(name, middleware.clone());
            // pairs.extend(middleware.to_etcd_pairs(&base_key)?);
        }

        add_deployment_rules(
            &self.host_config,
            &[internal_deployment.clone()],
            traefik_config.services.as_ref(),
            &mut pairs,
            base_key,
            &mut rules,
        )?;
        Ok(pairs)
    }
}

impl InternalDeploymentConfig {
    pub fn init(&mut self) -> &mut Self {
        let mut rules = self.rules.clone();
        debug!("Initializing deployment config for {}", self.name);

        match self.deployment.protocol {
            DeploymentProtocol::Http => {
                rules.add_host_rule(&self.host_config.domain);
            }
            DeploymentProtocol::Https => {
                rules.add_host_rule(&self.host_config.domain);
            }
            DeploymentProtocol::Tcp => {
                rules.add_tcp_rule(&self.host_config.domain);
            }
            DeploymentProtocol::Invalid => {
                error!("Invalid deployment protocol for {}", self.name);
            }
        };
        // Add the path rule if it exists
        rules.add_default_rule_from_optional_path("PathPrefix", self.path_config.as_ref());
        // Add the selection rules
        add_selection_rules(&self.deployment, &mut rules);
        self.rules = rules;
        // Add the weight of the rules to the weight of the deployment
        self.weight += 1000 + self.rules.get_weight();

        self
    }

    fn process_middleware_templates(&self) -> TraefikResult<Vec<(String, MiddlewareConfig)>> {
        let mut processed = Vec::new();
        let mut tera = Tera::default();
        debug!("Processing middleware templates for {}", self.name);

        if let Some(templates) = &self.deployment.middleware_templates {
            let context = self.create_template_context()?;

            for (name, template) in templates {
                debug!(
                    "Processing middleware templates for {} (or name: {})",
                    self.name, name
                );
                let mut middleware = template.clone();
                middleware.set_name(&format!("{}-{}-templated", self.name, name));

                if let Some(runtime_headers) = middleware.runtime_headers.take() {
                    // Note the take()
                    let mut rendered_headers = HashMap::new();

                    for (header_name, template_value) in &runtime_headers.template_headers {
                        let rendered = tera
                            .render_str(template_value, &context)
                            .map_err(|e| TraefikError::MiddlewareTemplateError(e.to_string()))?;

                        rendered_headers.insert(header_name.clone(), rendered);
                    }

                    // Set the rendered headers in the headers field
                    middleware.headers = Some(HeadersConfig {
                        custom_request_headers: rendered_headers,
                        ..Default::default()
                    });
                }

                processed.push((middleware.name.clone(), middleware));
            }
        }

        Ok(processed)
    }

    fn create_template_context(&self) -> TraefikResult<Context> {
        debug!("Creating template context for {}", self.name);
        let mut context = Context::new();

        let service_context = match &self.deployment.target {
            DeploymentTarget::IpAndPort { ip, port } => {
                json!({
                    "service": {
                        "ip": ip,
                        "port": port,
                    }
                })
            }
            DeploymentTarget::Service { service_name } => {
                if let Some(services) = &self.traefik_config.services {
                    if let Some(service) = services.get(service_name) {
                        match &service.deployment.target {
                            DeploymentTarget::IpAndPort { ip, port } => {
                                json!({
                                    "service": {
                                        "ip": ip,
                                        "port": port,
                                    }
                                })
                            }
                            _ => {
                                return Err(TraefikError::ServiceConfig(format!(
                                    "Service {} has invalid target type",
                                    service_name
                                )))
                            }
                        }
                    } else {
                        return Err(TraefikError::ServiceConfig(format!(
                            "Service {} not found",
                            service_name
                        )));
                    }
                } else {
                    return Err(TraefikError::ServiceConfig("No services defined".into()));
                }
            }
        };

        context.insert("deployment", &service_context);
        context.insert("host", &self.host_config);
        if let Some(path) = &self.path_config {
            context.insert("path", &path);
        }
        context.insert("traefik_config", &self.traefik_config);
        Ok(context)
    }
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
        internal_deployments.extend(get_all_internal_deployments(
            traefik_config,
            host,
            &host.deployments,
            None,
        )?);

        // next get all the deployments from the paths
        for path in host.paths.iter() {
            internal_deployments.extend(get_all_internal_deployments(
                traefik_config,
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
    let mut internal_deployments_result = Vec::new();
    for internal_deployment in internal_deployments.iter_mut() {
        // Add the host rule
        internal_deployments_result.push(internal_deployment.init().clone());
    }
    Ok(internal_deployments_result)
}

/// Get all the internal deployments for a given host or path
fn get_all_internal_deployments(
    traefik_config: &TraefikConfig,
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
            let mut deployment = deployment.clone();
            deployment.set_name(&format!("{}-deployment", *key));
            internal_deployments.push(InternalDeploymentConfig {
                deployment: deployment.clone(),
                name: (*key).clone(),
                host_config: host_config.clone(),
                weight: idx,
                path_config: path.cloned(),
                rules: RuleConfig::default(),
                traefik_config: traefik_config.clone(),
            });
        }
    }
    Ok(internal_deployments)
}

pub fn add_deployment_rules(
    host: &HostConfig,
    sorted_deployments: &[InternalDeploymentConfig],
    services: Option<&HashMap<String, ServiceConfig>>,
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    rules: &mut RuleConfig,
) -> TraefikResult<()> {
    for deployment in sorted_deployments.iter() {
        let router_name = match host.domain.as_str() {
            "" => format!("{}-router", get_safe_key(&deployment.name)),
            domain => format!(
                "{}-{}-router",
                get_safe_key(domain),
                get_safe_key(&deployment.name)
            ),
        };
        let rule = rules.clone();
        let deployment_protocol = &deployment.deployment.protocol;
        let base_key = format!("{}/{}", base_key, deployment_protocol);

        debug!("Adding deployment middlewares for {}", router_name);
        let mut additional_middlewares = host.middlewares.clone();
        let strip_prefix_name = add_strip_prefix_middleware(
            pairs,
            &base_key,
            &router_name,
            deployment.path_config.clone(),
        )?;
        if let Some(strip_prefix_middleware_name) = strip_prefix_name.clone() {
            additional_middlewares.push(strip_prefix_middleware_name);
        }

        let mut deployment_traefik_config = deployment.traefik_config.clone();
        set_all_middleware_names(&mut deployment_traefik_config)?;

        let service_name = format!("{}-service", router_name);
        debug!("Adding deployment rules for {}", router_name);
        let (calculated_service_name, processed_middlewares) =
            add_base_service_configuration(pairs, &base_key, &service_name, &deployment, services)?;

        let service_name = calculated_service_name;

        let mut traefik_config = deployment_traefik_config.clone();
        if !processed_middlewares.is_empty() {
            let mut processed_middleware_names = Vec::new();
            for (middleware_name, middleware) in processed_middlewares.iter() {
                processed_middleware_names.push(middleware_name.clone());
                let middleware_key = format!("{}/middlewares", base_key);
                println!(
                    "from templated middleware middleware_key: {}",
                    middleware_key
                );
                println!("middleware_name: {}", middleware_name);
                traefik_config
                    .middlewares
                    .insert(middleware_name.clone(), middleware.clone());
                println!(
                    "traefik_config.middlewares: {:#?}",
                    traefik_config.middlewares
                );
            }

            additional_middlewares.extend(processed_middleware_names);
        }

        debug!(
            "Additional middlewares pre-adding: {:?}",
            additional_middlewares
        );
        let middleware_names = add_middlewares(
            &traefik_config,
            pairs,
            &base_key,
            &router_name,
            &additional_middlewares,
            strip_prefix_name.as_deref(),
            host,
        )?;

        attach_middlewares(pairs, &base_key, &router_name, &middleware_names)?;

        // Link router to the correct service based on rule
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/service", base_key, router_name),
            service_name.clone(),
        ));

        debug!("Adding deployment rules for {}", router_name);
        add_root_router(pairs, &base_key, &router_name, &rule)?;
    }

    Ok(())
}

fn set_all_middleware_names(traefik_config: &mut TraefikConfig) -> TraefikResult<()> {
    for (name, middleware) in traefik_config.middlewares.iter_mut() {
        middleware.set_name(name);
    }
    Ok(())
}

pub fn attach_middlewares(
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    router_name: &str,
    middleware_names: &[String],
) -> TraefikResult<()> {
    for (idx, middleware_name) in middleware_names.iter().enumerate() {
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/middlewares/{}", base_key, router_name, idx),
            middleware_name.clone(),
        ));
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

    let router_key = format!("{}/routers/{}", base_key, router_name);
    pairs.push(EtcdPair::new(
        format!("{}/rule", router_key),
        rule.rule_str(),
    ));
    debug!(
        "Added rule router {}: {} (weight: {})",
        router_name,
        rule.rule_str(),
        rule.get_weight()
    );
    pairs.push(EtcdPair::new(
        format!("{}/entryPoints/0", router_key),
        "websecure",
    ));
    debug!("Added entrypoint: websecure");
    pairs.push(EtcdPair::new(format!("{}/tls", router_key), "true"));
    debug!("Added tls: true");

    // Set priority based on rule complexity
    pairs.push(EtcdPair::new(
        format!("{}/priority", router_key),
        (1000 + rule.get_weight() * 10).to_string(),
    ));
    Ok(())
}

fn add_base_service_configuration(
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    service_name: &str,
    internal_deployment_config: &InternalDeploymentConfig,
    services: Option<&HashMap<String, ServiceConfig>>,
) -> TraefikResult<(String, Vec<(String, MiddlewareConfig)>)> {
    let deployment = internal_deployment_config.deployment.clone();
    // let deployment_protocol = deployment.protocol;

    let mut actual_service_name = service_name;

    match &deployment.target {
        DeploymentTarget::IpAndPort { ip, port } => {
            let base_key = format!("{}/services/{}", base_key, service_name);
            debug!(
                "Adding service {} pointing to http://{}:{}",
                service_name, ip, port
            );

            pairs.push(EtcdPair::new(
                format!("{}/loadBalancer/servers/0/url", base_key),
                format!("http://{}:{}", ip, port),
            ));

            pairs.push(EtcdPair::new(
                format!("{}/loadBalancer/passHostHeader", base_key),
                "true".to_string(),
            ));

            pairs.push(EtcdPair::new(
                format!("{}/loadBalancer/responseForwarding/flushInterval", base_key),
                "100ms".to_string(),
            ));
        }
        DeploymentTarget::Service { service_name } => {
            if let Some(services) = services {
                if let Some(service) = services.get(service_name) {
                    let mut service = service.clone();
                    actual_service_name = service_name.as_str();
                    service.set_name(service_name);
                    println!("2> service_name: {}", service_name);
                    println!("2> base_key: {}", base_key);
                    let service_pairs = service.to_etcd_pairs(base_key)?;
                    pairs.extend(service_pairs);
                } else {
                    return Err(TraefikError::ServiceConfig(format!(
                        "Service {} not found",
                        service_name
                    )));
                }
            } else {
                return Err(TraefikError::ServiceConfig(
                    "Services not found in config".to_string(),
                ));
            }
        }
    };

    let processed_middlewares = internal_deployment_config.process_middleware_templates()?;

    Ok((actual_service_name.to_string(), processed_middlewares))
}

pub fn add_middlewares(
    traefik_config: &TraefikConfig,
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    router_name: &str,
    additional_middlewares: &[String],
    strip_prefix_name: Option<&str>,
    host_config: &HostConfig,
) -> TraefikResult<Vec<String>> {
    let mut middleware_idx = 0;
    let mut middleware_names = Vec::new();

    // Add strip prefix if provided
    if let Some(strip_name) = strip_prefix_name {
        middleware_names.push(strip_name.to_string());
        middleware_idx += 1;
    }

    if host_config.forward_host {
        pairs.push(EtcdPair::new(
            format!(
                // traefik/http/middlewares/host-helpdesk-herringbank-com-headers/headers/customRequestHeaders/X-Forwarded-Host
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Forwarded-Host",
                base_key, router_name
            ),
            host_config.domain.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                // traefik/http/middlewares/host-helpdesk-herringbank-com-headers/headers/customRequestHeaders/X-Forwarded-Proto
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Forwarded-Proto",
                base_key, router_name
            ),
            "https".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!(
                // traefik/http/middlewares/host-helpdesk-herringbank-com-headers/headers/customRequestHeaders/X-Original-Host
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Original-Host",
                base_key, router_name
            ),
            host_config.domain.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                // traefik/http/middlewares/host-helpdesk-herringbank-com-headers/headers/customRequestHeaders/X-Real-IP
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Real-IP",
                base_key, router_name
            ),
            "true".to_string(),
        ));
        middleware_idx += 1;
        middleware_names.push(format!("{}-headers", router_name));
    }

    println!(
        "traefik_config.middlewares: {:#?}",
        traefik_config.middlewares
    );

    // Add additional middlewares
    for middleware_name in additional_middlewares {
        println!("additional_middlewares: {:?}", additional_middlewares);
        match traefik_config.middlewares.get(middleware_name) {
            Some(middleware) => {
                let mut middleware = middleware.clone();
                middleware.set_name(&middleware_name);
                let mw_base_key = format!("{}/middlewares", base_key);
                debug!(
                    "Adding additional middleware: {} => base_key: {}",
                    middleware_name, mw_base_key
                );
                let mw_pairs = middleware.to_etcd_pairs(&mw_base_key)?;
                debug!("mw_pairs: {:?}", mw_pairs);
                pairs.extend(mw_pairs);

                pairs.push(EtcdPair::new(
                    format!(
                        "{}/routers/{}/middlewares/{}",
                        base_key, router_name, middleware_idx
                    ),
                    middleware_name.clone(),
                ));
                middleware_idx += 1;
                middleware_names.push(middleware_name.clone());
            }
            None => {
                // Strip prefix middleware is handled separately
                if !middleware_name.ends_with("-strip") {
                    return Err(TraefikError::MiddlewareConfig(format!(
                        "middleware {} not found",
                        middleware_name
                    )));
                }
            }
        }
    }

    Ok(middleware_names)
}

pub fn add_strip_prefix_middleware(
    pairs: &mut Vec<EtcdPair>,
    base_key: &str,
    path_safe_name: &str,
    path_config: Option<PathConfig>,
) -> TraefikResult<Option<String>> {
    let strip_prefix_name = if let Some(path_config) = path_config {
        let key = format!(
            "{}/middlewares/{}-strip/stripPrefix/prefixes/0",
            base_key, path_safe_name
        );
        let value = path_config.path.clone();
        debug!("Adding strip prefix middleware: {} => {}", key, value);
        pairs.push(EtcdPair::new(key, value));
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
    path_config: &PathConfig,
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

    use crate::{
        config::{
            headers::RuntimeHeadersConfig, middleware::MiddlewareConfig, services::ServiceConfig,
        },
        test_helpers::{
            assert_contains_pair, assert_does_not_contain_pair, create_complex_test_config,
            create_test_config, create_test_deployment, create_test_host, read_test_config,
        },
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
        let mut deployment1 = InternalDeploymentConfig {
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
        deployment1.init();

        let mut deployment2 = InternalDeploymentConfig {
            name: "test2".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            path_config: None,
            ..Default::default()
        };
        deployment2.init();

        let deployments = vec![deployment1, deployment2];

        add_deployment_rules(&host, &deployments, None, &mut pairs, base_key, &mut rules).unwrap();

        // Verify router configurations
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test1-router/entryPoints/0 websecure",
        );

        // Verify strip prefix middleware for path deployment
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-router-strip/stripPrefix/prefixes/0 /api",
        );

        // Verify service configurations
        // test/http/services/test1-router-service/loadBalancer/servers/0/url
        assert_contains_pair(
            &pairs,
            "test/http/services/test-example-com-test1-router-service/loadBalancer/servers/0/url http://10.0.0.1:8080",
        );
    }

    #[test]
    fn test_add_deployment_rules_with_strip_prefix() {
        let host = create_test_host();
        let base_key = "test";
        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        // Create test deployments
        let mut deployment1 = InternalDeploymentConfig {
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
        deployment1.init();

        let deployments = vec![deployment1];

        add_deployment_rules(&host, &deployments, None, &mut pairs, base_key, &mut rules).unwrap();

        // Verify strip prefix middleware for path deployment
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-router-strip/stripPrefix/prefixes/0 /api",
        );
    }

    #[test]
    fn test_global_service_config_deployment_rules_when_service_exists() {
        let mut config = create_test_config(None);
        config.services = Some(HashMap::new());
        config
            .services
            .as_mut()
            .unwrap()
            .insert("redirector".to_string(), ServiceConfig::default());
        let base_key = "test";
        // let mut pairs = Vec::new();

        let pairs = config.to_etcd_pairs(base_key).unwrap();

        // Verify strip prefix middleware for path deployment
        assert_contains_pair(
            &pairs,
            "test/http/services/redirector/loadBalancer/servers/0/url http://127.0.0.1:80",
        );
        assert_contains_pair(
            &pairs,
            "test/http/services/redirector/loadBalancer/passHostHeader true",
        );
    }

    #[test]
    fn test_global_service_config_deployment_rules_when_service_does_not_exist() {
        let config = create_test_config(None);
        let base_key = "test";

        let pairs = config.to_etcd_pairs(base_key).unwrap();

        // Verify strip prefix middleware for path deployment
        assert_does_not_contain_pair(
            &pairs,
            "test/services/redirector/loadBalancer/servers/0/url http://127.0.0.1:80",
        );
        assert_does_not_contain_pair(
            &pairs,
            "test/services/redirector/loadBalancer/passHostHeader true",
        );
    }

    #[test]
    fn test_forward_host_middleware() {
        let mut host = create_test_host();
        let base_key = "test";
        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();
        host.forward_host = true;

        // Create test deployments
        let mut deployment1 = InternalDeploymentConfig {
            name: "test1".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };
        deployment1.init();
        add_deployment_rules(
            &host,
            &[deployment1],
            None,
            &mut pairs,
            base_key,
            &mut rules,
        )
        .unwrap();

        debug!("{:#?}", pairs);
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-router-headers/headers/customRequestHeaders/X-Forwarded-Host test.example.com",
        );
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-router-headers/headers/customRequestHeaders/X-Real-IP true",
        );
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-router-headers/headers/customRequestHeaders/X-Forwarded-Proto https",
        );
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-router-headers/headers/customRequestHeaders/X-Original-Host test.example.com",
        );
    }

    #[test]
    fn test_attach_middlewares_attaches_to_the_router() {
        let mut pairs = Vec::new();
        attach_middlewares(
            &mut pairs,
            "test",
            "router",
            &["middleware1".to_string(), "middleware2".to_string()],
        )
        .unwrap();

        assert_contains_pair(&pairs, "test/routers/router/middlewares/0 middleware1");
        assert_contains_pair(&pairs, "test/routers/router/middlewares/1 middleware2");
    }

    #[test]
    fn test_add_deployment_rules_with_middlewares() {
        let mut test_traefik_config = read_test_config();
        let mut host = create_test_host();
        host.middlewares = vec!["test-middleware".to_string()];

        test_traefik_config
            .middlewares
            .insert("test-middleware".to_string(), MiddlewareConfig::default());

        test_traefik_config.hosts.push(host.clone());

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            traefik_config: test_traefik_config,
            ..Default::default()
        };
        deployment.init();

        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        add_deployment_rules(&host, &[deployment], None, &mut pairs, "test", &mut rules).unwrap();

        // Verify middleware configuration
        // test/http/routers/test-example-com-test-router/middlewares/0
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/middlewares/0 test-middleware",
        );
    }

    #[test]
    fn test_set_deployment_to_global_service_that_does_not_exist() {
        let mut test_traefik_config = read_test_config();
        let host = create_test_host();

        let mut services = HashMap::new();
        services.insert("another-service".to_string(), ServiceConfig::default());
        test_traefik_config.services = Some(services.clone());
        test_traefik_config.hosts.push(host.clone());

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: DeploymentConfig {
                name: "test".to_string(),
                target: DeploymentTarget::Service {
                    service_name: "redirector".to_string(),
                },
                ..Default::default()
            },
            host_config: host.clone(),
            traefik_config: test_traefik_config,
            ..Default::default()
        };
        deployment.init();

        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        let result = add_deployment_rules(
            &host,
            &[deployment],
            Some(&services),
            &mut pairs,
            "test",
            &mut rules,
        );
        // Global service does not exist
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Service config error: Service redirector not found".to_string()
        );
    }

    #[test]
    fn test_set_deployment_to_global_service_that_exists() {
        let mut test_traefik_config = read_test_config();
        let host = create_test_host();

        let mut services = HashMap::new();
        services.insert("redirector".to_string(), ServiceConfig::default());
        test_traefik_config.services = Some(services.clone());
        test_traefik_config.hosts.push(host.clone());

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: DeploymentConfig {
                name: "test".to_string(),
                target: DeploymentTarget::Service {
                    service_name: "redirector".to_string(),
                },
                ..Default::default()
            },
            host_config: host.clone(),
            traefik_config: test_traefik_config,
            ..Default::default()
        };
        deployment.init();

        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        let result = add_deployment_rules(
            &host,
            &[deployment],
            Some(&services),
            &mut pairs,
            "test",
            &mut rules,
        );
        // Global service exists
        assert!(result.is_ok());
        println!("pairs: {:#?}", pairs);
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/service redirector",
        );
    }

    #[test]
    fn test_add_deployment_rules_empty_deployments() {
        let host = create_test_host();
        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();

        let result = add_deployment_rules(&host, &[], None, &mut pairs, "test", &mut rules);
        assert!(result.is_ok());
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_deployment_rules_and_services() {
        let mut host = create_test_host();
        host.domain = "domain.com".to_string();
        let base_key = "test";
        let mut pairs = Vec::new();
        let mut rules = RuleConfig::default();
        rules.add_host_rule(&host.domain);

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };
        deployment.init();

        add_deployment_rules(&host, &[deployment], None, &mut pairs, base_key, &mut rules).unwrap();

        // Verify router rule exists
        assert_contains_pair(
            &pairs,
            "test/http/routers/domain-com-test-router/rule Host(`domain.com`)",
        );

        // Verify service exists with correct URL
        assert_contains_pair(
            &pairs,
            "test/http/services/domain-com-test-router-service/loadBalancer/servers/0/url http://10.0.0.1:8080",
        );

        // Verify router is linked to service
        assert_contains_pair(
            &pairs,
            "test/http/services/domain-com-test-router-service/loadBalancer/passHostHeader true",
        );
    }

    #[test]
    fn test_internal_deployment_to_etcd_pairs() {
        let host = create_test_host();
        let base_key = "test";

        let rule_config = RuleConfig::default();

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host,
            rules: rule_config,
            ..Default::default()
        };
        deployment.init();

        let pairs = deployment.to_etcd_pairs(base_key).unwrap();

        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/rule Host(`test.example.com`)",
        );
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/entryPoints/0 websecure",
        );
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/tls true",
        );
        assert_contains_pair(
            &pairs,
            "test/http/services/test-example-com-test-router-service/loadBalancer/passHostHeader true",
        );
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/priority 1010",
        );
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/service test-example-com-test-router-service",
        );
    }

    #[test]
    fn test_middleware_template_processing_with_service_context() {
        let deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: DeploymentConfig {
                target: DeploymentTarget::IpAndPort {
                    ip: "10.0.0.1".to_string(),
                    port: 8080,
                },
                middleware_templates: Some(HashMap::from([(
                    "headers".to_string(),
                    MiddlewareConfig {
                        runtime_headers: Some(RuntimeHeadersConfig {
                            template_headers: HashMap::from([(
                                "X-ServiceUrl".to_string(),
                                "http://{{ deployment.service.ip }}:{{ deployment.service.port }}"
                                    .to_string(),
                            )]),
                        }),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
            ..Default::default()
        };

        let processed = deployment.process_middleware_templates().unwrap();
        assert_eq!(processed.len(), 1);

        let (name, middleware) = &processed[0];
        assert_eq!(name, "test-headers-templated");

        // Check the processed headers instead of runtime_headers
        if let Some(headers) = &middleware.headers {
            assert_eq!(
                headers.custom_request_headers.get("X-ServiceUrl").unwrap(),
                "http://10.0.0.1:8080"
            );
        } else {
            panic!("Headers not found in processed middleware");
        }
    }
    #[test]
    fn test_middleware_template_processing_with_traefik_context() {
        let deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            traefik_config: TraefikConfig {
                name: Some("test".to_string()),
                ..Default::default()
            },
            deployment: DeploymentConfig {
                target: DeploymentTarget::IpAndPort {
                    ip: "10.0.0.1".to_string(),
                    port: 8080,
                },
                middleware_templates: Some(HashMap::from([(
                    "headers".to_string(),
                    MiddlewareConfig {
                        runtime_headers: Some(RuntimeHeadersConfig {
                            template_headers: HashMap::from([(
                                "X-ServiceName".to_string(),
                                "{{ traefik_config.name }}".to_string(),
                            )]),
                        }),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
            ..Default::default()
        };

        let processed = deployment.process_middleware_templates().unwrap();
        assert_eq!(processed.len(), 1);

        let (name, middleware) = &processed[0];
        assert_eq!(name, "test-headers-templated");

        // Check the processed headers instead of runtime_headers
        if let Some(headers) = &middleware.headers {
            assert_eq!(
                headers.custom_request_headers.get("X-ServiceName").unwrap(),
                "test"
            );
        } else {
            panic!("Headers not found in processed middleware");
        }
    }
}
