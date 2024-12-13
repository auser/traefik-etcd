use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        etcd_trait::{EtcdPair, ToEtcdPairs},
        templating::{TemplateContext, TemplateResolver},
        util::validate_is_alphanumeric,
        Validate,
    },
    error::{TraefikError, TraefikResult},
};

use super::deployment::{DeploymentConfig, DeploymentTarget};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct ServiceConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default, flatten)]
    pub deployment: DeploymentConfig,
    #[serde(default)]
    pub pass_host_header: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "redirector".to_string(),
            deployment: DeploymentConfig::default(),
            pass_host_header: true,
        }
    }
}

impl ServiceConfig {
    pub fn get_service_name(&self) -> String {
        format!("service-{}", self.name)
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }
}

impl ServiceConfig {
    pub fn builder() -> ServiceConfigBuilder {
        ServiceConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ServiceConfigBuilder {
    name: String,
    deployment: DeploymentConfig,
    pass_host_header: bool,
}

impl ServiceConfigBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn deployment(mut self, deployment: DeploymentConfig) -> Self {
        self.deployment = deployment;
        self
    }

    pub fn ip_and_port(mut self, ip: String, port: u16) -> Self {
        self.deployment = DeploymentConfig::builder().ip_and_port(ip, port).build();
        self
    }

    pub fn pass_host_header(mut self, pass_host_header: bool) -> Self {
        self.pass_host_header = pass_host_header;
        self
    }

    pub fn build(self) -> ServiceConfig {
        ServiceConfig {
            name: self.name,
            deployment: self.deployment,
            pass_host_header: self.pass_host_header,
        }
    }
}

/*
 $ecd put "traefik/http/services/redirector/loadBalancer/servers/0/url" "http://redirector:3000"
 $ecd put "traefik/http/services/redirector/loadBalancer/responseForwarding/flushInterval" "100ms"

 $ecd put "traefik/http/services/redirector/loadBalancer/passHostHeader" "true"
 $ecd put "traefik/http/services/redirector/loadBalancer/healthCheck/path" "/health"
 $ecd put "traefik/http/services/redirector/loadBalancer/healthCheck/interval" "10s"
 $ecd put "traefik/http/services/redirector/loadBalancer/healthCheck/timeout" "5s"
*/
impl ToEtcdPairs for ServiceConfig {
    fn to_etcd_pairs(
        &self,
        base_key: &str,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        // The base_key: `{prefix}/{protocol}`
        let mut pairs = Vec::new();

        let service_base_key = format!("{}/services/{}", base_key, self.name);
        // Create the url
        let (ip, port) = match &self.deployment.target {
            DeploymentTarget::IpAndPort { ip, port } => (ip, port),
            _ => {
                return Err(TraefikError::ServiceConfig(format!(
                    "Service {} requires an ip and port: {}",
                    self.name, self.deployment.target
                )))
            }
        };
        let url = format!("{}://{}:{}", self.deployment.protocol, ip, port);
        // TODO: handle multiple hosts?
        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/servers/0/url", service_base_key),
            url,
        ));
        pairs.push(EtcdPair::new(
            format!(
                "{}/loadBalancer/responseForwarding/flushInterval",
                service_base_key
            ),
            "100ms".to_string(),
        ));
        if self.pass_host_header {
            pairs.push(EtcdPair::new(
                format!("{}/loadBalancer/passHostHeader", service_base_key),
                self.pass_host_header.to_string(),
            ));
        }

        Ok(pairs)
    }
}

impl Validate for ServiceConfig {
    fn validate(
        &self,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<()> {
        if self.name.is_empty() {
            return Err(TraefikError::ServiceConfig("service name is empty".into()));
        }

        validate_is_alphanumeric(&self.name)?;
        self.deployment.validate(resolver, context)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::templating::TeraResolver,
        test_helpers::{assert_contains_pair, create_test_resolver, create_test_template_context},
    };

    use super::*;

    #[test]
    fn test_validate_service_config_empty_name() {
        let service = ServiceConfig {
            name: "".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = service.validate(&mut resolver, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_service_config_invalid_name() {
        let service = ServiceConfig {
            name: "invalid-name!".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = service.validate(&mut resolver, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_service_config_valid() {
        let service = ServiceConfig::default();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let result = service.validate(&mut resolver, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_name() {
        let mut service = ServiceConfig::default();
        service.set_name("test");
        assert_eq!(service.name, "test");
    }

    #[test]
    fn test_default_service_config_to_etcd_pairs() {
        let service = ServiceConfig::default();
        let mut resolver = TeraResolver::new().unwrap();
        let context = TemplateContext::default();
        let result = service.to_etcd_pairs("traefik/http", &mut resolver, &context);
        assert!(result.is_ok());
        let pairs = result.unwrap();
        assert_eq!(pairs.len(), 3);

        assert_contains_pair(
            &pairs,
            "traefik/http/services/redirector/loadBalancer/servers/0/url http://127.0.0.1:80",
        );
        assert_contains_pair(
            &pairs,
            "traefik/http/services/redirector/loadBalancer/responseForwarding/flushInterval 100ms",
        );
        assert_contains_pair(
            &pairs,
            "traefik/http/services/redirector/loadBalancer/passHostHeader true",
        );
    }
}
