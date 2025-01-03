use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::config::deployment::{DeploymentProtocol, DeploymentTarget};
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
use super::templating::{TemplateContext, TemplateOr, TemplateResolver};

lazy_static::lazy_static! {
    static ref GLOBAL_SERVICES: Arc<Mutex<HashMap<String, ServiceConfig>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref GLOBAL_MIDDLEWARES: Arc<Mutex<HashMap<String, MiddlewareConfig>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

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

pub fn add_deployment_rules<R>(
    sorted_deployments: &mut [InternalDeploymentConfig],
    base_key: &str,
    resolver: &mut R,
    context: &TemplateContext,
) -> TraefikResult<Vec<EtcdPair>>
where
    R: TemplateResolver,
{
    let mut pairs = Vec::new();
    for internal_deployment in sorted_deployments.iter_mut() {
        let deployment_pairs =
            internal_deployment.process_deployment(base_key, resolver, context)?;
        pairs.extend(deployment_pairs);
    }
    Ok(pairs)
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
    /// The variables of the deployment
    #[serde(default)]
    pub variables: Option<HashMap<String, TemplateOr<String>>>,
    /// The headers of the deployment
    #[serde(skip)]
    _middlewares: HashMap<String, Option<MiddlewareConfig>>,
}

fn default_weight() -> usize {
    100
}

impl ToEtcdPairs for InternalDeploymentConfig {
    fn to_etcd_pairs(
        &self,
        base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut internal_deployment = self.clone();
        let internal_deployment = internal_deployment.init(resolver, context);
        let mut pairs = Vec::new();

        // Process middleware templates and add them to pairs
        let traefik_config = internal_deployment.traefik_config.clone();

        let root_middleware_key = format!("{}/{}", base_key, "middlewares");
        for (name, middleware) in traefik_config.middlewares.iter() {
            let base_middleware_key = format!("{}/{}", root_middleware_key, name);
            debug!(
                "Adding middleware In InternalDeploymentConfig: {} => {:#?}",
                name, middleware
            );
            pairs.extend(middleware.to_etcd_pairs(&base_middleware_key, resolver, context)?);
        }

        // for internal_deployment in sorted_deployments.iter() {
        let deployment_context = context.clone();
        let deployment_pairs =
            internal_deployment.process_deployment(base_key, resolver, &deployment_context)?;
        pairs.extend(deployment_pairs);

        // Remove duplicate EtcdPairs by converting to HashSet and back to Vec
        // let unique_pairs: HashSet<_> = pairs.into_iter().collect();
        // pairs = unique_pairs.into_iter().collect();
        Ok(pairs)
    }
}

impl InternalDeploymentConfig {
    pub fn process_deployment(
        &mut self,
        base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        // Initialize the pairs
        self.init(resolver, context);
        let rule = self.rules.clone();

        let mut collected_pairs: Vec<EtcdPair> = Vec::new();
        let base_key = format!("{}/{}", base_key, self.get_deployment_protocol());

        let context = self.create_deployment_context(context);

        let pairs = self.handle_middlewares(&base_key, resolver, &context)?;
        collected_pairs.extend(pairs);

        // Add service
        let service_pairs = self.add_service_configuration(&base_key)?;
        collected_pairs.extend(service_pairs);

        // Add root router
        let root_router_pairs = self.add_root_router(&base_key, &rule)?;
        collected_pairs.extend(root_router_pairs);

        Ok(collected_pairs)
    }

    fn handle_middlewares(
        &mut self,
        base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let mut collected_middlewares = self.collect_middlewares();

        // Deployment for path if it exists
        let mut internal_deployment = self.clone();
        if let Some(path_config) = &self.path_config {
            let pass_through_pairs = internal_deployment.add_pass_through_middleware(
                &base_key,
                &mut collected_middlewares,
                path_config,
            )?;
            pairs.extend(pass_through_pairs);
        }

        // Add deployment target base
        let base_service_pairs =
            self.create_base_service_configuration(&base_key, resolver, &context)?;
        pairs.extend(base_service_pairs);

        // Add deployment middlewares
        let deployment_middleware_pairs = self.add_deployment_middlewares(
            &base_key,
            &mut collected_middlewares,
            resolver,
            &context,
        )?;
        pairs.extend(deployment_middleware_pairs);

        // Add the middleware pairs
        let middleware_pairs =
            self.add_middleware_pairs(&base_key, &mut collected_middlewares, resolver, &context)?;
        pairs.extend(middleware_pairs);

        // Add the deployment rules
        let middleware_pairs: Vec<EtcdPair> =
            self.attach_middleware_names(&base_key, &collected_middlewares)?;
        pairs.extend(middleware_pairs);

        Ok(pairs)
    }

    fn add_service_configuration(&mut self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        match &self.deployment.target {
            DeploymentTarget::IpAndPort { .. } => {
                pairs.push(EtcdPair::new(
                    format!("{}/routers/{}/service", base_key, self.get_router_name()),
                    self.get_service_name().clone(),
                ));
            }
            DeploymentTarget::Service { service_name } => {
                pairs.push(EtcdPair::new(
                    format!("{}/routers/{}/service", base_key, self.get_router_name()),
                    service_name.clone(),
                ));
            }
        }
        Ok(pairs)
    }

    fn add_root_router(
        &mut self,
        base_key: &str,
        rule: &RuleConfig,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let router_name = self.get_router_name();
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
        Ok(pairs)
    }

    fn get_router_name(&self) -> String {
        if self.path_config.is_some() {
            return format!(
                "{}-{}-path-router",
                get_safe_key(&self.host_config.domain),
                get_safe_key(&self.name)
            );
        }
        format!(
            "{}-{}-router",
            get_safe_key(&self.host_config.domain),
            get_safe_key(&self.name)
        )
    }

    fn get_service_name(&self) -> String {
        if self.path_config.is_some() {
            return format!(
                "{}-{}-path-service",
                get_safe_key(&self.host_config.domain),
                get_safe_key(&self.name)
            );
        }
        format!(
            "{}-{}-service",
            get_safe_key(&self.host_config.domain),
            get_safe_key(&self.name)
        )
    }

    fn get_deployment_protocol(&self) -> DeploymentProtocol {
        self.deployment.protocol.clone()
    }

    fn get_safe_middleware_name(&self, middleware_name: &str) -> String {
        format!("{}-{}", self.get_router_name(), middleware_name)
    }

    fn add_pass_through_middleware(
        &mut self,
        base_key: &str,
        additional_middlewares: &mut Vec<String>,
        path_config: &PathConfig,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        if path_config.strip_prefix {
            let key = format!(
                "{}/middlewares/{}-strip/stripPrefix/prefixes/0",
                base_key,
                self.get_router_name()
            );
            let value = path_config.path.clone();
            debug!("Adding strip prefix middleware: {} => {}", key, value);
            let new_pair = EtcdPair::new(key, value);
            self._middlewares
                .insert(format!("{}-strip", self.get_router_name()), None);
            pairs.push(new_pair);
            additional_middlewares.push(format!("{}-strip", self.get_router_name()));
        }

        if path_config.pass_through {
            let key = format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Pass-Through",
                base_key,
                self.get_router_name()
            );
            let value = "true".to_string();
            debug!("Adding pass through middleware: {} => {}", key, value);
            let new_pair = EtcdPair::new(key, value);
            self._middlewares
                .insert(format!("{}-headers", self.get_router_name()), None);
            pairs.push(new_pair);
            additional_middlewares.push(format!("{}-headers", self.get_router_name()));
        }
        Ok(pairs)
    }

    fn add_deployment_middlewares(
        &mut self,
        base_key: &str,
        collected_middlewares: &mut Vec<String>,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        if self.host_config.forward_host {
            let forward_host_pairs =
                self.add_forward_host_middleware(collected_middlewares, base_key)?;
            pairs.extend(forward_host_pairs);
        }

        Ok(pairs)
    }

    fn add_middleware_pairs(
        &mut self,
        base_key: &str,
        collected_middlewares: &mut Vec<String>,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let additional_middlewares = collected_middlewares.clone();
        let middleware_names_clone = additional_middlewares.clone();
        let mut middleware_names_set = HashSet::new();

        for original_middleware_name in middleware_names_clone.iter() {
            let middleware_name = original_middleware_name;
            debug!(
                "Looking for middleware: {} => {}",
                original_middleware_name, middleware_name
            );

            // if let Some(middleware_name) = self.find_in_already_collected_middlewares(
            //     original_middleware_name,
            //     collected_middlewares,
            // ) {
            //     let new_middleware_name = self.get_safe_middleware_name(&middleware_name);
            //     pairs.push(EtcdPair::new(
            //         format!("{}/middlewares/{}", base_key, new_middleware_name),
            //         "true".to_string(),
            //     ));
            // } else
            // if self._middlewares.contains_key(original_middleware_name) {
            debug!("Middleware {} already exists", original_middleware_name);
            // continue;
            // Remove the middleware from the deployment config
            // if self._middlewares.contains_key(original_middleware_name) {
            //     let _ = self._middlewares.remove(original_middleware_name);
            // }
            if let Some(index) = collected_middlewares
                .iter()
                .position(|r| r == original_middleware_name)
            {
                let _ = collected_middlewares.remove(index);
            }
            // let _ = collected_middlewares.remove(
            // }
            // Attach global middleware
            println!("Looking for middleware: {}", original_middleware_name);
            if let Some((middleware_name, middleware)) =
                self.find_middleware_in_config(original_middleware_name)
            {
                if let Some(middleware) = middleware {
                    let mut middleware = middleware.clone();
                    let new_middleware_name = self.get_safe_middleware_name(&middleware_name);
                    middleware.set_name(&new_middleware_name);

                    let mw_base_key = format!("{}/middlewares/{}", base_key, new_middleware_name);
                    debug!(
                        "Adding middleware: {} => {}",
                        new_middleware_name, mw_base_key
                    );
                    let mw_pairs = middleware.to_etcd_pairs(&mw_base_key, resolver, context)?;
                    debug!("Adding middleware pairs: {:#?}", mw_pairs);
                    let mw_pairs_clone = mw_pairs.clone();
                    pairs.extend(mw_pairs);
                    for pair in mw_pairs_clone.iter() {
                        self._middlewares
                            .insert(pair.key().to_string(), Some(middleware.clone()));
                    }
                    middleware_names_set.insert(new_middleware_name.clone());
                }
            } else if original_middleware_name.ends_with("-strip") {
                // It's a strip middleware
                // and we created it
            } else {
                // It's not found in the traefik config, host config or deployment config
                // return Err(TraefikError::MiddlewareConfig(format!(
                //     "middleware {} not found",
                //     middleware_name
                // )));
            }
        }
        for middleware_name in middleware_names_set.iter() {
            collected_middlewares.push(middleware_name.clone());
        }
        debug!("Collected middlewares: {:#?}", collected_middlewares);
        Ok(pairs)
    }

    fn collect_middlewares(&self) -> Vec<String> {
        let mut middleware_names = Vec::new();
        if let Some(middlewares) = &self.deployment.middlewares {
            middleware_names.extend(middlewares.clone());
        }
        middleware_names.extend(self.host_config.middlewares.iter().cloned());
        if let Some(path_config) = &self.path_config {
            for middleware_name in path_config.middlewares.iter() {
                middleware_names.push(middleware_name.clone());
            }
        }
        middleware_names
    }

    fn find_middleware_in_config(
        &self,
        middleware_name: &str,
    ) -> Option<(String, Option<MiddlewareConfig>)> {
        if self._middlewares.contains_key(middleware_name) {
            return Some((middleware_name.to_string(), None));
        }
        if self.traefik_config_contains_middleware(middleware_name) {
            let middleware = self
                .traefik_config
                .middlewares
                .get(middleware_name)
                .unwrap()
                .clone();
            return Some((middleware_name.to_string(), Some(middleware)));
        }
        None
    }

    fn traefik_config_contains_middleware(&self, middleware_name: &str) -> bool {
        self.traefik_config
            .middlewares
            .contains_key(middleware_name)
    }

    fn attach_middleware_names(
        &mut self,
        base_key: &str,
        middleware_names: &Vec<String>,
    ) -> TraefikResult<Vec<EtcdPair>> {
        // Attach middleware names to the router
        let mut pairs = vec![];
        let mut sorted_middleware_names = middleware_names.clone();
        sorted_middleware_names.sort();

        for (idx, middleware_name) in sorted_middleware_names.iter().enumerate() {
            {
                pairs.push(EtcdPair::new(
                    format!(
                        "{}/routers/{}/middlewares/{}",
                        base_key,
                        self.get_router_name(),
                        idx
                    ),
                    middleware_name,
                ));
            }
        }

        Ok(pairs)
    }

    fn add_forward_host_middleware(
        &mut self,
        middleware_names: &mut Vec<String>,
        base_key: &str,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let router_name = self.get_router_name();
        let middleware_name = format!("{}-headers", router_name);
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Forwarded-Host",
                base_key, router_name
            ),
            self.host_config.domain.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Forwarded-Proto",
                base_key, router_name
            ),
            "https".to_string(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Original-Host",
                base_key, router_name
            ),
            self.host_config.domain.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/{}-headers/headers/customRequestHeaders/X-Real-IP",
                base_key, router_name
            ),
            "true".to_string(),
        ));
        middleware_names.push(middleware_name.clone());
        self._middlewares.insert(middleware_name, None);
        Ok(pairs)
    }

    fn create_base_service_configuration(
        &mut self,
        base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let deployment_service_name = self.get_service_name();
        debug!(
            "Adding base service configuration for {}",
            deployment_service_name
        );

        match &self.deployment.target {
            DeploymentTarget::IpAndPort { ip, port } => {
                let base_key = format!("{}/services/{}", base_key, deployment_service_name);
                debug!(
                    "Adding service {} pointing to http://{}:{}",
                    deployment_service_name, ip, port
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
                // let base_key = format!("{}/services/{}", base_key, service_name);
                debug!("Adding service {} pointing to {}", service_name, base_key);
                // Add the service configuration
                if let Some(services) = &self.traefik_config.services {
                    if let Some(service) = services.get(service_name) {
                        let mut service = service.clone();
                        service.set_name(&service_name);
                        let service_pairs = service.to_etcd_pairs(&base_key, resolver, context)?;
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

        Ok(pairs)
    }

    fn create_deployment_context(&mut self, context: &TemplateContext) -> TemplateContext {
        let mut deployment_context = context.clone();
        // Set deployment
        deployment_context.set_deployment(self.deployment.clone());
        // Set host
        deployment_context.set_host(self.host_config.clone());

        // Set the path config
        if let Some(path) = &self.path_config {
            deployment_context.set_path_config(path.clone());
        }

        // Set host variables
        if let Some(variables) = &self.host_config.variables {
            for (key, value) in variables.iter() {
                deployment_context.insert_variable(key, value.clone());
            }
        }

        // Set deployment variables
        if let Some(variables) = &self.deployment.variables {
            for (key, value) in variables.iter() {
                deployment_context.insert_variable(key, value.clone());
            }
        };

        deployment_context
    }
}

impl InternalDeploymentConfig {
    pub fn init(
        &mut self,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> &mut Self {
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
        // Add the variables
        if let Some(variables) = &self.variables {
            let mut new_variables = HashMap::new();
            for (key, value) in variables.iter() {
                new_variables.insert(key.clone(), value.clone());
            }
            self.variables = Some(new_variables);
        }
        self.rules = rules;
        // Add the weight of the rules to the weight of the deployment
        self.weight += 1000 + self.rules.get_weight();

        self
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
    let mut resolver = traefik_config.resolver()?;
    let context = traefik_config.context()?;

    let mut internal_deployments_result = Vec::new();
    for internal_deployment in internal_deployments.iter_mut() {
        // Add the host rule
        internal_deployments_result.push(internal_deployment.init(&mut resolver, &context).clone());
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
                variables: deployment.variables.clone(),
                _middlewares: HashMap::new(),
            });
        }
    }
    Ok(internal_deployments)
}

#[cfg(test)]
mod tests {

    #[allow(unused_imports)]
    use crate::test_helpers::init_test_tracing;
    use crate::{
        config::{headers::HeadersConfig, middleware::MiddlewareConfig, services::ServiceConfig},
        test_helpers::{
            assert_contains_pair, assert_does_not_contain_pair, create_complex_test_config,
            create_test_config, create_test_deployment, create_test_host, create_test_resolver,
            create_test_template_context, read_test_config,
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
    fn test_deployment_adds_rule_to_pairs() {
        let host = create_test_host();
        let base_key = "test";

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };
        deployment.init(&mut resolver, &context);

        let mut deployments = vec![deployment];
        let pairs = add_deployment_rules(&mut deployments, base_key, &mut resolver, &context);

        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();

        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/rule Host(`test.example.com`)",
        );
    }

    #[test]
    fn test_add_deployment_rules_from_internal_deployment_config() {
        let host = create_test_host();
        let base_key = "test";

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

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
        deployment1.init(&mut resolver, &context);

        let mut deployment2 = InternalDeploymentConfig {
            name: "test2".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            path_config: None,
            ..Default::default()
        };
        deployment2.init(&mut resolver, &context);

        let mut deployments = vec![deployment1, deployment2];

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs =
            add_deployment_rules(&mut deployments, base_key, &mut resolver, &context).unwrap();

        // Verify router configurations
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test1-path-router/entryPoints/0 websecure",
        );

        // Verify strip prefix middleware for path deployment
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-path-router-strip/stripPrefix/prefixes/0 /api",
        );

        // Verify service configurations
        // test/http/services/test1-router-service/loadBalancer/servers/0/url
        assert_contains_pair(
            &pairs,
            "test/http/services/test-example-com-test1-path-service/loadBalancer/servers/0/url http://10.0.0.1:8080",
        );
    }

    #[test]
    fn test_add_deployment_rules_with_strip_prefix() {
        let host = create_test_host();
        let base_key = "test";

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

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
        deployment1.init(&mut resolver, &context);

        let mut deployments = vec![deployment1];
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs =
            add_deployment_rules(&mut deployments, base_key, &mut resolver, &context).unwrap();

        // Verify strip prefix middleware for path deployment
        assert_contains_pair(
            &pairs,
            "test/http/middlewares/test-example-com-test1-path-router-strip/stripPrefix/prefixes/0 /api",
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

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = config
            .to_etcd_pairs(base_key, &mut resolver, &context)
            .unwrap();

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

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = config
            .to_etcd_pairs(base_key, &mut resolver, &context)
            .unwrap();

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
        host.forward_host = true;

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

        // Create test deployments
        let mut deployment1 = InternalDeploymentConfig {
            name: "test1".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };
        deployment1.init(&mut resolver, &context);

        let mut deployments = vec![deployment1];

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs =
            add_deployment_rules(&mut deployments, base_key, &mut resolver, &context).unwrap();

        debug!("test_forward_host_middleware {:#?}", pairs);
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
        let (_host, _base_key, deployments, _resolver, _context) = create_test_env();
        let mut middleware_names = vec!["middleware1".to_string(), "middleware2".to_string()];
        let mut dp1 = deployments[0].clone();
        let pairs = dp1
            .attach_middleware_names("test/http", &mut middleware_names)
            .unwrap();

        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test1-router/middlewares/0 middleware1",
        );
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test1-router/middlewares/1 middleware2",
        );
    }

    #[test]
    fn test_add_deployment_rules_with_middlewares() {
        let mut test_traefik_config = read_test_config();
        let mut host = create_test_host();
        host.middlewares = vec!["default-headers".to_string()];

        test_traefik_config
            .middlewares
            .insert("default-headers".to_string(), MiddlewareConfig::default());

        test_traefik_config.hosts.push(host.clone());

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            traefik_config: test_traefik_config,
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        deployment.init(&mut resolver, &context);

        let mut deployments = vec![deployment];

        let pairs =
            add_deployment_rules(&mut deployments, "test", &mut resolver, &context).unwrap();

        // Verify middleware configuration
        // test/http/routers/test-example-com-test-router/middlewares/0

        let first_middleware_pair = pairs
            .iter()
            .find(|pair| pair.key().contains("middlewares/0"));
        assert!(first_middleware_pair.is_some());
        let middleware_pair = first_middleware_pair.unwrap();
        assert!(middleware_pair.value().contains("default-headers"));
        // assert_contains_pair(
        //     &pairs,
        //     "test/http/routers/test-example-com-test-router/middlewares/0 default-headers",
        // );
    }

    #[test]
    fn test_set_deployment_to_global_service_that_does_not_exist() {
        let mut test_traefik_config = read_test_config();
        let host = create_test_host();
        let mut services = HashMap::new();
        services.insert("another-service".to_string(), ServiceConfig::default());
        test_traefik_config.services = Some(services.clone());
        test_traefik_config.hosts.push(host.clone());
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

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
        deployment.init(&mut resolver, &context);

        let mut deployments = vec![deployment];

        let result = add_deployment_rules(&mut deployments, "test", &mut resolver, &context);
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
        // init_test_tracing();

        let mut test_traefik_config = read_test_config();
        let host = create_test_host();
        let mut services = HashMap::new();
        services.insert("redirector".to_string(), ServiceConfig::default());
        test_traefik_config.services = Some(services.clone());
        test_traefik_config.hosts.push(host.clone());

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

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
        deployment.init(&mut resolver, &context);

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let mut deployments = vec![deployment];
        let result = add_deployment_rules(&mut deployments, "test", &mut resolver, &context);
        // Global service exists
        assert!(result.is_ok());
        let pairs = result.unwrap();
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/service redirector",
        );
    }

    #[test]
    fn test_add_deployment_rules_empty_deployments() {
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let mut deployments = vec![];
        let result = add_deployment_rules(&mut deployments, "test", &mut resolver, &context);
        assert!(result.is_ok());
        let pairs = result.unwrap();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_deployment_rules_and_services() {
        let mut host = create_test_host();
        host.domain = "domain.com".to_string();
        let base_key = "test";

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };
        deployment.init(&mut resolver, &context);

        let mut deployments = vec![deployment];
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs =
            add_deployment_rules(&mut deployments, base_key, &mut resolver, &context).unwrap();

        // Verify router rule exists
        assert_contains_pair(
            &pairs,
            "test/http/routers/domain-com-test-router/rule Host(`domain.com`)",
        );

        // Verify service exists with correct URL
        assert_contains_pair(
            &pairs,
            "test/http/services/domain-com-test-service/loadBalancer/servers/0/url http://10.0.0.1:8080",
        );

        // Verify router is linked to service
        assert_contains_pair(
            &pairs,
            "test/http/services/domain-com-test-service/loadBalancer/passHostHeader true",
        );
    }

    #[test]
    fn test_internal_deployment_to_etcd_pairs() {
        let host = create_test_host();
        let base_key = "test";
        let test_traefik_config = create_test_config(Some(vec![host.clone()]));

        let rule_config = RuleConfig::default();

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: create_test_deployment(),
            host_config: host,
            rules: rule_config,
            traefik_config: test_traefik_config,
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        deployment.init(&mut resolver, &context);

        let result = deployment.to_etcd_pairs(base_key, &mut resolver, &context);
        assert!(result.is_ok());
        let pairs = result.unwrap();

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
            "test/http/services/test-example-com-test-service/loadBalancer/passHostHeader true",
        );
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/priority 1010",
        );
        assert_contains_pair(
            &pairs,
            "test/http/routers/test-example-com-test-router/service test-example-com-test-service",
        );
    }

    #[test]
    fn test_path_and_host_middleware_names_are_set_to_router_name() {
        // init_test_tracing();

        let mut traefik_config = create_test_config(None);
        let mut host = create_test_host();
        let base_key = "test";
        let rule_config = RuleConfig::default();

        let mut host_middleware = MiddlewareConfig::default();
        host_middleware.set_name("test-host-middleware");
        traefik_config
            .middlewares
            .insert("test-host-middleware".to_string(), host_middleware);
        host.middlewares = vec!["test-host-middleware".to_string()];
        traefik_config.hosts.push(host.clone());
        traefik_config.middlewares.insert(
            "test-host-middleware".to_string(),
            MiddlewareConfig::default(),
        );
        traefik_config.middlewares.insert(
            "test-deployment-middleware".to_string(),
            MiddlewareConfig::default(),
        );
        traefik_config.middlewares.insert(
            "test-path-middleware".to_string(),
            MiddlewareConfig::default(),
        );

        let header_config = HeadersConfig {
            additional_headers: HashMap::from([(
                "X-Forwarded-Host".to_string(),
                TemplateOr::Static("test.example.com".to_string()),
            )]),
            ..Default::default()
        };
        let mut path_middleware = MiddlewareConfig::default();
        path_middleware.set_name("test-path-middleware");
        path_middleware.headers = Some(header_config);

        let mut deployment_middleware = MiddlewareConfig::default();
        deployment_middleware.set_name("test-deployment-middleware");

        traefik_config
            .middlewares
            .insert("test-path-middleware".to_string(), path_middleware);

        let mut original_deployment = create_test_deployment();
        original_deployment.middlewares = Some(vec!["test-deployment-middleware".to_string()]);

        original_deployment.middlewares = Some(vec![
            "test-deployment-middleware".to_string(),
            "test-path-middleware".to_string(),
        ]);

        host.paths.push(PathConfig {
            path: "/api".to_string(),
            deployments: HashMap::from([("blue".to_string(), original_deployment.clone())]),
            middlewares: vec!["test-path-middleware".to_string()],
            strip_prefix: true,
            pass_through: false,
        });

        let mut deployment = InternalDeploymentConfig {
            name: "test".to_string(),
            deployment: original_deployment,
            host_config: host,
            rules: rule_config,
            traefik_config: traefik_config,
            ..Default::default()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        deployment.init(&mut resolver, &context);

        let pairs = deployment
            .to_etcd_pairs(base_key, &mut resolver, &context)
            .unwrap();

        let first_middleware_pair = pairs
            .iter()
            .find(|pair| pair.key().contains("middlewares/0"));
        assert!(first_middleware_pair.is_some());
        let middleware_pair = first_middleware_pair.unwrap();
        assert!(middleware_pair
            .value()
            .contains("test-deployment-middleware"));

        let second_middleware_pair = pairs
            .iter()
            .find(|pair| pair.key().contains("middlewares/1"));
        assert!(second_middleware_pair.is_some());
        let middleware_pair = second_middleware_pair.unwrap();
        assert!(middleware_pair.value().contains("test-host-middleware"));

        let third_middleware_pair = pairs
            .iter()
            .find(|pair| pair.key().contains("middlewares/2"));
        assert!(third_middleware_pair.is_some());
        let middleware_pair = third_middleware_pair.unwrap();
        assert!(middleware_pair.value().contains("test-path-middleware"));

        // assert_contains_pair(
        //     &pairs,
        //     "test/http/routers/test-example-com-test-router/middlewares/0 test-deployment-middleware",
        // );
        // assert_contains_pair(
        //     &pairs,
        //     "test/http/routers/test-example-com-test-router/middlewares/5 test-path-middleware",
        // );
    }

    fn create_test_env() -> (
        HostConfig,
        String,
        Vec<InternalDeploymentConfig>,
        impl TemplateResolver,
        TemplateContext,
    ) {
        let mut host = create_test_host();
        let base_key = "test";
        // let mut rules = RuleConfig::default();
        host.forward_host = true;

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();

        // Create test deployments
        let mut deployment1 = InternalDeploymentConfig {
            name: "test1".to_string(),
            deployment: create_test_deployment(),
            host_config: host.clone(),
            ..Default::default()
        };
        deployment1.init(&mut resolver, &context);

        let deployments = vec![deployment1];
        (host, base_key.to_string(), deployments, resolver, context)
    }
}
