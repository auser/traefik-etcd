use std::fmt::Display;

use crate::error::TraefikResult;

pub trait ToEtcdPairs {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Display for EtcdPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.key(), self.value())
    }
}
