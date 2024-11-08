use crate::common::{
    error::TraefikResult,
    etcd::{EtcdPair, ToEtcdPairs},
};
use serde::{Deserialize, Serialize};

use super::load_balancer::LoadBalancerConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceConfig {
    pub(crate) load_balancer: LoadBalancerConfig,
    pub(crate) access_control_allow_headers: Option<Vec<String>>,
    pub(crate) access_control_expose_headers: Option<Vec<String>>,
}

impl ToEtcdPairs for ServiceConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/passHostHeader", base_key),
            self.load_balancer.pass_host_header.to_string(),
        ));

        for (i, server) in self.load_balancer.servers.iter().enumerate() {
            pairs.push(EtcdPair::new(
                format!("{}/loadBalancer/servers/{}/url", base_key, i),
                server.url.clone(),
            ));
        }

        Ok(pairs)
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            load_balancer: LoadBalancerConfig {
                servers: Vec::new(),
                pass_host_header: true,

                response_forwarding: None,
            },
            access_control_allow_headers: None,
            access_control_expose_headers: None,
        }
    }
}
