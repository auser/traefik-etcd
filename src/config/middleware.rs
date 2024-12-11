use crate::{
    core::{
        etcd_trait::{EtcdPair, ToEtcdPairs},
        util::{format_list_value, get_safe_key, validate_is_alphanumeric},
        Validate,
    },
    error::{TraefikError, TraefikResult},
};
use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::headers::HeadersConfig;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct ForwardAuthConfig {
    /// The address of the forward auth service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Whether to trust the forward header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_forward_header: Option<bool>,
    /// The auth response headers to add to the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_response_headers: Option<Vec<String>>,
    /// The auth response headers regex to add to the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_response_headers_regex: Option<String>,
    /// The auth request headers to add to the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_request_headers: Option<Vec<String>>,
}

impl ToEtcdPairs for ForwardAuthConfig {
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base = "forwardAuth";

        if let Some(address) = &self.address {
            pairs.push(EtcdPair::new(format!("{}/address", base), address.clone()));
        }
        if let Some(trust_forward_header) = &self.trust_forward_header {
            pairs.push(EtcdPair::new(
                format!("{}/trustForwardHeader", base),
                trust_forward_header.to_string(),
            ));
        }

        if let Some(auth_request_headers) = &self.auth_request_headers {
            pairs.push(EtcdPair::new(
                format!("{}/authRequestHeaders", base),
                format_list_value(auth_request_headers),
            ));
        }
        if let Some(auth_response_headers) = &self.auth_response_headers {
            pairs.push(EtcdPair::new(
                format!("{}/authResponseHeaders", base),
                format_list_value(auth_response_headers),
            ));
        }
        if let Some(auth_response_headers_regex) = &self.auth_response_headers_regex {
            pairs.push(EtcdPair::new(
                format!("{}/authResponseHeadersRegex", base),
                auth_response_headers_regex.clone(),
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
}

// Add configuration structs for each middleware type
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RedirectRegexConfig {
    pub permanent: bool,
    pub regex: String,
    pub replacement: String,
}

impl ToEtcdPairs for RedirectRegexConfig {
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let base_key = "redirectRegex";
        Ok(vec![
            EtcdPair::new(
                format!("{}/permanent", base_key),
                self.permanent.to_string(),
            ),
            EtcdPair::new(format!("{}/regex", base_key), self.regex.clone()),
            EtcdPair::new(
                format!("{}/replacement", base_key),
                self.replacement.clone(),
            ),
        ])
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RedirectSchemeConfig {
    pub scheme: String,
    pub permanent: bool,
    pub port: Option<String>,
}

impl ToEtcdPairs for RedirectSchemeConfig {
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base_key = "redirectScheme";
        pairs.push(EtcdPair::new(
            format!("{}/scheme", base_key),
            self.scheme.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!("{}/permanent", base_key),
            self.permanent.to_string(),
        ));
        if let Some(port) = &self.port {
            pairs.push(EtcdPair::new(format!("{}/port", base_key), port.clone()));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct StripPrefixConfig {
    pub prefixes: Vec<String>,
    pub force_slash: Option<bool>,
}

impl ToEtcdPairs for StripPrefixConfig {
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base_key = "stripPrefix";

        for (idx, prefix) in self.prefixes.iter().enumerate() {
            pairs.push(EtcdPair::new(
                format!("{}/prefixes/{}", base_key, idx),
                prefix.clone(),
            ));
        }
        if let Some(force_slash) = self.force_slash {
            pairs.push(EtcdPair::new(
                format!("{}/forceSlash", base_key),
                force_slash.to_string(),
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
    pub average: u32,
    pub burst: u32,
    pub period: Option<String>,
}

impl ToEtcdPairs for RateLimitConfig {
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let base_key = "rateLimit";
        Ok(vec![
            EtcdPair::new(format!("{}/average", base_key), self.average.to_string()),
            EtcdPair::new(format!("{}/burst", base_key), self.burst.to_string()),
            EtcdPair::new(
                format!("{}/period", base_key),
                self.period.clone().unwrap_or_default(),
            ),
        ])
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct BasicAuthConfig {
    pub users: Vec<String>,
    pub realm: Option<String>,
    pub header_field: Option<String>,
}

impl ToEtcdPairs for BasicAuthConfig {
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let base_key = "basicAuth";
        for (idx, user) in self.users.iter().enumerate() {
            pairs.push(EtcdPair::new(
                format!("{}/users/{}", base_key, idx),
                format_list_value(&[user.clone()]),
            ));
        }
        if let Some(realm) = &self.realm {
            pairs.push(EtcdPair::new(format!("{}/realm", base_key), realm.clone()));
        }
        if let Some(header_field) = &self.header_field {
            pairs.push(EtcdPair::new(
                format!("{}/headerField", base_key),
                header_field.clone(),
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
    pub expression: String,
}

impl ToEtcdPairs for CircuitBreakerConfig {
    fn to_etcd_pairs(&self, _base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let base_key = "circuitBreaker";
        Ok(vec![EtcdPair::new(
            format!("{}/expression", base_key),
            self.expression.clone(),
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
    fn to_etcd_pairs(&self, root_base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        // let base_key = format!("{}/{}", base_key, self.name);
        let base_key = self.name.clone();

        // All middleware configurations should be under the protocol path
        if let Some(headers) = &self.headers {
            pairs.extend(headers.to_etcd_pairs(&base_key)?);
        }

        if let Some(forward_auth) = &self.forward_auth {
            pairs.extend(forward_auth.to_etcd_pairs(&base_key)?);
        }

        if let Some(redirect_regex) = &self.redirect_regex {
            pairs.extend(redirect_regex.to_etcd_pairs(&base_key)?);
        }

        if let Some(redirect_scheme) = &self.redirect_scheme {
            pairs.extend(redirect_scheme.to_etcd_pairs(&base_key)?);
        }

        if let Some(strip_prefix) = &self.strip_prefix {
            pairs.extend(strip_prefix.to_etcd_pairs(&base_key)?);
        }

        if let Some(rate_limit) = &self.rate_limit {
            pairs.extend(rate_limit.to_etcd_pairs(&base_key)?);
        }

        if let Some(basic_auth) = &self.basic_auth {
            pairs.extend(basic_auth.to_etcd_pairs(&base_key)?);
        }

        if self.compress {
            pairs.push(EtcdPair::new("compress", "true".to_string()));
        }

        if let Some(circuit_breaker) = &self.circuit_breaker {
            pairs.extend(circuit_breaker.to_etcd_pairs(&base_key)?);
        }

        let prefixed_pairs = pairs
            .iter()
            .map(|pair| {
                EtcdPair::new(
                    format!("{}/{}/{}", root_base_key, base_key, pair.key()),
                    pair.value(),
                )
            })
            .collect();

        Ok(prefixed_pairs)
    }
}

impl Validate for MiddlewareConfig {
    /// Validate the middleware configuration
    fn validate(&self) -> TraefikResult<()> {
        if self.name.is_empty() {
            return Err(TraefikError::MiddlewareConfig(
                "middleware name is empty".into(),
            ));
        }

        validate_is_alphanumeric(&self.name)?;

        if let Some(headers) = &self.headers {
            headers.validate()?;
        }
        Ok(())
    }
}

impl MiddlewareConfig {
    pub fn get_safe_key(&self) -> String {
        get_safe_key(&self.name)
    }

    pub fn get_path(&self, base_key: &str) -> String {
        format!("{}/{}/middlewares", base_key, self.protocol,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{
        assert_contains_pair, create_base_middleware_config, create_test_middleware,
    };

    #[test]
    fn test_headers_config_validate() {
        let middleware = create_test_middleware();
        middleware
            .get("enable-headers")
            .unwrap()
            .validate()
            .unwrap();
        assert!(!middleware.contains_key("invalid-middleware"));
    }

    #[test]
    fn test_middleware_is_invalid_if_name_is_empty() {
        let middleware = MiddlewareConfig {
            name: "".to_string(),
            ..Default::default()
        };
        assert!(middleware.validate().is_err());
    }

    #[test]
    fn test_middleware_is_invalid_if_name_is_not_alphanumeric_or_hyphens() {
        let middleware = MiddlewareConfig {
            name: "invalid-%middleware".to_string(),
            ..Default::default()
        };
        assert!(middleware.validate().is_err());
    }

    #[test]
    fn test_middleware_is_valid_if_name_is_alphanumeric_or_hyphens() {
        let middleware = MiddlewareConfig {
            name: "valid-middleware".to_string(),
            ..Default::default()
        };
        assert!(middleware.validate().is_ok());
    }

    #[test]
    fn test_to_etcd_pairs_global() {
        let middleware = create_test_middleware();
        let mut result_pairs = vec![];
        for (_name, middleware) in middleware {
            let pairs = middleware.to_etcd_pairs("test/middlewares").unwrap();
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
            let base_key = middleware.get_path("test");
            let pairs = middleware.to_etcd_pairs(&base_key).unwrap();
            result_pairs.extend(pairs);
        }
        let pair_strs: Vec<String> = result_pairs.iter().map(|p| p.to_string()).collect();

        println!("pair_strs: {:?}", pair_strs);
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
            address: Some("http://localhost:8080".to_string()),
            trust_forward_header: Some(true),
            auth_response_headers: Some(vec!["X-Forwarded-Proto".to_string()]),
            auth_response_headers_regex: Some(".*".to_string()),
            auth_request_headers: None,
        };
        let pairs = forward_auth.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"forwardAuth/address http://localhost:8080".to_string()));
        assert!(pair_strs.contains(&"forwardAuth/trustForwardHeader true".to_string()));
    }

    #[test]
    fn test_middleware_config_to_etcd_pairs_with_forward_auth() {
        let forward_auth = ForwardAuthConfig {
            address: None,
            trust_forward_header: Some(true),
            auth_response_headers: Some(vec!["X-Forwarded-Proto".to_string()]),
            auth_response_headers_regex: Some(".*".to_string()),
            auth_request_headers: None,
        };
        let pairs = forward_auth.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"forwardAuth/trustForwardHeader true".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_auth_response_headers() {
        let forward_auth = ForwardAuthConfig {
            address: None,
            trust_forward_header: Some(true),
            auth_response_headers: Some(vec![
                "X-Forwarded-Proto".to_string(),
                "ServiceAddr".to_string(),
                "ServiceUrl".to_string(),
            ]),
            auth_response_headers_regex: None,
            auth_request_headers: None,
        };
        let pairs = forward_auth.to_etcd_pairs("test").unwrap();
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
            trust_forward_header: Some(true),
            auth_response_headers: None,
            auth_response_headers_regex: Some("^X-.*".to_string()),
            auth_request_headers: None,
        };
        let pairs = forward_auth.to_etcd_pairs("test").unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"forwardAuth/authResponseHeadersRegex ^X-.*".to_string()));
    }

    #[test]
    fn test_middleware_headers() {
        let mut headers = HeadersConfig::default();
        headers
            .custom_request_headers
            .insert("X-Forwarded-Proto".to_string(), "https".to_string());

        let middleware = MiddlewareConfig {
            headers: Some(headers),
            ..create_base_middleware_config()
        };

        let pairs = middleware.to_etcd_pairs("test").unwrap();
        assert_contains_pair(
            &pairs,
            "test/test-middleware/headers/customRequestHeaders/X-Forwarded-Proto https",
        );
    }

    #[test]
    fn test_middleware_redirect_regex() {
        let redirect_regex = RedirectRegexConfig {
            permanent: true,
            regex: "^/old/(.*)".to_string(),
            replacement: "/new/$1".to_string(),
        };

        let middleware = MiddlewareConfig {
            redirect_regex: Some(redirect_regex),
            ..create_base_middleware_config()
        };

        let pairs = middleware.to_etcd_pairs("test").unwrap();
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
            scheme: "https".to_string(),
            permanent: true,
            port: Some("443".to_string()),
        };

        let middleware = MiddlewareConfig {
            redirect_scheme: Some(redirect_scheme),
            ..create_base_middleware_config()
        };

        let pairs = middleware.to_etcd_pairs("test").unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/redirectScheme/scheme https");
        assert_contains_pair(&pairs, "test/test-middleware/redirectScheme/permanent true");
        assert_contains_pair(&pairs, "test/test-middleware/redirectScheme/port 443");
    }

    #[test]
    fn test_middleware_strip_prefix() {
        let strip_prefix = StripPrefixConfig {
            prefixes: vec!["/api".to_string(), "/v1".to_string()],
            force_slash: Some(true),
        };

        let middleware = MiddlewareConfig {
            strip_prefix: Some(strip_prefix),
            ..create_base_middleware_config()
        };

        let pairs = middleware.to_etcd_pairs("test").unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/stripPrefix/prefixes/0 /api");
        assert_contains_pair(&pairs, "test/test-middleware/stripPrefix/prefixes/1 /v1");
        assert_contains_pair(&pairs, "test/test-middleware/stripPrefix/forceSlash true");
    }

    #[test]
    fn test_middleware_rate_limit() {
        let rate_limit = RateLimitConfig {
            average: 100,
            burst: 200,
            period: Some("1s".to_string()),
        };

        let middleware = MiddlewareConfig {
            rate_limit: Some(rate_limit),
            ..create_base_middleware_config()
        };

        let pairs = middleware.to_etcd_pairs("test").unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/rateLimit/average 100");
        assert_contains_pair(&pairs, "test/test-middleware/rateLimit/burst 200");
        assert_contains_pair(&pairs, "test/test-middleware/rateLimit/period 1s");
    }

    #[test]
    fn test_middleware_basic_auth() {
        let basic_auth = BasicAuthConfig {
            users: vec!["user:password".to_string()],
            realm: Some("My Realm".to_string()),
            header_field: Some("X-Auth".to_string()),
        };

        let middleware = MiddlewareConfig {
            basic_auth: Some(basic_auth),
            ..create_base_middleware_config()
        };

        let pairs = middleware.to_etcd_pairs("test").unwrap();
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

        let pairs = middleware.to_etcd_pairs("test").unwrap();
        assert_contains_pair(&pairs, "test/test-middleware/compress true");
    }

    #[test]
    fn test_middleware_circuit_breaker() {
        let circuit_breaker = CircuitBreakerConfig {
            expression: "NetworkErrorRatio() > 0.5".to_string(),
        };

        let middleware = MiddlewareConfig {
            circuit_breaker: Some(circuit_breaker),
            ..create_base_middleware_config()
        };

        let pairs = middleware.to_etcd_pairs("test").unwrap();
        assert_contains_pair(
            &pairs,
            "test/test-middleware/circuitBreaker/expression NetworkErrorRatio() > 0.5",
        );
    }
}
