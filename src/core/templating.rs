use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tera::{Context, Tera};

use crate::{
    config::{
        deployment::{DeploymentConfig, DeploymentTarget},
        host::{HostConfig, PathConfig},
    },
    error::{TraefikError, TraefikResult},
    TraefikConfig,
};

const FORBIDDEN_KEYS: [&str; 3] = ["deployment", "host", "path"];

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ServiceContext {
    ip: String,
    port: u16,
    name: String,
}

impl ServiceContext {
    pub fn new(ip: String, port: u16, name: String) -> Self {
        Self { ip, port, name }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct DeploymentContext {
    service: ServiceContext,
    name: String,
}

impl DeploymentContext {
    pub fn new(service: ServiceContext, name: String) -> Self {
        Self { service, name }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub context: Arc<Context>,
}

impl TemplateContext {
    pub fn new<T: ToString>(
        traefik_config: Option<TraefikConfig>,
        env_vars: Vec<T>,
    ) -> TraefikResult<Self> {
        let mut context = Context::new();

        // Initialize with environment variables
        let mut env_json = json!({});
        for env_var in env_vars {
            let key = env_var.to_string();
            let value = std::env::var(&key).unwrap_or_else(|_| "".to_string());
            env_json[key] = json!(value);
        }
        context.insert("env", &env_json);

        if let Some(config) = traefik_config {
            context.insert("config", &json!(config));
        }

        Ok(Self {
            context: Arc::new(context),
        })
    }

    pub fn with_env_vars<T: ToString>(
        traefik_config: Option<TraefikConfig>,
        env_vars: Vec<T>,
    ) -> TraefikResult<Self> {
        Self::new(traefik_config, env_vars)
    }

    pub fn get_env_vars(&self) -> Vec<(String, String)> {
        let mut env_vars = Vec::new();
        if let Some(env) = self.context.get("env") {
            if let Some(env_obj) = env.as_object() {
                for (key, value) in env_obj {
                    if let Some(value_str) = value.as_str() {
                        env_vars.push((key.clone(), value_str.to_string()));
                    }
                }
            }
        }
        env_vars
    }

    pub fn insert_variable(&mut self, key: &str, value: impl Serialize) {
        if let Some(context) = Arc::get_mut(&mut self.context) {
            context.insert(key, &value);
        }
    }

    pub fn get_variable(&self, key: &str) -> Option<serde_json::Value> {
        self.context.get(key).cloned()
    }

    pub fn get_variables(&self) -> Vec<(String, serde_json::Value)> {
        let mut variables = Vec::new();
        let context_json = self.context.as_ref().clone().into_json();
        if let Some(obj) = context_json.as_object() {
            for (key, value) in obj {
                if !FORBIDDEN_KEYS.contains(&key.as_str()) {
                    variables.push((key.clone(), value.clone()));
                }
            }
        }
        variables
    }

    pub fn set_deployment(&mut self, deployment: DeploymentConfig) {
        if let Some(context) = Arc::get_mut(&mut self.context) {
            context.insert("deployment", &deployment);
        }
    }

    pub fn get_deployment(&self) -> Option<DeploymentConfig> {
        self.context
            .get("deployment")
            .and_then(|value| serde_json::from_value::<DeploymentConfig>(value.clone()).ok())
    }

    pub fn set_host(&mut self, host: HostConfig) {
        if let Some(context) = Arc::get_mut(&mut self.context) {
            context.insert("host", &host);
        }
    }

    pub fn get_host(&self) -> Option<HostConfig> {
        self.context
            .get("host")
            .and_then(|value| serde_json::from_value::<HostConfig>(value.clone()).ok())
    }

    pub fn set_path(&mut self, path: PathConfig) {
        if let Some(context) = Arc::get_mut(&mut self.context) {
            context.insert("path", &path);
        }
    }

    pub fn get_path(&self) -> Option<PathConfig> {
        self.context
            .get("path")
            .and_then(|value| serde_json::from_value::<PathConfig>(value.clone()).ok())
    }

    pub fn add_variable<T: serde::Serialize>(&mut self, key: &str, value: T) -> TraefikResult<()> {
        if FORBIDDEN_KEYS.contains(&key) {
            return Err(TraefikError::Template(format!("Forbidden key: {}", key)));
        }
        let value = serde_json::to_value(value)
            .map_err(|e| TraefikError::Template(format!("Failed to serialize value: {}", e)))?;

        if let Some(context) = Arc::get_mut(&mut self.context) {
            context.insert(key, &value);
        }
        Ok(())
    }

    pub fn add_env_var(&mut self, key: &str, value: &str) {
        if let Some(context) = Arc::get_mut(&mut self.context) {
            let mut env = context
                .get("env")
                .and_then(|v| v.as_object().cloned())
                .unwrap_or_default();
            env.insert(key.to_string(), json!(value));
            context.insert("env", &env);
        }
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

        let result = self
            .tera
            .render(&template_name, &template_context.context)
            .map_err(|e| TraefikError::Template(format!("Failed to render template: {}", e)))?;

        Ok(result)
    }
}

pub fn create_template_context(
    traefik_config: &TraefikConfig,
    deployment: Option<DeploymentConfig>,
    host_config: Option<HostConfig>,
    path_config: Option<PathConfig>,
) -> TraefikResult<TemplateContext> {
    let mut context = Context::new();

    // Initialize with empty environment variables
    context.insert("env", &json!({}));

    if let Some(deployment) = deployment {
        let service_context = match &deployment.target {
            DeploymentTarget::IpAndPort { ip, port } => {
                json!({
                    "service": {
                        "ip": ip,
                        "port": port,
                    }
                })
            }
            DeploymentTarget::Service { service_name } => {
                if let Some(services) = &traefik_config.services {
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

        context.insert("service", &service_context);
        context.insert("deployment", &deployment);
    }

    if let Some(host_config) = host_config {
        context.insert("host", &host_config);
    }
    if let Some(path_config) = path_config {
        context.insert("path", &path_config);
    }

    context.insert("traefik", &traefik_config);

    Ok(TemplateContext {
        context: Arc::new(context),
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_template() -> TraefikResult<()> {
        let mut resolver = TeraResolver::new()?;
        let mut context = TemplateContext::new(None, Vec::<String>::new())?;
        context.add_env_var("SERVICE_HOST", "example.com");
        context.add_env_var("SERVICE_PORT", "8080");

        let result = resolver.resolve_template(
            "http://{{ env.SERVICE_HOST }}:{{ env.SERVICE_PORT }}",
            &context,
        )?;

        assert_eq!(result, "http://example.com:8080");
        Ok(())
    }

    #[test]
    fn test_complex_template() -> TraefikResult<()> {
        let mut resolver = TeraResolver::new()?;
        let mut context = TemplateContext::new(None, Vec::<String>::new())?;

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
    fn test_with_env_vars() {
        std::env::set_var("TEST_HOST", "testhost.com");

        let mut resolver = TeraResolver::new().unwrap();
        let context = TemplateContext::new(None, vec!["TEST_HOST"]).unwrap();

        let result = resolver
            .resolve_template("{{ env.TEST_HOST }}", &context)
            .unwrap();
        assert_eq!(result, "testhost.com");
    }

    #[test]
    fn test_get_deployment() {
        let mut context = TemplateContext::new(None, Vec::<String>::new()).unwrap();
        context.set_deployment(DeploymentConfig::default());
        let deployment = context.get_deployment();
        assert!(deployment.is_some());
    }

    #[test]
    fn test_get_host() {
        let mut context = TemplateContext::new(None, Vec::<String>::new()).unwrap();
        context.set_host(HostConfig::default());
        let host = context.get_host();
        assert!(host.is_some());
    }
}
