use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace};

use crate::{
    core::{
        client::StoreClient,
        etcd_trait::{EtcdPair, ToEtcdPairs},
        rules::{add_deployment_rules, get_sorted_deployments, RouterRule},
        templating::{TemplateContext, TemplateOr, TemplateResolver, TeraResolver},
        Validate,
    },
    error::{TraefikError, TraefikResult},
    features::etcd::{self, Etcd},
};

use super::{
    deployment::{DeploymentConfig, DeploymentProtocol, DeploymentTarget},
    entry_points::EntryPointsConfig,
    host::{HostConfig, PathConfig},
    middleware::MiddlewareConfig,
    services::ServiceConfig,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct TraefikConfigVersion {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct ConfigVersionHistory {
    pub id: i64,
    pub config_id: i64,
    pub name: String,
    pub config: serde_json::Value,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    pub version: i32,
}

impl ConfigVersionHistory {
    pub fn new(config_id: i64, name: String, config: serde_json::Value, version: i32) -> Self {
        Self {
            id: 0, // Will be set by database
            config_id,
            name,
            config,
            created_at: Utc::now(),
            version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct TraefikConfig {
    #[serde(default = "default_name")]
    pub name: Option<String>,
    #[serde(default = "default_description")]
    pub description: Option<String>,
    #[serde(default = "default_rule_prefix")]
    pub rule_prefix: String,
    #[cfg(feature = "etcd")]
    #[serde(default = "default_etcd_config")]
    pub etcd: etcd::EtcdConfig,
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, TemplateOr<String>>>,
    #[serde(default)]
    pub middlewares: HashMap<String, MiddlewareConfig>,
    #[serde(default)]
    pub services: Option<HashMap<String, ServiceConfig>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_points: Option<EntryPointsConfig>,
}

fn default_etcd_config() -> etcd::EtcdConfig {
    etcd::EtcdConfig::default()
}

fn default_rule_prefix() -> String {
    "traefik".to_string()
}

fn default_name() -> Option<String> {
    None
}

fn default_description() -> Option<String> {
    None
}

impl TraefikConfig {
    pub fn resolver(&self) -> TraefikResult<TeraResolver> {
        let resolver = TeraResolver::new()?;
        Ok(resolver)
    }

    pub fn context(&self) -> TraefikResult<TemplateContext> {
        let env_vars = vec!["SERVICE_HOST", "SERVICE_PORT"];
        let mut context = TemplateContext::new(self.clone(), env_vars)?;
        if let Some(variables) = &self.variables {
            for (key, value) in variables.iter() {
                context.insert_variable(key, value);
            }
        }
        Ok(context)
    }
}

impl TraefikConfig {
    pub fn get_service(&self, service_name: &str) -> Option<&ServiceConfig> {
        self.services
            .as_ref()
            .and_then(|services| services.get(service_name))
    }
}

impl From<TraefikConfig> for serde_json::Value {
    fn from(config: TraefikConfig) -> Self {
        serde_json::to_value(config).unwrap()
    }
}

impl ToEtcdPairs for TraefikConfig {
    fn to_etcd_pairs(
        &self,
        base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        // let mut pairs: Vec<EtcdPair> = Vec::new();
        let mut rule_set: HashSet<EtcdPair> = HashSet::new();
        let mut pairs: Vec<EtcdPair> = rule_set.clone().into_iter().collect();

        // Add global pairs
        // pairs.push(EtcdPair::new(base_key, "true"));
        // rule_set.insert(EtcdPair::new(base_key, "true"));
        // pairs.push(EtcdPair::new(format!("{}/http", base_key), "true"));
        // rule_set.insert(EtcdPair::new(format!("{}/http", base_key), "true"));
        let mut context = context.clone();

        // Add services
        debug!("Adding global services");
        if let Some(services) = &self.services {
            for (service_name, service) in services.iter() {
                let mut service = service.clone();
                service.set_name(service_name);
                debug!("Adding global service: {}", service_name);
                let service_base_key = format!("{}/http", base_key);
                let service_pairs = service.to_etcd_pairs(&service_base_key, resolver, &context)?;
                pairs.extend(service_pairs.clone());
                rule_set.extend(service_pairs.iter().cloned());
            }
        }

        let mut rule_set: HashSet<EtcdPair> = rule_set.clone();
        let sorted_hosts = get_sorted_deployments(self)?;

        // Run through all deployments and set the variables globally
        for deployment in sorted_hosts.iter() {
            if let Some(variables) = &deployment.variables {
                for (key, value) in variables.iter() {
                    let resolved_value = value.resolve(resolver, &context)?;
                    debug!("Inserting variable: {} = {}", key, resolved_value);
                    context.insert_variable(key, resolved_value);
                }
            }
        }

        // self.add_defaults(&mut pairs, base_key)?;
        // Start with middleware rules
        debug!("Adding middleware rules");
        // // TODO: add middleware before deployment rules, within the scope of a deployment
        for (name, middleware) in self.middlewares.clone().iter_mut() {
            middleware.set_name(name);
            let middleware_base_key = format!("{}/http/middlewares/{}", base_key, name);
            let new_rules = middleware.to_etcd_pairs(&middleware_base_key, resolver, &context)?;
            debug!("New rules middleware rules: {:?}", new_rules);
            for new_rule in new_rules.iter().cloned() {
                pairs.push(new_rule.clone());
                rule_set.insert(new_rule);
            }
        }

        let mut deployments = vec![];
        for deployment_config in sorted_hosts.iter() {
            deployments.push(deployment_config.clone());
        }

        let deployment_pairs =
            add_deployment_rules(&mut deployments, base_key, resolver, &mut context)?;
        pairs.extend(deployment_pairs.clone());
        rule_set.extend(deployment_pairs.iter().cloned());

        // Ok(pairs)
        Ok(rule_set.into_iter().collect())
    }
}

impl TraefikConfig {
    pub fn validate_config(&self) -> TraefikResult<()> {
        let mut resolver = self.resolver()?;
        let context = self.context()?;
        self.validate(&mut resolver, &context)
    }
}

impl Validate for TraefikConfig {
    fn validate(
        &self,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<()> {
        let mut validation_context = context.clone();
        // Validate services
        debug!("Validating services");
        if let Some(services) = &self.services {
            let mut services = services.clone();
            for (name, service) in services.iter_mut() {
                service.set_name(name);
                service.validate(resolver, &mut validation_context)?;
            }
        }

        // Validate middlewares
        // Because middleware validation is done in the deployment validation, we don't need to validate them here
        // debug!("Validating middlewares");
        // let mut middlewares = self.middlewares.clone();
        // for (name, middleware) in middlewares.iter_mut() {
        //     middleware.set_name(name);
        //     middleware.validate(resolver, &mut validation_context)?;
        // }

        // Validate hosts
        debug!("Validating hosts");
        let mut domain_set: HashSet<String> = HashSet::new();
        for host in self.hosts.iter() {
            if !domain_set.insert(host.domain.clone()) {
                return Err(TraefikError::HostConfig(format!(
                    "duplicate host: {}",
                    host.domain
                )));
            }

            // Validate host
            validation_context.set_host(host.clone());
            host.validate(resolver, &mut validation_context)?;

            // Validate host middleware references
            self.validate_middleware_references(host)?;
        }

        Ok(())
    }
}

impl TraefikConfig {
    pub fn validate_middleware_references(&self, host: &HostConfig) -> TraefikResult<()> {
        // Validate host middleware references
        // for middleware in host.middlewares.iter() {
        //     self.validate_middleware_references_in_host(middleware)?;
        // }
        Ok(())
    }

    // fn validate_middleware_references_in_host(&self, middleware_name: &str) -> TraefikResult<()> {
    //     if !self.middlewares.contains_key(middleware_name) {
    //         return Err(TraefikError::MiddlewareConfig(format!(
    //             "middleware {} not found",
    //             middleware_name
    //         )));
    //     }
    //     Ok(())
    // }
}

impl TraefikConfig {
    pub async fn clean_etcd(&self, client: &StoreClient<Etcd>) -> TraefikResult<()> {
        client.delete_with_prefix(self.rule_prefix.as_str()).await?;
        Ok(())
    }

    pub async fn apply_to_etcd(
        &mut self,
        client: &StoreClient<Etcd>,
        dry_run: bool,
        show_rules: bool,
        should_clean: bool,
    ) -> TraefikResult<()> {
        trace!("applying to etcd: {:#?}", self);
        let mut resolver = self.resolver()?;
        debug!("Resolved template");
        let context = self.context()?;
        debug!("Created context");
        self.validate(&mut resolver, &context)?;
        debug!("Validated config");
        let pairs = self.to_etcd_pairs(&self.rule_prefix, &mut resolver, &context)?;
        debug!("Generated pairs");
        let rules = RouterRule::from_pairs(&pairs);

        let mut rule_to_priority: HashMap<String, i32> = HashMap::new();

        // Build priority map
        for rule in &rules {
            rule_to_priority.insert(rule.get_rule().clone(), rule.get_priority());
        }

        if dry_run {
            if show_rules {
                for rule in rules.iter() {
                    println!(
                        "Rule = {} (priority: {})",
                        rule.get_rule(),
                        rule.get_priority()
                    );
                }
            } else {
                for pair in &pairs {
                    println!("Would set: {} = {}", pair.key(), pair.value());
                }
            }
            return Ok(());
        }

        if should_clean {
            self.clean_etcd(client).await?;
        }

        let mut client = client.actor.client.clone();

        for pair in pairs.iter() {
            debug!("applying: {:#?}", pair.to_string());
            match client.put(pair.key(), pair.value(), None).await {
                Ok(_kv) => {}
                Err(e) => error!("error: {:?}", e),
            }
        }

        Ok(())
    }
}

impl TraefikConfig {
    pub fn parse_etcd_to_traefik_config(pairs: Vec<EtcdPair>) -> TraefikResult<TraefikConfig> {
        let mut config_map: HashMap<String, HostConfig> = HashMap::new();

        for pair in pairs {
            let key = pair.key();
            let value = pair.value();

            if !key.starts_with("traefik/http/services/host-") {
                continue;
            }

            let service_parts = key.split('/').collect::<Vec<_>>();
            debug!("Service parts: {:?}", service_parts);

            // Extract the service name portion after 'host-'
            let service_name = match service_parts.get(3) {
                Some(name) => {
                    debug!("Found service name: {}", name);
                    name.strip_prefix("host-").unwrap_or(name)
                }
                None => {
                    debug!("No service name found in key: {}", key);
                    continue;
                }
            };

            debug!("service_name: {}", service_name);

            // Parse the domain and deployment parts
            let mut parts = service_name.split('-').collect::<Vec<_>>();
            if parts.is_empty() {
                continue;
            }

            // Last part should be the deployment index, remove it
            parts.pop();
            // Second to last part should be the deployment color
            let deployment_name = if let Some(color) = parts.pop() {
                color.to_string()
            } else {
                continue;
            };

            // Reconstruct domain from remaining parts
            let domain = parts.join(".");

            // Get or create host config
            let host_config = config_map
                .entry(domain.clone())
                .or_insert_with(|| HostConfig {
                    domain: domain.clone(),
                    deployments: HashMap::new(),
                    paths: Vec::new(),
                    middlewares: Vec::new(),
                    selection: None,
                    forward_host: true,
                    variables: None,
                });

            // Parse deployment if this is a URL entry
            if key.ends_with("/url") {
                let url_parts: Vec<&str> = value.split("://").collect();
                if url_parts.len() == 2 {
                    let host_port: Vec<&str> = url_parts[1].split(':').collect();
                    let ip = host_port[0].to_string();
                    let port = host_port
                        .get(1)
                        .and_then(|p| p.trim_end_matches("/").parse().ok())
                        .unwrap_or(80);

                    let deployment = DeploymentConfig::builder()
                        .name(deployment_name.clone())
                        .ip_and_port(ip, port)
                        .protocol(DeploymentProtocol::Http)
                        .weight(100)
                        .build();

                    host_config
                        .deployments
                        .insert(deployment_name.clone(), deployment);
                }
            }
        }

        Ok(TraefikConfig {
            hosts: config_map.into_values().collect(),
            ..Default::default()
        })
    }
}

impl TraefikConfig {
    pub fn generate_config(domain: Option<String>) -> TraefikConfig {
        let domain = domain.unwrap_or_else(|| "your-domain.com".to_string());

        let host_config = HostConfig::builder()
            .domain(domain)
            .path(
                "/api".to_string(),
                PathConfig::builder()
                    .path("/api".to_string())
                    .deployment(
                        "blue".to_string(),
                        DeploymentConfig::builder()
                            .ip_and_port("10.0.0.1".to_string(), 80)
                            .weight(100)
                            .build(),
                    )
                    .build(),
            )
            .deployment(
                "default".to_string(),
                DeploymentConfig::builder()
                    .ip_and_port("10.0.0.1".to_string(), 80)
                    .weight(100)
                    .build(),
            )
            .build()
            .unwrap();

        // Demo
        let host_configs = vec![host_config];

        TraefikConfig {
            name: Some("test".to_string()),
            description: Some("test".to_string()),
            #[cfg(feature = "etcd")]
            etcd: Default::default(),
            middlewares: HashMap::new(),
            hosts: host_configs,
            rule_prefix: "test".to_string(),
            services: None,
            entry_points: None,
            variables: None,
        }
    }
}

impl TraefikConfig {
    pub fn into_graph(
        &self,
        provide_dot: bool,
    ) -> TraefikResult<(petgraph::Graph<String, String>, Option<String>)> {
        let mut graph = petgraph::Graph::new();
        let mut resolver = self.resolver()?;
        let context = self.context()?;
        for host in &self.hosts {
            let host_node = graph.add_node(host.domain.clone());

            for path in &host.paths {
                let path_node = graph.add_node(path.path.clone());
                graph.add_edge(host_node, path_node, "path".to_string());
                for (deployment_name, deployment) in &path.deployments {
                    // let deployment_node = graph.add_node(deployment_name.clone());
                    for (_path_deployment_name, _path_deployment) in &path.deployments {
                        self.deployment_into_graph(
                            &mut graph,
                            path_node,
                            deployment_name,
                            deployment,
                            &mut resolver,
                            &context,
                        );
                    }
                }
            }
            for (deployment_name, deployment) in &host.deployments {
                self.deployment_into_graph(
                    &mut graph,
                    host_node,
                    deployment_name,
                    deployment,
                    &mut resolver,
                    &context,
                );
            }
        }
        if provide_dot {
            let graph_clone = graph.clone();
            let dot_graph = petgraph::dot::Dot::with_config(
                &graph_clone,
                &[petgraph::dot::Config::EdgeNoLabel],
            );

            Ok((graph, Some(dot_graph.to_string())))
        } else {
            Ok((graph, None))
        }
    }

    fn deployment_into_graph(
        &self,
        graph: &mut petgraph::Graph<String, String>,
        root_node: petgraph::graph::NodeIndex,
        deployment_name: &str,
        deployment: &DeploymentConfig,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) {
        let path_deployment_node = graph.add_node(deployment_name.to_string());
        // TODO: add middlewares
        let mut into_service_node = path_deployment_node;
        if let Some(middlewares) = &deployment.middlewares {
            for middleware in middlewares {
                let middleware_node = graph.add_node(middleware.clone());
                graph.add_edge(into_service_node, middleware_node, "middleware".to_string());
                into_service_node = middleware_node;
            }
        }
        let service_name = match &deployment.target {
            DeploymentTarget::Service { service_name } => service_name.clone(),
            DeploymentTarget::IpAndPort { ip, port } => {
                format!("{}:{}", ip, port)
            }
        };
        let service_node = graph.add_node(service_name.clone());
        graph.add_edge(into_service_node, service_node, "service".to_string());
        graph.add_edge(
            root_node,
            path_deployment_node,
            deployment.weight.to_string(),
        );
        let mut backend_name = String::new();
        if let Some(variables) = &deployment.variables {
            backend_name = variables
                .iter()
                .map(|(k, v)| format!("{}={:?} ", k, v.resolve(resolver, &context)))
                .collect::<Vec<String>>()
                .join(" ");
        }
        let backend_node = graph.add_node(backend_name.to_string());
        graph.add_edge(service_node, backend_node, backend_name);
    }
}

impl TraefikConfig {
    pub fn builder() -> TraefikConfigBuilder {
        TraefikConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct TraefikConfigBuilder {
    pub rule_prefix: String,
    pub hosts: Vec<HostConfig>,
    pub middlewares: HashMap<String, MiddlewareConfig>,
    pub services: Option<HashMap<String, ServiceConfig>>,
}

impl TraefikConfigBuilder {
    pub fn rule_prefix(mut self, rule_prefix: String) -> Self {
        self.rule_prefix = rule_prefix;
        self
    }

    pub fn hosts(mut self, hosts: Vec<HostConfig>) -> Self {
        self.hosts = hosts;
        self
    }

    pub fn middlewares(mut self, middlewares: HashMap<String, MiddlewareConfig>) -> Self {
        self.middlewares = middlewares;
        self
    }

    pub fn services(mut self, services: Option<HashMap<String, ServiceConfig>>) -> Self {
        self.services = services;
        self
    }

    pub fn build(&self) -> TraefikResult<TraefikConfig> {
        Ok(TraefikConfig {
            hosts: self.hosts.clone(),
            middlewares: self.middlewares.clone(),
            services: self.services.clone(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{
            deployment::{DeploymentProtocol, DeploymentTarget},
            host::HostConfigBuilder,
        },
        core::templating::TemplateOr,
        test_helpers::{create_test_resolver, create_test_template_context},
    };

    use super::*;

    #[test]
    fn test_validate_middleware_references() {
        let config = TraefikConfig::default();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = config.validate(&mut resolver, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_middleware_references_duplicate_host() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        config.hosts.push(HostConfig::default());
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = config.validate(&mut resolver, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_duplicate_middleware_in_host() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = config.validate(&mut resolver, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_middleware_not_found() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = config.validate(&mut resolver, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_middleware_not_found_in_host() {
        let mut config = TraefikConfig::default();
        config.hosts.push(HostConfig::default());
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = config.validate(&mut resolver, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_middleware_references_middleware_found_in_host() {
        let mut config = TraefikConfig::default();
        config
            .middlewares
            .insert("test".to_string(), MiddlewareConfig::default());
        config.hosts.push(
            HostConfigBuilder::default()
                .middleware("test".to_string())
                .build()
                .unwrap(),
        );
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = config.validate(&mut resolver, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_with_complex_config() {
        let config_str = r#"
        etcd:
            endpoints: ["https://0.0.0.0:2379"]
            timeout: 2000
            keep_alive: 300
            tls:
                cert: "./config/tls/etcd-peer.pem"
                key: "./config/tls/etcd-peer-key.pem"
                ca: "./config/tls/ca.pem"
                domain: herringbank.com
        middlewares:
        enable-headers:
            additional_headers:
                frameDeny: true
                browserXssFilter: true
                contentTypeNosniff: true
                forceSTSHeader: true
                stsIncludeSubdomains: true
                stsPreload: true
                stsSeconds: 31536000
                customFrameOptionsValue: "SAMEORIGIN"
            custom_request_headers:
                X-Forwarded-Proto: "https"
                X-Forwarded-Port: "443"
                Location: ""

        hosts:
            - domain: ari.io
              paths:
                - path: /
                  deployments:
                    green_with_cookie:
                        ip: 10.0.0.1
                        port: 80
                        weight: 100
              deployments:
                green:
                    protocol: http
                    ip: 10.0.0.1
                    port: 80
                    weight: 50
                blue:
                    protocol: http
                    ip: 10.0.0.1
                    port: 80
                    weight: 50
        "#;
        let config: TraefikConfig = serde_yaml::from_str(config_str).unwrap();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let validation_result = config.validate(&mut resolver, &context);
        assert!(validation_result.is_ok());
        assert_eq!(config.hosts.len(), 1);
        assert_eq!(config.hosts[0].domain, "ari.io");
        assert_eq!(config.hosts[0].deployments.len(), 2);
        let green = config.hosts[0].deployments["green"].clone();
        let (ip, port) = match &green.target {
            DeploymentTarget::IpAndPort { ip, port } => (ip, port),
            _ => unreachable!(),
        };
        assert_eq!(*port, 80);
        assert_eq!(ip, "10.0.0.1");
        assert_eq!(config.hosts[0].deployments["blue"].weight, 50);
        assert_eq!(
            config.hosts[0].deployments["green"].protocol,
            DeploymentProtocol::Http
        );
        assert_eq!(
            config.hosts[0].deployments["blue"].protocol,
            DeploymentProtocol::Http
        );
        let paths = config.hosts[0].paths.iter().find(|p| p.path == "/");
        assert!(paths.is_some());
        let path = paths.unwrap();
        assert_eq!(path.deployments.len(), 1);
        let green_with_cookie = path.deployments["green_with_cookie"].clone();
        let (ip, port) = match &green_with_cookie.target {
            DeploymentTarget::IpAndPort { ip, port } => (ip, port),
            _ => unreachable!(),
        };
        assert_eq!(*port, 80);
        assert_eq!(ip, "10.0.0.1");
    }

    #[test]
    fn test_validate_middleware_references_www_redirect() {
        let config_str = include_str!("../../config/config.yml");
        let mut config: TraefikConfig = serde_yaml::from_str(config_str).unwrap();
        let blue_deployment = config.hosts[0].paths[0]
            .deployments
            .get_mut("blue")
            .unwrap();
        blue_deployment.variables = Some(HashMap::from([(
            "name".to_string(),
            TemplateOr::Static("9999".to_string()),
        )]));
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let validation_result = config.validate(&mut resolver, &context);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_config_can_be_serialized() {
        let config = TraefikConfig::generate_config(None);
        let serialized = serde_yaml::to_string(&config).unwrap();
        assert!(!serialized.is_empty());
        assert!(serialized.contains("domain: your-domain.com"));
    }

    #[test]
    fn test_parse_basic_config() {
        let pairs = vec![
            EtcdPair::new(
                "traefik/http/services/host-example-com-blue-0/loadBalancer/servers/0/url",
                "http://redirector:3000",
            ),
            EtcdPair::new(
                "traefik/http/services/host-example-com-blue-0/loadBalancer/passHostHeader",
                "true",
            ),
        ];

        let config = TraefikConfig::parse_etcd_to_traefik_config(pairs).unwrap();
        assert_eq!(config.hosts.len(), 1);
        let host = &config.hosts[0];
        assert_eq!(host.domain, "example.com");
        assert_eq!(host.deployments.len(), 1);
        assert!(host.deployments.contains_key("blue"));

        let deployment = &host.deployments["blue"];
        let (ip, port) = match &deployment.target {
            DeploymentTarget::IpAndPort { ip, port } => (ip, port),
            _ => unreachable!(),
        };
        assert_eq!(ip, "redirector");
        assert_eq!(*port, 3000);
    }

    #[test]
    fn test_parse_multiple_deployments() {
        let pairs = vec![
            EtcdPair::new(
                "traefik/http/services/host-example-com-blue-0/loadBalancer/servers/0/url",
                "http://redirector:3000",
            ),
            EtcdPair::new(
                "traefik/http/services/host-example-com-green-0/loadBalancer/servers/0/url",
                "http://app:8080",
            ),
        ];

        let config = TraefikConfig::parse_etcd_to_traefik_config(pairs).unwrap();
        assert_eq!(config.hosts.len(), 1);
        let host = &config.hosts[0];
        assert_eq!(host.deployments.len(), 2);
        assert!(host.deployments.contains_key("blue"));
        assert!(host.deployments.contains_key("green"));
    }
}
