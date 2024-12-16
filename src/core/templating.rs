use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::{
    config::{
        deployment::{DeploymentConfig, DeploymentTarget},
        host::{HostConfig, PathConfig},
        services::ServiceConfig,
    },
    error::{TraefikError, TraefikResult},
    TraefikConfig,
};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(untagged)]
pub enum TemplateOr<T> {
    Static(T),
    Template(String),
}

impl<T: Default> Default for TemplateOr<T> {
    fn default() -> Self {
        Self::Static(T::default())
    }
}

pub trait TemplateResolver {
    fn resolve_template(
        &mut self,
        template: &str,
        context: &TemplateContext,
    ) -> TraefikResult<String>;
}

impl<T: ToString> TemplateOr<T> {
    pub fn resolve(
        &self,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<String> {
        match self {
            Self::Static(value) => Ok(value.to_string()),
            Self::Template(template) => resolver.resolve_template(template, context),
        }
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
// pub struct ServiceContext {
//     service: ServiceConfig,
//     deployment: DeploymentConfig,
// }

// impl ServiceContext {
//     pub fn new(service: ServiceConfig, deployment: DeploymentConfig) -> Self {
//         Self {
//             service,
//             deployment,
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
pub struct DeploymentContext {
    service: ServiceConfig,
    name: String,
}

impl DeploymentContext {
    pub fn new(service: ServiceConfig, name: String) -> Self {
        Self { service, name }
    }
}

#[derive(Debug, Default)]
pub struct DeploymentContextBuilder {
    service: ServiceConfig,
    name: String,
}

impl DeploymentContextBuilder {
    pub fn service(mut self, service: ServiceConfig) -> Self {
        self.service = service;
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn build(self) -> DeploymentContext {
        DeploymentContext {
            service: self.service,
            name: self.name,
        }
    }
}

impl DeploymentContext {
    pub fn builder() -> DeploymentContextBuilder {
        DeploymentContextBuilder::default()
    }
}

#[derive(Debug, Clone, Default)]
struct InnerTemplateContext {
    deployment: DeploymentConfig,
    host: HostConfig,
    path: PathConfig,
    env: HashMap<String, String>,
    config: TraefikConfig,
    variables: HashMap<String, serde_json::Value>,
    service: ServiceConfig,
}

#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    inner: Arc<Mutex<InnerTemplateContext>>,
}

const FORBIDDEN_KEYS: [&str; 6] = [
    "deployment",
    "host",
    "path",
    "config",
    "variables",
    "service",
];
impl TemplateContext {
    /// Create a new template context
    ///
    /// This function initializes the template context with the provided environment variables and Traefik configuration.
    /// It also sets up the context with the provided environment variables and Traefik configuration.
    ///
    /// # Arguments
    ///
    /// * `traefik_config` - An optional `TraefikConfig` object.
    /// * `env_vars` - A vector of strings representing environment variables.
    ///
    /// # Returns
    ///
    /// A `TraefikResult` containing the newly created `TemplateContext` or an error if the context cannot be created.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use traefik_config::{config::traefik_config::TraefikConfig, core::templating::TemplateContext};
    /// let env_vars = vec!["SERVICE_HOST", "SERVICE_PORT"];
    /// let traefik_config = TraefikConfig::default();
    /// let context = TemplateContext::new(traefik_config, env_vars)?;
    /// ```
    pub fn new<T: ToString>(
        traefik_config: TraefikConfig,
        env_vars: Vec<T>,
    ) -> TraefikResult<Self> {
        let mut inner = InnerTemplateContext::default();

        // Initialize with environment variables
        for env_var in env_vars {
            let key = env_var.to_string();
            let value = std::env::var(&key).unwrap_or_else(|_| "".to_string());
            inner.env.insert(key, value);
        }

        inner.config = traefik_config;

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// Insert a variable into the template context
    ///
    /// This function inserts a variable into the template context.
    ///
    /// # Arguments
    ///
    /// * `key` - A string representing the key of the variable.
    /// * `value` - A value implementing the `Serialize` trait.
    pub fn insert_variable(&mut self, key: &str, value: impl Serialize) {
        let mut inner = self.inner.lock().unwrap();
        inner
            .variables
            .insert(key.to_string(), serde_json::to_value(value).unwrap());
    }

    pub fn set_deployment(&mut self, deployment: DeploymentConfig) {
        let mut inner = self.inner.lock().unwrap();
        inner.deployment = deployment;
    }

    pub fn set_host(&mut self, host: HostConfig) {
        let mut inner = self.inner.lock().unwrap();
        inner.host = host;
    }

    pub fn set_service(&mut self, service: ServiceConfig) {
        let mut inner = self.inner.lock().unwrap();
        inner.service = service;
    }

    pub fn set_path_config(&mut self, path: PathConfig) {
        let mut inner = self.inner.lock().unwrap();
        inner.path = path;
    }

    pub fn add_variable<T: serde::Serialize>(&mut self, key: &str, value: T) -> TraefikResult<()> {
        if FORBIDDEN_KEYS.contains(&key) {
            return Err(TraefikError::Template(format!("Forbidden key: {}", key)));
        }
        let value = serde_json::to_value(value)
            .map_err(|e| TraefikError::Template(format!("Failed to serialize value: {}", e)))?;

        let mut inner = self.inner.lock().unwrap();
        inner.variables.insert(key.to_string(), value);

        Ok(())
    }

    pub fn add_env_var(&mut self, key: &str, value: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.env.insert(key.to_string(), value.to_string());
    }
}

impl TemplateContext {
    pub fn get_tera_context(&self) -> Arc<Context> {
        let inner = self.inner.lock().unwrap();
        let mut context = Context::new();
        for (key, value) in &inner.variables {
            context.insert(key, value);
        }
        for (key, value) in &inner.env {
            context.insert(key, value);
        }
        let traefik_config = inner.config.clone();
        let deployment = inner.deployment.clone();
        let mut deployment_json = serde_json::to_value(deployment.clone()).unwrap();
        match &deployment.target {
            DeploymentTarget::IpAndPort { ip, port } => {
                deployment_json["ip"] = serde_json::to_value(ip).unwrap();
                deployment_json["port"] = serde_json::to_value(port).unwrap();
            }
            DeploymentTarget::Service { service_name } => {
                let service_name = service_name.clone();
                match traefik_config.get_service(&service_name) {
                    Some(service) => match &service.deployment.target {
                        DeploymentTarget::IpAndPort { ip, port } => {
                            deployment_json["ip"] = serde_json::to_value(ip).unwrap();
                            deployment_json["port"] = serde_json::to_value(port).unwrap();
                        }
                        DeploymentTarget::Service { service_name } => {
                            deployment_json["service_name"] =
                                serde_json::to_value(service_name).unwrap();
                        }
                    },
                    None => {}
                }
            }
        }
        context.insert("deployment", &deployment_json);
        context.insert("host", &inner.host);
        context.insert("path", &inner.path);
        context.insert("config", &inner.config);
        context.insert("service", &inner.service);
        context.insert("traefik", &inner.config);
        Arc::new(context)
    }
}

#[derive(Clone)]
pub struct TeraResolver {
    tera: Arc<Tera>,
}

impl TeraResolver {
    pub fn new() -> TraefikResult<Self> {
        let mut tera = Tera::default();
        tera.autoescape_on(vec!["html"]);
        Ok(Self {
            tera: Arc::new(tera),
        })
    }
}

impl TemplateResolver for TeraResolver {
    fn resolve_template(
        &mut self,
        template: &str,
        template_context: &TemplateContext,
    ) -> TraefikResult<String> {
        let template_name = format!("inline_{}", uuid::Uuid::new_v4());

        let tera = Arc::make_mut(&mut self.tera);
        tera.add_raw_template(&template_name, template)
            .map_err(|e| TraefikError::Template(format!("Failed to add template: {}", e)))?;

        let tera_context = template_context.get_tera_context();

        let result = self
            .tera
            .render(&template_name, &tera_context)
            .map_err(|e| TraefikError::Template(format!("Failed to render template: {}", e)))?;

        Ok(result)
    }
}

pub fn is_template(s: &str) -> bool {
    s.contains("{{") && s.contains("}}")
}

pub fn deserialize_template_vec<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<TemplateOr<String>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings: Option<Vec<String>> = Option::deserialize(deserializer)?;
    Ok(strings.map(|vec| {
        vec.into_iter()
            .map(|s| {
                if is_template(&s) {
                    TemplateOr::Template(s)
                } else {
                    TemplateOr::Static(s)
                }
            })
            .collect()
    }))
}

pub fn deserialize_template_or_string<'de, D>(
    deserializer: D,
) -> Result<Option<TemplateOr<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.map(|s| {
        if is_template(&s) {
            TemplateOr::Template(s)
        } else {
            TemplateOr::Static(s)
        }
    }))
}

