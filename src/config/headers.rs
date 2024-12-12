use std::collections::{HashMap, HashSet};

use export_type::ExportType;
use schemars::JsonSchema;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    core::{
        etcd_trait::{EtcdPair, ToEtcdPairs},
        templating::{is_template, TemplateContext, TemplateOr, TemplateResolver},
        util::format_list_value,
        Validate,
    },
    error::{TraefikError, TraefikResult},
};

#[derive(Debug, Serialize, Default, Clone, PartialEq, Eq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct HeadersConfig {
    #[serde(default)]
    pub headers: HashMap<String, TemplateOr<String>>,
    #[serde(default)]
    pub custom_request_headers: HashMap<String, TemplateOr<String>>,
    #[serde(default)]
    pub custom_response_headers: HashMap<String, TemplateOr<String>>,
    #[serde(default)]
    pub access_control_allow_methods: Vec<TemplateOr<String>>,
    #[serde(default)]
    pub access_control_allow_headers: Vec<TemplateOr<String>>,
    #[serde(default)]
    pub access_control_expose_headers: Vec<TemplateOr<String>>,
    #[serde(default)]
    pub access_control_allow_origin_list: Vec<TemplateOr<String>>,
    #[serde(default)]
    pub add_vary_header: bool,
}

impl<'de> Deserialize<'de> for HeadersConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Headers,
            CustomRequestHeaders,
            CustomResponseHeaders,
            AccessControlAllowMethods,
            AccessControlAllowHeaders,
            AccessControlExposeHeaders,
            AccessControlAllowOriginList,
            AddVaryHeader,
        }

        struct HeadersConfigVisitor;

        impl<'de> Visitor<'de> for HeadersConfigVisitor {
            type Value = HeadersConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct HeadersConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<HeadersConfig, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut headers = HashMap::new();
                let mut custom_request_headers = HashMap::new();
                let mut custom_response_headers = HashMap::new();
                let mut access_control_allow_methods = Vec::new();
                let mut access_control_allow_headers = Vec::new();
                let mut access_control_expose_headers = Vec::new();
                let mut access_control_allow_origin_list = Vec::new();
                let mut add_vary_header = false;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Headers => {
                            let values: HashMap<String, String> = map.next_value()?;
                            for (k, v) in values {
                                headers.insert(
                                    k,
                                    if is_template(&v) {
                                        TemplateOr::Template(v)
                                    } else {
                                        TemplateOr::Static(v)
                                    },
                                );
                            }
                        }
                        Field::CustomRequestHeaders => {
                            let values: HashMap<String, String> = map.next_value()?;
                            for (k, v) in values {
                                custom_request_headers.insert(
                                    k,
                                    if is_template(&v) {
                                        TemplateOr::Template(v)
                                    } else {
                                        TemplateOr::Static(v)
                                    },
                                );
                            }
                        }
                        Field::CustomResponseHeaders => {
                            let values: HashMap<String, String> = map.next_value()?;
                            for (k, v) in values {
                                custom_response_headers.insert(
                                    k,
                                    if is_template(&v) {
                                        TemplateOr::Template(v)
                                    } else {
                                        TemplateOr::Static(v)
                                    },
                                );
                            }
                        }
                        Field::AccessControlAllowMethods => {
                            let values: Vec<String> = map.next_value()?;
                            access_control_allow_methods = values
                                .into_iter()
                                .map(|v| {
                                    if is_template(&v) {
                                        TemplateOr::Template(v)
                                    } else {
                                        TemplateOr::Static(v)
                                    }
                                })
                                .collect();
                        }
                        Field::AccessControlAllowHeaders => {
                            let values: Vec<String> = map.next_value()?;
                            access_control_allow_headers = values
                                .into_iter()
                                .map(|v| {
                                    if is_template(&v) {
                                        TemplateOr::Template(v)
                                    } else {
                                        TemplateOr::Static(v)
                                    }
                                })
                                .collect();
                        }
                        Field::AccessControlExposeHeaders => {
                            let values: Vec<String> = map.next_value()?;
                            access_control_expose_headers = values
                                .into_iter()
                                .map(|v| {
                                    if is_template(&v) {
                                        TemplateOr::Template(v)
                                    } else {
                                        TemplateOr::Static(v)
                                    }
                                })
                                .collect();
                        }
                        Field::AccessControlAllowOriginList => {
                            let values: Vec<String> = map.next_value()?;
                            access_control_allow_origin_list = values
                                .into_iter()
                                .map(|v| {
                                    if is_template(&v) {
                                        TemplateOr::Template(v)
                                    } else {
                                        TemplateOr::Static(v)
                                    }
                                })
                                .collect();
                        }
                        Field::AddVaryHeader => {
                            add_vary_header = map.next_value()?;
                        }
                    }
                }

                Ok(HeadersConfig {
                    headers,
                    custom_request_headers,
                    custom_response_headers,
                    access_control_allow_methods,
                    access_control_allow_headers,
                    access_control_expose_headers,
                    access_control_allow_origin_list,
                    add_vary_header,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "headers",
            "custom_request_headers",
            "custom_response_headers",
            "access_control_allow_methods",
            "access_control_allow_headers",
            "access_control_expose_headers",
            "access_control_allow_origin_list",
            "add_vary_header",
        ];

        deserializer.deserialize_struct("HeadersConfig", FIELDS, HeadersConfigVisitor)
    }
}

