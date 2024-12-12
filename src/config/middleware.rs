use crate::core::templating::{
    deserialize_template_or_bool, deserialize_template_or_string, deserialize_template_vec,
};
use crate::{
    core::{
        etcd_trait::{EtcdPair, ToEtcdPairs},
        templating::{TemplateContext, TemplateOr, TemplateResolver},
        util::{format_list_value, get_safe_key, validate_is_alphanumeric},
        Validate,
    },
    error::{TraefikError, TraefikResult},
};
use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::headers::{HeadersConfig, RuntimeHeadersConfig};

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct ForwardAuthConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_template_or_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub address: Option<TemplateOr<String>>,
    #[serde(
        default,
        deserialize_with = "deserialize_template_or_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub trust_forward_header: Option<TemplateOr<bool>>,
    #[serde(
        default,
        deserialize_with = "deserialize_template_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth_response_headers: Option<Vec<TemplateOr<String>>>,
    #[serde(
        default,
        deserialize_with = "deserialize_template_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth_request_headers: Option<Vec<TemplateOr<String>>>,
    #[serde(
        default,
        deserialize_with = "deserialize_template_or_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth_response_headers_regex: Option<TemplateOr<String>>,
}

impl ToEtcdPairs for ForwardAuthConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base = "forwardAuth";

        if let Some(address) = &self.address {
            pairs.push(EtcdPair::new(
                format!("{}/address", base),
                address.resolve(resolver, &context)?,
            ));
        }
        if let Some(trust_forward_header) = &self.trust_forward_header {
            pairs.push(EtcdPair::new(
                format!("{}/trustForwardHeader", base),
                trust_forward_header.resolve(resolver, &context)?,
            ));
        }

        if let Some(auth_request_headers) = &self.auth_request_headers {
            let resolved_headers = auth_request_headers
                .iter()
                .map(|header| header.resolve(resolver, &context))
                .collect::<Result<Vec<String>, _>>()?;
            pairs.push(EtcdPair::new(
                format!("{}/authRequestHeaders", base),
                format_list_value(&resolved_headers),
            ));
        }
        if let Some(auth_response_headers) = &self.auth_response_headers {
            let resolved_headers = auth_response_headers
                .iter()
                .map(|header| header.resolve(resolver, &context))
                .collect::<Result<Vec<String>, _>>()?;
            pairs.push(EtcdPair::new(
                format!("{}/authResponseHeaders", base),
                format_list_value(&resolved_headers),
            ));
        }
        if let Some(auth_response_headers_regex) = &self.auth_response_headers_regex {
            pairs.push(EtcdPair::new(
                format!("{}/authResponseHeadersRegex", base),
                auth_response_headers_regex.resolve(resolver, &context)?,
            ));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub enum MiddlewareType {
    Headers,
    RedirectRegex,
    StripPrefix,
    ForwardAuth,
    RedirectScheme,
    RateLimit,
    BasicAuth,
    Compress,
    CircuitBreaker,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct MiddlewareEntry {
    pub middleware_type: MiddlewareType,
    pub pairs: Vec<EtcdPair>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct MiddlewareConfig {
    #[serde(default)]
    pub name: String,
    /// The protocol to use for the middleware
    #[serde(default = "default_protocol")]
    pub protocol: String,
    /// The headers configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HeadersConfig>,
    /// The forward auth configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_auth: Option<ForwardAuthConfig>,
    /// The redirect regex configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_regex: Option<RedirectRegexConfig>,
    /// The redirect scheme configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_scheme: Option<RedirectSchemeConfig>,
    /// The strip prefix configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_prefix: Option<StripPrefixConfig>,
    /// The rate limit configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
    /// The basic auth configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<BasicAuthConfig>,
    /// The compress configuration
    #[serde(default)]
    pub compress: bool,
    /// The circuit breaker configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_breaker: Option<CircuitBreakerConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_headers: Option<RuntimeHeadersConfig>,
}

// Add configuration structs for each middleware type
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RedirectRegexConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permanent: Option<TemplateOr<bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<TemplateOr<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<TemplateOr<String>>,
}

impl ToEtcdPairs for RedirectRegexConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let base_key = "redirectRegex";
        let mut pairs = vec![];
        if let Some(permanent) = &self.permanent {
            pairs.push(EtcdPair::new(
                format!("{}/permanent", base_key),
                permanent.resolve(resolver, &context)?.to_string(),
            ));
        }
        if let Some(regex) = &self.regex {
            pairs.push(EtcdPair::new(
                format!("{}/regex", base_key),
                regex.resolve(resolver, &context)?,
            ));
        }
        if let Some(replacement) = &self.replacement {
            pairs.push(EtcdPair::new(
                format!("{}/replacement", base_key),
                replacement.resolve(resolver, &context)?,
            ));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RedirectSchemeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<TemplateOr<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permanent: Option<TemplateOr<bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<TemplateOr<String>>,
}

impl ToEtcdPairs for RedirectSchemeConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base_key = "redirectScheme";
        if let Some(scheme) = &self.scheme {
            pairs.push(EtcdPair::new(
                format!("{}/scheme", base_key),
                scheme.resolve(resolver, &context)?,
            ));
        }
        if let Some(permanent) = &self.permanent {
            pairs.push(EtcdPair::new(
                format!("{}/permanent", base_key),
                permanent.resolve(resolver, &context)?.to_string(),
            ));
        }
        if let Some(port) = &self.port {
            pairs.push(EtcdPair::new(
                format!("{}/port", base_key),
                port.resolve(resolver, &context)?,
            ));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct StripPrefixConfig {
    #[serde(
        default,
        deserialize_with = "deserialize_template_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub prefixes: Option<Vec<TemplateOr<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_slash: Option<TemplateOr<bool>>,
}

impl ToEtcdPairs for StripPrefixConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base_key = "stripPrefix";

        if let Some(prefixes) = &self.prefixes {
            for (idx, prefix) in prefixes.iter().enumerate() {
                pairs.push(EtcdPair::new(
                    format!("{}/prefixes/{}", base_key, idx),
                    prefix.resolve(resolver, context)?,
                ));
            }
        }

        if let Some(force_slash) = &self.force_slash {
            pairs.push(EtcdPair::new(
                format!("{}/forceSlash", base_key),
                force_slash.resolve(resolver, context)?.to_string(),
            ));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RateLimitConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average: Option<TemplateOr<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burst: Option<TemplateOr<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<TemplateOr<String>>,
}

impl ToEtcdPairs for RateLimitConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let base_key = "rateLimit";
        let mut pairs = vec![];
        if let Some(average) = &self.average {
            pairs.push(EtcdPair::new(
                format!("{}/average", base_key),
                average.resolve(resolver, &context)?.to_string(),
            ));
        }
        if let Some(burst) = &self.burst {
            pairs.push(EtcdPair::new(
                format!("{}/burst", base_key),
                burst.resolve(resolver, &context)?.to_string(),
            ));
        }
        if let Some(period) = &self.period {
            pairs.push(EtcdPair::new(
                format!("{}/period", base_key),
                period.resolve(resolver, &context)?,
            ));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct BasicAuthConfig {
    pub users: Vec<TemplateOr<String>>,
    pub realm: Option<TemplateOr<String>>,
    pub header_field: Option<TemplateOr<String>>,
}

impl ToEtcdPairs for BasicAuthConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base_key = "basicAuth";
        for (idx, user) in self.users.iter().enumerate() {
            pairs.push(EtcdPair::new(
                format!("{}/users/{}", base_key, idx),
                user.resolve(resolver, &context)?,
            ));
        }
        if let Some(realm) = &self.realm {
            pairs.push(EtcdPair::new(
                format!("{}/realm", base_key),
                realm.resolve(resolver, &context)?,
            ));
        }
        if let Some(header_field) = &self.header_field {
            pairs.push(EtcdPair::new(
                format!("{}/headerField", base_key),
                header_field.resolve(resolver, &context)?,
            ));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct CircuitBreakerConfig {
    pub expression: TemplateOr<String>,
}

impl ToEtcdPairs for CircuitBreakerConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let base_key = "circuitBreaker";
        Ok(vec![EtcdPair::new(
            format!("{}/expression", base_key),
            self.expression.resolve(resolver, &context)?,
        )])
    }
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        MiddlewareConfig {
            name: "test-middleware".to_string(),
            headers: None,
            protocol: default_protocol(),
            forward_auth: None,
            strip_prefix: None,
            rate_limit: None,
            basic_auth: None,
            compress: false,
            circuit_breaker: None,
            redirect_regex: None,
            redirect_scheme: None,
            runtime_headers: None,
        }
    }
}

fn default_protocol() -> String {
    "http".to_string()
}

impl MiddlewareConfig {
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_protocol(&mut self, protocol: &str) {
        self.protocol = protocol.to_string();
    }
}

/// Convert the middleware configuration to etcd pairs
///
/// The middleware configuration is stored in etcd under the following path:
/// `{base_key}/{protocol}/middlewares`

impl ToEtcdPairs for MiddlewareConfig {
    fn to_etcd_pairs(
        &self,
        root_base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        // let base_key = format!("{}/{}", base_key, self.name);
        let base_key = self.name.clone();
        // let base_key = "".to_string();

        // All middleware configurations should be under the protocol path
        if let Some(headers) = &self.headers {
            let header_pairs = headers.to_etcd_pairs("headers", resolver, context)?;
            for pair in header_pairs {
                // Remove the first 'headers/' from the path
                let new_key = pair.key().replace("headers/headers/", "headers/");
                pairs.push(EtcdPair::new(new_key, pair.value()));
            }

            pairs.extend(headers.to_etcd_pairs("headers", resolver, context)?);
        }

        if let Some(runtime_headers) = &self.runtime_headers {
            pairs.extend(runtime_headers.to_etcd_pairs(&base_key, resolver, context)?);
        }

        if let Some(forward_auth) = &self.forward_auth {
            pairs.extend(forward_auth.to_etcd_pairs(&base_key, resolver, context)?);
        }

        if let Some(redirect_regex) = &self.redirect_regex {
            pairs.extend(redirect_regex.to_etcd_pairs(&base_key, resolver, context)?);
        }

        if let Some(redirect_scheme) = &self.redirect_scheme {
            pairs.extend(redirect_scheme.to_etcd_pairs(&base_key, resolver, context)?);
        }

        if let Some(strip_prefix) = &self.strip_prefix {
            pairs.extend(strip_prefix.to_etcd_pairs(&base_key, resolver, context)?);
        }

        if let Some(rate_limit) = &self.rate_limit {
            pairs.extend(rate_limit.to_etcd_pairs(&base_key, resolver, context)?);
        }

        if let Some(basic_auth) = &self.basic_auth {
            pairs.extend(basic_auth.to_etcd_pairs(&base_key, resolver, context)?);
        }

        if self.compress {
            pairs.push(EtcdPair::new("compress", "true".to_string()));
        }

        if let Some(circuit_breaker) = &self.circuit_breaker {
            pairs.extend(circuit_breaker.to_etcd_pairs(&base_key, resolver, context)?);
        }

        let prefixed_pairs = pairs
            .iter()
            .map(|pair| {
                // let prefixed_key = format!("{}/{}/{}", root_base_key, base_key, pair.key());
                let prefixed_key = format!("{}/{}/{}", root_base_key, self.name, pair.key());

                EtcdPair::new(prefixed_key, pair.value())
            })
            .collect();

        Ok(prefixed_pairs)
    }
}

impl Validate for MiddlewareConfig {
    /// Validate the middleware configuration
    fn validate(
        &self,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<()> {
        if self.name.is_empty() {
            return Err(TraefikError::MiddlewareConfig(
                "middleware name is empty".into(),
            ));
        }

        validate_is_alphanumeric(&self.name)?;

        if let Some(headers) = &self.headers {
            headers.validate(resolver, context)?;
        }
        Ok(())
    }
}

impl MiddlewareConfig {
    pub fn get_safe_key(&self) -> String {
        get_safe_key(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{
        assert_contains_pair, create_base_middleware_config, create_test_middleware,
        create_test_resolver, create_test_template_context,
    };

    #[test]
    fn test_headers_config_validate() {
        let middleware = create_test_middleware();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        middleware
            .get("enable-headers")
            .unwrap()
            .validate(&mut resolver, &context)
            .unwrap();
        assert!(!middleware.contains_key("invalid-middleware"));
    }

    #[test]
    fn test_middleware_is_invalid_if_name_is_empty() {
        let middleware = MiddlewareConfig {
            name: "".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(middleware.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_middleware_is_invalid_if_name_is_not_alphanumeric_or_hyphens() {
        let middleware = MiddlewareConfig {
            name: "invalid-%middleware".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(middleware.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_middleware_is_valid_if_name_is_alphanumeric_or_hyphens() {
        let middleware = MiddlewareConfig {
            name: "valid-middleware".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(middleware.validate(&mut resolver, &context).is_ok());
    }

    #[test]
    fn test_to_etcd_pairs_global() {
        let middleware = create_test_middleware();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let mut result_pairs = vec![];
        for (_name, middleware) in middleware {
            let pairs = middleware
                .to_etcd_pairs("test/middlewares", &mut resolver, &context)
                .unwrap();
            result_pairs.extend(pairs);
        }
        let pair_strs: Vec<String> = result_pairs.iter().map(|p| p.to_string()).collect();

        // assert!(pair_strs.contains(&"test/http/middlewares/enable-headers true".to_string()));
        // assert!(pair_strs.contains(&"test/http/middlewares/handle-redirects true".to_string()));
        assert!(pair_strs.contains(
            &"test/middlewares/enable-headers/headers/customRequestHeaders/X-Forwarded-Proto https"
                .to_string()
        ));
        assert!(pair_strs.contains(
            &"test/middlewares/enable-headers/headers/customRequestHeaders/X-Forwarded-Port 443"
                .to_string()
        ));

        assert!(pair_strs.contains(
            &"test/middlewares/enable-headers/headers/customResponseHeaders/Location ".to_string()
        ));
        assert!(pair_strs.contains(
            &"test/middlewares/enable-headers/headers/accessControlAllowMethods GET, POST, OPTIONS"
                .to_string()
        ));
        assert!(pair_strs.contains(
            &"test/middlewares/enable-headers/headers/accessControlAllowHeaders Content-Type, Authorization"
                .to_string()
        ));
    }

    #[test]
    fn test_to_etcd_pairs() {
        let middleware = create_test_middleware();
        let mut result_pairs = vec![];
        for (_name, middleware) in middleware {
            let base_key = "test/http/middlewares";
            let mut resolver = create_test_resolver();
            let context = create_test_template_context();
            let pairs = middleware
                .to_etcd_pairs(&base_key, &mut resolver, &context)
                .unwrap();
            result_pairs.extend(pairs);
        }
        let pair_strs: Vec<String> = result_pairs.iter().map(|p| p.to_string()).collect();

        // assert!(pair_strs.contains(&"test/http/middlewares/enable-headers true".to_string()));
        // assert!(pair_strs.contains(&"test/http/middlewares/handle-redirects true".to_string()));
        assert!(pair_strs.contains(
            &"test/http/middlewares/enable-headers/headers/customRequestHeaders/X-Forwarded-Proto https"
                .to_string()
        ));
        assert!(pair_strs.contains(
            &"test/http/middlewares/enable-headers/headers/customRequestHeaders/X-Forwarded-Port 443"
                .to_string()
        ));

        assert!(pair_strs.contains(
            &"test/http/middlewares/enable-headers/headers/customResponseHeaders/Location "
                .to_string()
        ));
        assert!(pair_strs.contains(
            &"test/http/middlewares/enable-headers/headers/accessControlAllowMethods GET, POST, OPTIONS"
                .to_string()
        ));
        assert!(pair_strs.contains(
            &"test/http/middlewares/enable-headers/headers/accessControlAllowHeaders Content-Type, Authorization"
                .to_string()
        ));
    }

    #[test]
    fn test_forward_auth_config_to_etcd_pairs() {
        let forward_auth = ForwardAuthConfig {
            address: Some(TemplateOr::Static("http://localhost:8080".to_string())),
            trust_forward_header: Some(TemplateOr::Static(true)),
            auth_response_headers: Some(vec![
                TemplateOr::Static("X-Forwarded-Proto".to_string()),
                TemplateOr::Static("ServiceAddr".to_string()),
                TemplateOr::Static("ServiceUrl".to_string()),
            ]),
            auth_response_headers_regex: Some(TemplateOr::Static(".*".to_string())),
            auth_request_headers: None,
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = forward_auth
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"forwardAuth/address http://localhost:8080".to_string()));
        assert!(pair_strs.contains(&"forwardAuth/trustForwardHeader true".to_string()));
    }

    #[test]
    fn test_middleware_config_to_etcd_pairs_with_forward_auth() {
        let forward_auth = ForwardAuthConfig {
            address: None,
            trust_forward_header: Some(TemplateOr::Static(true)),
            auth_response_headers: Some(vec![
                TemplateOr::Static("X-Forwarded-Proto".to_string()),
                TemplateOr::Static("ServiceAddr".to_string()),
                TemplateOr::Static("ServiceUrl".to_string()),
            ]),
            auth_response_headers_regex: Some(TemplateOr::Static(".*".to_string())),
            auth_request_headers: None,
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = forward_auth
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"forwardAuth/trustForwardHeader true".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_auth_response_headers() {
        let forward_auth = ForwardAuthConfig {
            address: None,
            trust_forward_header: Some(TemplateOr::Static(true)),
            auth_response_headers: Some(vec![
                TemplateOr::Static("X-Forwarded-Proto".to_string()),
                TemplateOr::Static("ServiceAddr".to_string()),
                TemplateOr::Static("ServiceUrl".to_string()),
            ]),
            auth_response_headers_regex: None,
            auth_request_headers: None,
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = forward_auth
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(
            &"forwardAuth/authResponseHeaders X-Forwarded-Proto, ServiceAddr, ServiceUrl"
                .to_string()
        ));
    }

    #[test]
    fn test_to_etcd_pairs_with_auth_response_headers_regex() {
        let forward_auth = ForwardAuthConfig {
            address: None,
            trust_forward_header: Some(TemplateOr::Static(true)),
            auth_response_headers: None,
            auth_response_headers_regex: Some(TemplateOr::Static("^X-.*".to_string())),
            auth_request_headers: None,
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = forward_auth
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"forwardAuth/authResponseHeadersRegex ^X-.*".to_string()));
    }

    #[test]
    fn test_middleware_headers() {
        let mut headers = HeadersConfig::default();
        headers.custom_request_headers.insert(
            "X-Forwarded-Proto".to_string(),
            TemplateOr::Static("https".to_string()),
        );

        let middleware = MiddlewareConfig {
            headers: Some(headers),
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(
            &pairs,
            "test/test-middleware/headers/customRequestHeaders/X-Forwarded-Proto https",
        );
    }

    #[test]
    fn test_middleware_redirect_regex() {
        let redirect_regex = RedirectRegexConfig {
            permanent: Some(TemplateOr::Static(true)),
            regex: Some(TemplateOr::Static("^/old/(.*)".to_string())),
            replacement: Some(TemplateOr::Static("/new/$1".to_string())),
        };

        let middleware = MiddlewareConfig {
            redirect_regex: Some(redirect_regex),
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/redirectRegex/permanent true");
        assert_contains_pair(
            &pairs,
            "test/test-middleware/redirectRegex/regex ^/old/(.*)",
        );
        assert_contains_pair(
            &pairs,
            "test/test-middleware/redirectRegex/replacement /new/$1",
        );
    }

    #[test]
    fn test_middleware_redirect_scheme() {
        let redirect_scheme = RedirectSchemeConfig {
            scheme: Some(TemplateOr::Static("https".to_string())),
            permanent: Some(TemplateOr::Static(true)),
            port: Some(TemplateOr::Static("443".to_string())),
        };

        let middleware = MiddlewareConfig {
            redirect_scheme: Some(redirect_scheme),
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/redirectScheme/scheme https");
        assert_contains_pair(&pairs, "test/test-middleware/redirectScheme/permanent true");
        assert_contains_pair(&pairs, "test/test-middleware/redirectScheme/port 443");
    }

    #[test]
    fn test_middleware_strip_prefix() {
        let strip_prefix = StripPrefixConfig {
            prefixes: Some(vec![
                TemplateOr::Static("/api".to_string()),
                TemplateOr::Static("/v1".to_string()),
            ]),
            force_slash: Some(TemplateOr::Static(true)),
        };

        let middleware = MiddlewareConfig {
            strip_prefix: Some(strip_prefix),
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/stripPrefix/prefixes/0 /api");
        assert_contains_pair(&pairs, "test/test-middleware/stripPrefix/prefixes/1 /v1");
        assert_contains_pair(&pairs, "test/test-middleware/stripPrefix/forceSlash true");
    }

    #[test]
    fn test_middleware_rate_limit() {
        let rate_limit = RateLimitConfig {
            average: Some(TemplateOr::Static(100)),
            burst: Some(TemplateOr::Static(200)),
            period: Some(TemplateOr::Static("1s".to_string())),
        };

        let middleware = MiddlewareConfig {
            rate_limit: Some(rate_limit),
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/rateLimit/average 100");
        assert_contains_pair(&pairs, "test/test-middleware/rateLimit/burst 200");
        assert_contains_pair(&pairs, "test/test-middleware/rateLimit/period 1s");
    }

    #[test]
    fn test_middleware_basic_auth() {
        let basic_auth = BasicAuthConfig {
            users: vec![TemplateOr::Static("user:password".to_string())],
            realm: Some(TemplateOr::Static("My Realm".to_string())),
            header_field: Some(TemplateOr::Static("X-Auth".to_string())),
        };

        let middleware = MiddlewareConfig {
            basic_auth: Some(basic_auth),
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(
            &pairs,
            "test/test-middleware/basicAuth/users/0 user:password",
        );
        assert_contains_pair(&pairs, "test/test-middleware/basicAuth/realm My Realm");
        assert_contains_pair(&pairs, "test/test-middleware/basicAuth/headerField X-Auth");
    }

    #[test]
    fn test_middleware_compress() {
        let middleware = MiddlewareConfig {
            compress: true,
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/compress true");
    }

    #[test]
    fn test_middleware_circuit_breaker() {
        let circuit_breaker = CircuitBreakerConfig {
            expression: TemplateOr::Static("NetworkErrorRatio() > 0.5".to_string()),
        };

        let middleware = MiddlewareConfig {
            circuit_breaker: Some(circuit_breaker),
            ..create_base_middleware_config()
        };

        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = middleware
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        assert_contains_pair(
            &pairs,
            "test/test-middleware/circuitBreaker/expression NetworkErrorRatio() > 0.5",
        );
    }

    #[cfg(test)]
    mod tests {
        use std::sync::Arc;

        use serde_json::json;

        use super::*;
        use crate::config::deployment::{DeploymentConfig, DeploymentTarget};
        use crate::core::templating::{TemplateContext, TeraResolver};

        #[test]
        fn test_forward_auth_config_templates() -> TraefikResult<()> {
            let yaml = r#"
            address: "http://redirector:3000"
            trust_forward_header: true
            auth_response_headers:
                - "X-Real-Ip"
                - "ServiceAddr"
                - "{{ deployment.service.ip }}:{{ deployment.service.port }}"
                - "X-Forwarded-*"
            auth_request_headers:
                - "ServiceURL"
                - "{{ deployment.service.ip }}:{{ deployment.service.port }}"
                - "RequestAddr"
                - "ServiceURL: http://{{ deployment.service.ip }}:{{ deployment.service.port }}"
            auth_response_headers_regex: "^X-"
            "#;

            let config: ForwardAuthConfig = serde_yaml::from_str(yaml)?;

            // Create a deployment for the context
            let deployment = DeploymentConfig::builder()
                .name("test".to_string())
                .ip_and_port("10.0.0.1".to_string(), 8080)
                .build();

            // Create service context from deployment
            let service_json = match &deployment.target {
                DeploymentTarget::IpAndPort { ip, port } => {
                    json!({
                        "ip": ip,
                        "port": port,
                    })
                }
                _ => unreachable!(),
            };

            // Set up resolver and context
            let mut resolver = TeraResolver::new()?;
            let mut context = TemplateContext::new(None, Vec::<String>::new())?;

            // Add both deployment and service to context
            if let Some(ctx) = Arc::get_mut(&mut context.context) {
                ctx.insert(
                    "deployment",
                    &json!({
                        "service": service_json
                    }),
                );
            }

            // Test template resolution
            let pairs = config.to_etcd_pairs("test", &mut resolver, &context)?;

            // Verify the templated values
            for pair in pairs {
                match pair.key() {
                    "forwardAuth/authResponseHeaders" => {
                        assert!(pair.value().contains("10.0.0.1:8080"));
                        assert!(pair.value().contains("X-Real-Ip"));
                    }
                    "forwardAuth/authRequestHeaders" => {
                        assert!(pair.value().contains("10.0.0.1:8080"));
                        assert!(pair.value().contains("ServiceURL"));
                    }
                    _ => {}
                }
            }

            Ok(())
        }

        #[test]
        fn test_static_and_template_mixing() -> TraefikResult<()> {
            let yaml = r#"
        address: "http://{{ env.REDIRECTOR_HOST }}:{{ env.REDIRECTOR_PORT }}"
        trust_forward_header: true
        auth_response_headers:
          - "X-Real-Ip"
          - "{{ deployment.service.ip }}"
        auth_request_headers:
          - "Static-Header"
          - "{{ deployment.service.port }}"
        "#;

            let config: ForwardAuthConfig = serde_yaml::from_str(yaml)?;

            // Verify we can mix static and template values
            assert!(matches!(config.address, Some(TemplateOr::Template(_))));
            assert!(matches!(
                config.trust_forward_header,
                Some(TemplateOr::Static(true))
            ));

            if let Some(response_headers) = config.auth_response_headers {
                assert!(matches!(&response_headers[0], TemplateOr::Static(_)));
                assert!(matches!(&response_headers[1], TemplateOr::Template(_)));
            } else {
                panic!("Expected auth_response_headers to be Some");
            }

            Ok(())
        }
    }
}
