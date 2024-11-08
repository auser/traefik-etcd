#![allow(dead_code)]
use std::collections::HashMap;

use crate::config::{core_traits::EtcdPair, DeploymentConfig, HostConfig, PathConfig};

pub fn assert_contains_pair(pairs: &[EtcdPair], key: &str, value: &str) {
    assert!(pairs.iter().any(|p| p.key() == key && p.value() == value));
}

pub fn create_test_deployment() -> DeploymentConfig {
    DeploymentConfig {
        ip: "10.0.0.1".to_string(),
        port: 8080,
        weight: 100,
    }
}

pub fn create_test_host() -> HostConfig {
    HostConfig {
        domain: "test.example.com".to_string(),
        paths: vec![PathConfig {
            path: "/api".to_string(),
            deployments: HashMap::from([(
                "blue".to_string(),
                DeploymentConfig {
                    ip: "10.0.0.1".to_string(),
                    port: 8080,
                    weight: 100,
                },
            )]),
            middlewares: vec!["enable-headers".to_string(), "redirect-handler".to_string()],
            strip_prefix: true,
            pass_through: true,
        }],
        deployments: HashMap::from([(
            "blue".to_string(),
            DeploymentConfig {
                ip: "10.0.0.1".to_string(),
                port: 80,
                weight: 100,
            },
        )]),
        middlewares: vec!["enable-headers".to_string()],
    }
}
