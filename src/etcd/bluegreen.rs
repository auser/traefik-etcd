use serde::{Deserialize, Serialize};

use crate::common::{
    error::{ConfigError, TraefikResult},
    etcd::{EtcdPair, ToEtcdPairs},
};

use super::backend::BackendConfig;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BackendDistributionConfig {
    pub backends: Vec<BackendConfig>,
    pub weight: u8,                  // 0-100, percentage of traffic to green
    pub cookie_name: Option<String>, // Optional cookie for sticky sessions
}

impl ToEtcdPairs for BackendDistributionConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        let mut cumulative_weight = self
            .backends
            .iter()
            .fold(0, |acc, backend| acc + backend.weight.unwrap_or(0));

        if cumulative_weight != 100 {
            return Err(ConfigError::BackendConfig(
                "Backend weights must sum to 100".to_string(),
            ));
        }

        for (_i, backend) in self.backends.iter().enumerate() {
            let backend_name = format!("backend-{}", _i);
            // Blue service
            pairs.push(EtcdPair::new(
                format!("{}/{}/loadBalancer/servers/0/url", base_key, backend_name),
                format!("http://{}:{}", backend.ip, backend.port),
            ));
            cumulative_weight = cumulative_weight + backend.weight.unwrap_or(0);
            pairs.push(EtcdPair::new(
                format!("{}/{}/loadBalancer/passHostHeader", base_key, backend_name),
                "true".to_string(),
            ));

            pairs.push(EtcdPair::new(
                format!("{}/weighted/services/0/name", base_key),
                format!("{}@etcd", backend_name),
            ));

            pairs.push(EtcdPair::new(
                format!("{}/weighted/services/0/weight", base_key),
                backend.weight.unwrap_or(0).to_string(),
            ));

            if let Some(cookie_name) = &self.cookie_name {
                pairs.push(EtcdPair::new(
                    format!("{}/weighted/sticky/cookie/name", base_key),
                    cookie_name.clone(),
                ));
            }
        }

        Ok(pairs)
    }
}
