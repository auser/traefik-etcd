#![allow(dead_code)]
use std::collections::HashMap;

use crate::config::{
    core_traits::EtcdPair, DeploymentConfig, HeadersConfig, HostConfig, MiddlewareConfig,
    PathConfig, TraefikConfig,
};

pub fn assert_contains_pair(pairs: &[EtcdPair], key: &str, value: &str) {
    assert!(pairs.iter().any(|p| p.key() == key && p.value() == value));
}

pub fn create_test_deployment() -> DeploymentConfig {
    DeploymentConfig {
        ip: "10.0.0.1".to_string(),
        port: 8080,
        weight: 100,
        selection: None,
    }
}

pub fn create_test_host() -> HostConfig {
    let mut host = HostConfig {
        domain: "test.example.com".to_string(),
        paths: Vec::new(),
        deployments: HashMap::new(),
        middlewares: vec![],
        selection: None,
    };

    // Add a default blue deployment
    host.deployments.insert(
        "blue".to_string(),
        DeploymentConfig {
            ip: "10.0.0.1".to_string(),
            port: 80,
            weight: 100,
            selection: None,
        },
    );

    // Add a default path configuration with explicit middleware order
    host.paths.push(PathConfig {
        path: "/api".to_string(),
        deployments: {
            let mut map = HashMap::new();
            map.insert(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 80,
                    weight: 100,
                    selection: None,
                },
            );
            map
        },
        middlewares: vec!["enable-headers".to_string()], // Will be added after headers and strip
        strip_prefix: true,                              // Will be added second
        pass_through: true,
    });

    host
}

pub fn create_test_config(host_configs: Option<Vec<HostConfig>>) -> TraefikConfig {
    let host_configs = host_configs.unwrap_or(vec![HostConfig {
        domain: "test.example.com".to_string(),
        selection: None,
        paths: vec![PathConfig {
            path: "/api".to_string(),
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 80,
                    weight: 100,
                    selection: None,
                },
            )]),
            middlewares: vec!["enable-headers".to_string()],
            strip_prefix: true,
            pass_through: false,
        }],
        deployments: HashMap::from([("blue".to_string(), DeploymentConfig::default())]),
        middlewares: vec![],
    }]);

    TraefikConfig {
        etcd: Default::default(),
        middlewares: create_test_middleware(),
        hosts: host_configs,
        www_redirect: Some(true),
        redirector: Default::default(),
    }
}

fn create_test_middleware() -> HashMap<String, MiddlewareConfig> {
    HashMap::from([
        (
            "enable-headers".to_string(),
            MiddlewareConfig {
                name: "enable-headers".to_string(),
                headers: Some(HeadersConfig {
                    custom_request_headers: HashMap::from([
                        ("X-Forwarded-Proto".to_string(), "https".to_string()),
                        ("X-Forwarded-Port".to_string(), "443".to_string()),
                    ]),
                    custom_response_headers: HashMap::from([(
                        "Location".to_string(),
                        "".to_string(),
                    )]),
                    access_control_allow_methods: vec![
                        "GET".to_string(),
                        "POST".to_string(),
                        "OPTIONS".to_string(),
                    ],
                    access_control_allow_headers: vec![
                        "Content-Type".to_string(),
                        "Authorization".to_string(),
                    ],
                    access_control_expose_headers: vec!["Location".to_string()],
                    access_control_allow_origin_list: vec![],
                    add_vary_header: true,
                }),
            },
        ),
        (
            "handle-redirects".to_string(),
            MiddlewareConfig {
                name: "handle-redirects".to_string(),
                headers: Some(HeadersConfig {
                    custom_request_headers: HashMap::from([(
                        "Location".to_string(),
                        "".to_string(),
                    )]),
                    ..Default::default()
                }),
            },
        ),
    ])
}
