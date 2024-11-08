use serde::{Deserialize, Serialize};

use crate::common::{
    error::TraefikResult,
    etcd::{EtcdPair, ToEtcdPairs},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoadBalancerConfig {
    pub(crate) servers: Vec<ServerConfig>,
    pub(crate) pass_host_header: bool,
    pub(crate) response_forwarding: Option<ResponseForwardingConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseForwardingConfig {
    pub(crate) flush_interval: String,
}

impl ToEtcdPairs for LoadBalancerConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();
        let load_balancer_key = format!("{}/loadBalancer", base_key);

        if self.pass_host_header {
            pairs.push(EtcdPair::new(
                format!("{}/passHostHeader", load_balancer_key),
                "true".to_string(),
            ));
        }

        for (i, server) in self.servers.iter().enumerate() {
            pairs.push(EtcdPair::new(
                format!("{}/servers/{}/url", load_balancer_key, i),
                server.url.clone(),
            ));
        }

        if let Some(response_forwarding) = &self.response_forwarding {
            pairs.push(EtcdPair::new(
                format!("{}/responseForwarding/flushInterval", load_balancer_key),
                response_forwarding.flush_interval.to_string(),
            ));
        }

        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_etcd_pairs() {
        let load_balancer_config = LoadBalancerConfig {
            servers: vec![ServerConfig {
                url: "http://localhost:8080".to_string(),
            }],
            pass_host_header: true,
            response_forwarding: None,
        };

        let pairs = load_balancer_config.to_etcd_pairs("traefik/http/services");
        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(
            pairs[0].key(),
            "traefik/http/services/loadBalancer/passHostHeader"
        );
        assert_eq!(pairs[0].value(), "true");
    }
}
