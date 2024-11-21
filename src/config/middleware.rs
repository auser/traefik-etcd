use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    core::Validate,
    error::{TraefikError, TraefikResult},
};

use super::headers::HeadersConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MiddlewareConfig {
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HeadersConfig>,
}

impl MiddlewareConfig {
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl Validate for MiddlewareConfig {
    fn validate(&self) -> TraefikResult<()> {
        if self.name.is_empty() {
            return Err(TraefikError::MiddlewareConfig(
                "middleware name is empty".into(),
            ));
        }

        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(TraefikError::MiddlewareConfig(format!(
                "middleware name must be alphanumeric or contain hyphens: {}",
                self.name
            )));
        }

        if let Some(headers) = &self.headers {
            headers.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_config_validate() {
        let headers = HeadersConfig::default();
        headers.validate().unwrap();
    }
}