// Custom deserializer for TemplateOr<bool>
pub fn deserialize_template_or_bool<'de, D>(
    deserializer: D,
) -> Result<Option<TemplateOr<bool>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        String(String),
    }

    let value: Option<BoolOrString> = Option::deserialize(deserializer)?;
    Ok(value.map(|v| match v {
        BoolOrString::Bool(b) => TemplateOr::Static(b),
        BoolOrString::String(s) => {
            if is_template(&s) {
                TemplateOr::Template(s)
            } else {
                TemplateOr::Static(s.parse().unwrap_or(false))
            }
        }
    }))
}

// Custom deserializer for TemplateOr<bool>
pub fn deserialize_template_or_u16<'de, D>(
    deserializer: D,
) -> Result<Option<TemplateOr<u16>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U16OrString {
        U16(u16),
        String(String),
    }

    let value: Option<U16OrString> = Option::deserialize(deserializer)?;
    Ok(value.map(|v| match v {
        U16OrString::U16(u) => TemplateOr::Static(u),
        U16OrString::String(s) => {
            if is_template(&s) {
                TemplateOr::Template(s)
            } else {
                TemplateOr::Static(s.parse().unwrap_or(0))
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_template() -> TraefikResult<()> {
        let mut resolver = TeraResolver::new()?;
        let mut context = TemplateContext::new(TraefikConfig::default(), Vec::<String>::new())?;
        context.add_env_var("SERVICE_HOST", "example.com");
        context.add_env_var("SERVICE_PORT", "8080");

        let result =
            resolver.resolve_template("http://{{ SERVICE_HOST }}:{{ SERVICE_PORT }}", &context)?;

        assert_eq!(result, "http://example.com:8080");
        Ok(())
    }

    #[test]
    fn test_complex_template() -> TraefikResult<()> {
        let mut resolver = TeraResolver::new()?;
        let mut context = TemplateContext::new(TraefikConfig::default(), Vec::<String>::new())?;

        context.add_variable("max_retries", 3)?;
        context.add_variable("enabled", true)?;

        let template = r#"
        {%- if enabled -%}
            retries={{ max_retries }}
        {%- else -%}
            retries=0
        {%- endif -%}
        "#;

        let result = resolver.resolve_template(template, &context)?;
        assert_eq!(result, "retries=3");

        Ok(())
    }

    #[test]
    fn test_with_u16() -> TraefikResult<()> {
        let mut resolver = TeraResolver::new()?;
        let mut context = TemplateContext::new(TraefikConfig::default(), Vec::<String>::new())?;
        context.add_variable("port", 8080)?;
        let result = resolver.resolve_template("{{ port }}", &context)?;
        assert_eq!(result, "8080");
        Ok(())
    }

    #[test]
    fn test_with_bool() -> TraefikResult<()> {
        let mut resolver = TeraResolver::new()?;
        let mut context = TemplateContext::new(TraefikConfig::default(), Vec::<String>::new())?;
        context.add_variable("enabled", true)?;
        let result = resolver.resolve_template("{{ enabled }}", &context)?;
        assert_eq!(result, "true");
        Ok(())
    }

    #[test]
    fn test_with_string() -> TraefikResult<()> {
        let mut resolver = TeraResolver::new()?;
        let mut context = TemplateContext::new(TraefikConfig::default(), Vec::<String>::new())?;
        context.add_variable("enabled", "true")?;
        let result = resolver.resolve_template("{{ enabled }}", &context)?;
        assert_eq!(result, "true");
        Ok(())
    }

    #[test]
    fn test_with_env_vars() {
        std::env::set_var("TEST_HOST", "testhost.com");

        let mut resolver = TeraResolver::new().unwrap();
        let context = TemplateContext::new(TraefikConfig::default(), vec!["TEST_HOST"]).unwrap();

        let result = resolver
            .resolve_template("{{ TEST_HOST }}", &context)
            .unwrap();
        assert_eq!(result, "testhost.com");
    }

    #[test]
    fn test_set_deployment_to_context_and_get_deployment_from_context() {
        let mut context =
            TemplateContext::new(TraefikConfig::default(), Vec::<String>::new()).unwrap();
        context.set_deployment(DeploymentConfig::default());
        let tera_context = context.get_tera_context();
        let deployment = tera_context.get("deployment");
        assert!(deployment.is_some());
    }

    #[test]
    fn test_get_host() {
        let mut context =
            TemplateContext::new(TraefikConfig::default(), Vec::<String>::new()).unwrap();
        context.set_host(HostConfig::default());
        let tera_context = context.get_tera_context();
        let host = tera_context.get("host");
        assert!(host.is_some());
    }
}
