use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::common::{
    error::TraefikResult,
    etcd::{EtcdPair, ToEtcdPairs},
};

use super::{
    backend::BackendConfig, load_balancer::LoadBalancerConfig, paths::PathConfig,
    testendpoint::TestCookieConfig, util::get_safe_key,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct HostConfig {
    pub domain: String,
    #[serde(default)]
    pub entrypoints: Vec<String>,
    #[serde(default)]
    pub middlewares: Vec<String>,
    #[serde(default)]
    pub paths: Vec<PathConfig>,
    #[serde(default)]
    pub deployments: HashMap<String, BackendConfig>,
    #[serde(default)]
    pub test_cookie: Option<TestCookieConfig>,
    #[serde(default)]
    pub tls: bool,
    #[serde(default, flatten)]
    pub load_balancer: Option<LoadBalancerConfig>,
}

impl ToEtcdPairs for HostConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];

        let safe_domain = get_safe_key(&self.domain);

        let host_mapping = format!("Host(`{}`)", self.domain);

        let backend_key = format!("{}/{}", base_key, safe_domain);

        // Setup services
        for (i, (name, deployment)) in self.deployments.iter().enumerate() {
            let service_key = format!("{}/services/{}", backend_key, get_safe_key(name));

            pairs.push(EtcdPair::new(
                format!("{}/loadBalancer/passHostHeader", service_key),
                "true",
            ));
            pairs.push(EtcdPair::new(
                format!("{}/loadBalancer/servers/{}/url", service_key, i),
                format!("http://{}:{}", deployment.ip, deployment.port),
            ));
        }

        pairs.push(EtcdPair::new(
            format!("{}/loadBalancer/passHostHeader", backend_key),
            "true",
        ));

        // First host, then PathPrefix
        pairs.push(EtcdPair::new(
            format!("{}/rule", backend_key),
            &host_mapping,
        ));
        for (i, entrypoint) in self.entrypoints.iter().enumerate() {
            pairs.push(EtcdPair::new(
                format!("{}/entrypoints/{}", backend_key, i),
                entrypoint.clone(),
            ));
        }

        if self.tls {
            pairs.push(EtcdPair::new(format!("{}/tls", backend_key), "true"));
        }

        for (i, middleware) in self.middlewares.iter().enumerate() {
            pairs.push(EtcdPair::new(
                format!("{}/middlewares/{}", backend_key, i),
                middleware.clone(),
            ));
        }

        for (i, path) in self.paths.iter().enumerate() {
            let rule_name = format!("{}-path-{}", safe_domain, i);
            let rule_path = format!("{} && PathPrefix(`{}`)", &host_mapping, path.path);

            let backend_key = format!("{}/{}", base_key, rule_name);
            pairs.push(EtcdPair::new(format!("{}/rule", backend_key), rule_path));
            let path_pairs = path.to_etcd_pairs(&rule_name)?;
            pairs.extend(path_pairs);
        }

        if let Some(load_balancer) = &self.load_balancer {
            let load_balancer_pairs = load_balancer.to_etcd_pairs(&backend_key)?;
            pairs.extend(load_balancer_pairs);
        }

        // Set the service
        let service_key = format!("{}/service", backend_key);
        pairs.push(EtcdPair::new(&service_key, service_key.clone()));

        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::assert_contains_pair;

    use super::*;

    #[test]
    fn test_simple_host() {
        let host = HostConfig {
            domain: "ibs.collegegreen.net".to_string(),
            entrypoints: vec!["websecure".to_string()],
            middlewares: vec!["enable-headers".to_string()],
            paths: vec![PathConfig {
                path: "/".to_string(),
                strip_prefix: false,
                middlewares: None,
                entrypoints: None,
            }],
            deployments: HashMap::new(),
            test_cookie: None,
            tls: true,
            load_balancer: None,
        };

        let pairs = host.to_etcd_pairs("test");
        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();
        assert!(pairs.len() > 0);
        assert_contains_pair(
            &pairs,
            "test/ibs-collegegreen-net/rule",
            "Host(`ibs.collegegreen.net`)",
        );
        assert_contains_pair(
            &pairs,
            "test/ibs-collegegreen-net/entrypoints/0",
            "websecure",
        );
        assert_contains_pair(
            &pairs,
            "test/ibs-collegegreen-net/middlewares/0",
            "enable-headers",
        );
        assert_contains_pair(
            &pairs,
            "test/ibs-collegegreen-net-path-0/rule",
            "Host(`ibs.collegegreen.net`) && PathPrefix(`/`)",
        );
    }
}
