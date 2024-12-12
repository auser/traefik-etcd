use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        templating::{TemplateContext, TemplateResolver},
        Validate,
    },
    error::{TraefikError, TraefikResult},
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct SelectionConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub with_cookie: Option<WithCookieConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_client_ip: Option<FromClientIpConfig>,
}

impl Validate for SelectionConfig {
    fn validate(
        &self,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<()> {
        if self.with_cookie.is_some() {
            self.with_cookie
                .as_ref()
                .unwrap()
                .validate(resolver, context)?;
        }

        if self.from_client_ip.is_some() {
            self.from_client_ip
                .as_ref()
                .unwrap()
                .validate(resolver, context)?;
        }

        Ok(())
    }
}

/// The configuration for the with cookie selection
/// This is used to select a deployment based on a cookie.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct WithCookieConfig {
    /// The name of the cookie
    pub name: String,
    /// The expected value of the cookie
    #[serde(default)]
    pub value: Option<String>,
}

impl Default for WithCookieConfig {
    fn default() -> Self {
        Self {
            name: "TEST_COOKIE".to_string(),
            value: None,
        }
    }
}

impl Validate for WithCookieConfig {
    fn validate(
        &self,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> TraefikResult<()> {
        if self.name.is_empty() {
            return Err(TraefikError::SelectionConfig("name is empty".to_string()));
        }

        if self.value.is_some() && self.value.as_ref().unwrap().is_empty() {
            return Err(TraefikError::SelectionConfig("value is empty".to_string()));
        }

        Ok(())
    }
}

/// The configuration for the from client ip selection
/// This is used to select a deployment based on the client's ip address
/// or a range of ip addresses.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct FromClientIpConfig {
    /// The range of the ip address
    pub range: Option<String>,
    /// The specific ip address to select
    pub ip: Option<String>,
}

impl Validate for FromClientIpConfig {
    fn validate(
        &self,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> TraefikResult<()> {
        if self.range.is_some() && self.range.as_ref().unwrap().is_empty() {
            return Err(TraefikError::SelectionConfig("range is empty".to_string()));
        }

        if self.ip.is_some() && self.ip.as_ref().unwrap().is_empty() {
            return Err(TraefikError::SelectionConfig("ip is empty".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{create_test_resolver, create_test_template_context};

    use super::*;

    #[test]
    fn test_default_values() {
        let selection = SelectionConfig::default();
        assert_eq!(selection.with_cookie, None);
        assert_eq!(selection.from_client_ip, None);
    }

    #[test]
    fn test_with_cookie_config_default_values() {
        let with_cookie = WithCookieConfig::default();
        assert_eq!(with_cookie.name, "TEST_COOKIE".to_string());
        assert_eq!(with_cookie.value, None);
    }

    #[test]
    fn test_from_client_ip_config_default_values() {
        let from_client_ip = FromClientIpConfig::default();
        assert_eq!(from_client_ip.range, None);
        assert_eq!(from_client_ip.ip, None);
    }

    #[test]
    fn test_with_cookie_config_is_invalid_if_name_is_empty() {
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let with_cookie = WithCookieConfig {
            name: "".to_string(),
            ..Default::default()
        };
        assert!(with_cookie.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_from_client_ip_config_is_invalid_if_range_is_empty() {
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let from_client_ip = FromClientIpConfig {
            range: Some("".to_string()),
            ..Default::default()
        };
        assert!(from_client_ip.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_from_client_ip_config_is_invalid_if_ip_is_empty() {
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let from_client_ip = FromClientIpConfig {
            ip: Some("".to_string()),
            ..Default::default()
        };
        assert!(from_client_ip.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_selection_config_is_valid() {
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        let selection = SelectionConfig::default();
        assert!(selection.validate(&mut resolver, &context).is_ok());
    }
}