impl ToEtcdPairs for HeadersConfig {
    fn to_etcd_pairs(
        &self,
        base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];

        for (key, value) in &self.headers {
            pairs.push(EtcdPair::new(
                format!("{}/headers/{}", base_key, key),
                value.resolve(resolver, context)?,
            ));
        }

        for (key, value) in &self.custom_request_headers {
            pairs.push(EtcdPair::new(
                format!("{}/headers/customRequestHeaders/{}", base_key, key),
                value.resolve(resolver, context)?,
            ));
        }

        for (key, value) in &self.custom_response_headers {
            pairs.push(EtcdPair::new(
                format!("{}/headers/customResponseHeaders/{}", base_key, key),
                value.resolve(resolver, context)?,
            ));
        }

        if !self.access_control_allow_methods.is_empty() {
            let resolved_methods: Result<Vec<String>, _> = self
                .access_control_allow_methods
                .iter()
                .map(|m| m.resolve(resolver, context))
                .collect();
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlAllowMethods", base_key),
                format_list_value(&resolved_methods?),
            ));
        }

        if !self.access_control_allow_headers.is_empty() {
            let resolved_headers: Result<Vec<String>, _> = self
                .access_control_allow_headers
                .iter()
                .map(|h| h.resolve(resolver, context))
                .collect();
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlAllowHeaders", base_key),
                format_list_value(&resolved_headers?),
            ));
        }

        if !self.access_control_expose_headers.is_empty() {
            let resolved_headers: Result<Vec<String>, _> = self
                .access_control_expose_headers
                .iter()
                .map(|h| h.resolve(resolver, context))
                .collect();
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlExposeHeaders", base_key),
                format_list_value(&resolved_headers?),
            ));
        }

        if !self.access_control_allow_origin_list.is_empty() {
            let resolved_origins: Result<Vec<String>, _> = self
                .access_control_allow_origin_list
                .iter()
                .map(|o| o.resolve(resolver, context))
                .collect();
            pairs.push(EtcdPair::new(
                format!("{}/headers/accessControlAllowOriginList", base_key),
                format_list_value(&resolved_origins?),
            ));
        }

        if self.add_vary_header {
            pairs.push(EtcdPair::new(
                format!("{}/headers/addVaryHeader", base_key),
                self.add_vary_header.to_string(),
            ));
        }

        Ok(pairs)
    }
}

