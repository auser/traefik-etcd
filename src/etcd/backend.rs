use serde::{Deserialize, Serialize};

use crate::common::{
    error::TraefikResult,
    etcd::{EtcdPair, ToEtcdPairs},
};

use super::healthcheck::HealthCheckConfig;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BackendConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheckConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u8>,
    #[serde(default)]
    pub tls: bool,
}

impl ToEtcdPairs for BackendConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];

        let backend_key = format!("{}/{}", self.name.clone(), base_key);
        let rule = format!("{}/rule", backend_key);
        pairs.push(EtcdPair::new(rule, format!("Host(`{}`)", self.name)));

        Ok(pairs)
    }
}
