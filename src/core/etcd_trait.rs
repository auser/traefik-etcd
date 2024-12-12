use std::{fmt::Display, path::PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::TraefikResult;

use super::templating::{TemplateContext, TemplateResolver};

pub trait ToEtcdPairs {
    fn to_etcd_pairs(
        &self,
        base_key: &str,
        resolver: &mut impl TemplateResolver,
        context: &TemplateContext,
    ) -> TraefikResult<Vec<EtcdPair>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct EtcdPair(String, String);

impl EtcdPair {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        EtcdPair(key.into(), value.into())
    }

    pub fn key(&self) -> &str {
        &self.0
    }

    pub fn value(&self) -> &str {
        &self.1
    }
}

// Add these utility implementations
impl EtcdPair {
    pub fn from_file(path: &PathBuf) -> TraefikResult<Vec<EtcdPair>> {
        let content = std::fs::read_to_string(path)?;
        Ok(content
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let mut parts = line.splitn(2, ' ');
                EtcdPair::new(
                    parts.next().unwrap_or_default(),
                    parts.next().unwrap_or_default(),
                )
            })
            .collect())
    }
}

impl Display for EtcdPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.key(), self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file() {
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmpfile.path(), "key1 value1\nkey2 value2").unwrap();
        let pairs = EtcdPair::from_file(&tmpfile.path().to_path_buf()).unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].key(), "key1");
        assert_eq!(pairs[0].value(), "value1");
        assert_eq!(pairs[1].key(), "key2");
        assert_eq!(pairs[1].value(), "value2");
    }
}