impl Validate for HeadersConfig {
    fn validate(
        &self,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<()> {
        // Validate HTTP methods
        let valid_methods: HashSet<&str> = vec![
            "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE", "CONNECT",
        ]
        .into_iter()
        .collect();

        for method in &self.access_control_allow_methods {
            if !valid_methods.contains(method.resolve(resolver, context)?.as_str()) {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Invalid HTTP method: {}",
                    method.resolve(resolver, context)?
                )));
            }
        }

        // Validate custom headers
        self.validate_header_names(&self.custom_request_headers, resolver, context)?;
        self.validate_header_names(&self.custom_response_headers, resolver, context)?;

        // Validate header lists
        self.validate_header_list(
            &self.access_control_allow_headers,
            "Access-Control-Allow-Headers",
            resolver,
            context,
        )?;
        self.validate_header_list(
            &self.access_control_expose_headers,
            "Access-Control-Expose-Headers",
            resolver,
            context,
        )?;

        // Validate header values
        for (name, value) in &self.custom_request_headers {
            self.validate_header_value(name, value, resolver, context)?;
        }
        for (name, value) in &self.custom_response_headers {
            self.validate_header_value(name, value, resolver, context)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadersConfigBuilder {
    pub custom_request_headers: HashMap<String, TemplateOr<String>>,
    pub custom_response_headers: HashMap<String, TemplateOr<String>>,
    pub access_control_allow_methods: Vec<TemplateOr<String>>,
    pub access_control_allow_headers: Vec<TemplateOr<String>>,
    pub access_control_expose_headers: Vec<TemplateOr<String>>,
    pub access_control_allow_origin_list: Vec<TemplateOr<String>>,
    pub auth_response_headers: Vec<TemplateOr<String>>,
    pub auth_response_headers_regex: TemplateOr<String>,
    pub add_vary_header: bool,
    pub headers: HashMap<String, TemplateOr<String>>,
}

impl HeadersConfigBuilder {
    pub fn add_custom_request_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.custom_request_headers
            .insert(name.to_string(), TemplateOr::Static(value.to_string()));
        self
    }

    pub fn add_custom_response_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.custom_response_headers
            .insert(name.to_string(), TemplateOr::Static(value.to_string()));
        self
    }

    pub fn add_access_control_allow_method(&mut self, method: &str) -> &mut Self {
        self.access_control_allow_methods
            .push(TemplateOr::Static(method.to_string()));
        self
    }

    pub fn add_access_control_allow_header(&mut self, header: &str) -> &mut Self {
        self.access_control_allow_headers
            .push(TemplateOr::Static(header.to_string()));
        self
    }

    pub fn add_access_control_expose_header(&mut self, header: &str) -> &mut Self {
        self.access_control_expose_headers
            .push(TemplateOr::Static(header.to_string()));
        self
    }

    pub fn add_access_control_allow_origin(&mut self, origin: &str) -> &mut Self {
        self.access_control_allow_origin_list
            .push(TemplateOr::Static(origin.to_string()));
        self
    }

    pub fn add_vary_header(&mut self, add_vary_header: bool) -> &mut Self {
        self.add_vary_header = add_vary_header;
        self
    }

    pub fn add_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers
            .insert(name.to_string(), TemplateOr::Static(value.to_string()));
        self
    }

    pub fn build(&self) -> HeadersConfig {
        HeadersConfig {
            headers: self.headers.clone(),
            custom_request_headers: self.custom_request_headers.clone(),
            custom_response_headers: self.custom_response_headers.clone(),
            access_control_allow_methods: self.access_control_allow_methods.clone(),
            access_control_allow_headers: self.access_control_allow_headers.clone(),
            access_control_expose_headers: self.access_control_expose_headers.clone(),
            access_control_allow_origin_list: self.access_control_allow_origin_list.clone(),
            add_vary_header: self.add_vary_header.clone(),
        }
    }
}

impl HeadersConfig {
    pub fn builder() -> HeadersConfigBuilder {
        HeadersConfigBuilder::default()
    }

