use std::collections::BTreeMap;

use crate::error::TraefikResult;

pub trait ToEtcdPairs {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<BTreeMap<String, Vec<EtcdPair>>>;
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

impl Into<String> for EtcdPair {
    fn into(self) -> String {
        format!("{} {}", self.key(), self.value())
    }
}
