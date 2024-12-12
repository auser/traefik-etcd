use crate::{
    core::{
        templating::{TemplateContext, TemplateResolver},
        Validate,
    },
    error::{TraefikError, TraefikResult},
};
use export_type::ExportType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct HealthCheckConfig {
    pub path: String,
    pub interval: String,
    pub timeout: String,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            interval: "10s".to_string(),
            timeout: "5s".to_string(),
        }
    }
}

impl Validate for HealthCheckConfig {
    fn validate(
        &self,
        _resolver: &mut impl TemplateResolver,
        _context: &TemplateContext,
    ) -> TraefikResult<()> {
        if self.interval.is_empty() || self.timeout.is_empty() || self.path.is_empty() {
            return Err(TraefikError::HealthCheckConfig(
                "interval, timeout and path must not be empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{create_test_resolver, create_test_template_context};

    use super::*;

    #[test]
    fn test_health_check_config_is_invalid_if_interval_is_empty() {
        let health_check = HealthCheckConfig {
            interval: "".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(health_check.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_health_check_config_is_invalid_if_timeout_is_empty() {
        let health_check = HealthCheckConfig {
            timeout: "".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(health_check.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_health_check_config_is_invalid_if_path_is_empty() {
        let health_check = HealthCheckConfig {
            path: "".to_string(),
            ..Default::default()
        };
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(health_check.validate(&mut resolver, &context).is_err());
    }

    #[test]
    fn test_health_check_config_is_valid() {
        let health_check = HealthCheckConfig::default();
        let mut resolver = create_test_resolver();
        let context = create_test_template_context();
        assert!(health_check.validate(&mut resolver, &context).is_ok());
    }
}