    fn validate_header_names(
        &self,
        headers: &HashMap<String, TemplateOr<String>>,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> TraefikResult<()> {
        for name in headers.keys() {
            if name.is_empty() {
                return Err(TraefikError::MiddlewareConfig(
                    "Header name cannot be empty".into(),
                ));
            }

            if !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
            {
                return Err(TraefikError::MiddlewareConfig(format!(
                "Invalid header name '{}': must contain only alphanumeric characters, hyphens, underscores, dots, or forward slashes",
                name
            )));
            }
        }
        Ok(())
    }

    fn validate_header_list(
        &self,
        headers: &[TemplateOr<String>],
        ctx: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<()> {
        let mut seen = HashSet::new();
        for header in headers {
            let header_lower = header.resolve(resolver, context)?.to_lowercase();
            if !seen.insert(header_lower) {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Duplicate header in {}: {}",
                    ctx,
                    header.resolve(resolver, context)?
                )));
            }

            if header.resolve(resolver, context)?.is_empty() {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Empty header name in {}",
                    ctx
                )));
            }
        }
        Ok(())
    }

    fn validate_header_value(
        &self,
        name: &str,
        value: &TemplateOr<String>,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<()> {
        if value
            .resolve(resolver, context)?
            .chars()
            .any(|c| c.is_control() && c != '\t')
        {
            return Err(TraefikError::MiddlewareConfig(format!(
                "Invalid value for header '{}': contains control characters",
                name
            )));
        }

        match name.to_lowercase().as_str() {
            "x-forwarded-proto" => {
                if !["http", "https"]
                    .contains(&value.resolve(resolver, context)?.to_lowercase().as_str())
                {
                    return Err(TraefikError::MiddlewareConfig(format!(
                        "Invalid value for X-Forwarded-Proto: must be 'http' or 'https', got '{}'",
                        value.resolve(resolver, context)?
                    )));
                }
            }
            "x-forwarded-port" => {
                if value.resolve(resolver, context)?.parse::<u16>().is_err() {
                    return Err(TraefikError::MiddlewareConfig(format!(
                        "Invalid value for X-Forwarded-Port: must be a valid port number, got '{}'",
                        value.resolve(resolver, context)?
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RuntimeHeadersConfig {
    #[serde(default)]
    pub template_headers: HashMap<String, String>,
}

impl ToEtcdPairs for RuntimeHeadersConfig {
    fn to_etcd_pairs(
        &self,
        _base_key: &str,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>> {
        // This should never be called directly - template processing happens in InternalDeploymentConfig
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{create_test_resolver, create_test_template_context};

    use super::*;

    #[test]
    fn test_validate_header_value() {
        let headers = HeadersConfig::default();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(headers.validate(&mut resolver, &context).is_ok());
    }

    #[test]
    fn test_validate_header_names() {
        let mut headers = HeadersConfig::default();
        headers.custom_request_headers.insert(
            "X-Forwarded-Proto".to_string(),
            TemplateOr::Static("http".to_string()),
        );
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(headers.validate(&mut resolver, &context).is_ok());
    }

    #[test]
    fn test_to_etcd_pairs_with_custom_request_headers() {
        let headers = HeadersConfig::builder()
            .add_custom_request_header("X-Forwarded-Proto", "https")
            .add_custom_request_header("X-Forwarded-Port", "80")
            .build();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = headers
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs
            .contains(&"test/headers/customRequestHeaders/X-Forwarded-Proto https".to_string()));
        assert!(pair_strs
            .contains(&"test/headers/customRequestHeaders/X-Forwarded-Port 80".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_allow_methods() {
        let headers = HeadersConfig::builder()
            .add_access_control_allow_method("GET")
            .add_access_control_allow_method("POST")
            .add_access_control_allow_method("PUT")
            .build();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = headers
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs
            .contains(&"test/headers/accessControlAllowMethods GET, POST, PUT".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_allow_headers() {
        let headers = HeadersConfig::builder()
            .add_access_control_allow_header("Content-Type")
            .add_access_control_allow_header("Content-Length")
            .build();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = headers
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(
            &"test/headers/accessControlAllowHeaders Content-Type, Content-Length".to_string()
        ));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_expose_headers() {
        let headers = HeadersConfig::builder()
            .add_access_control_expose_header("Content-Type")
            .add_access_control_expose_header("Content-Length")
            .build();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = headers
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(
            &"test/headers/accessControlExposeHeaders Content-Type, Content-Length".to_string()
        ));
    }

    #[test]
    fn test_to_etcd_pairs_with_access_control_allow_origin_list() {
        let headers = HeadersConfig::builder()
            .add_access_control_allow_origin("example.com")
            .build();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = headers
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs
            .contains(&"test/headers/accessControlAllowOriginList example.com".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_add_vary_header() {
        let headers = HeadersConfig::builder().add_vary_header(true).build();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = headers
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(&"test/headers/addVaryHeader true".to_string()));
    }

    #[test]
    fn test_to_etcd_pairs_with_custom_response_headers() {
        let headers = HeadersConfig::builder()
            .add_custom_response_header("Content-Type", "application/json")
            .build();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let pairs = headers
            .to_etcd_pairs("test", &mut resolver, &context)
            .unwrap();
        let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
        assert!(pair_strs.contains(
            &"test/headers/customResponseHeaders/Content-Type application/json".to_string()
        ));
    }
}
