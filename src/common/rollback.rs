use std::collections::HashMap;

use crate::{
    config::TraefikConfig,
    etcd::{
        backend::BackendConfig,
        host::HostConfig,
        middleware::{HeadersMiddleware, MiddlewareConfig, RedirectRegexMiddleware},
    },
};

use super::{
    changes::show_host_changes,
    error::TraefikResult,
    etcd::{Etcd, EtcdConfig, KeyValue},
};

/// Get current configuration from etcd
pub async fn get_current_config(client: &mut Etcd) -> TraefikResult<TraefikConfig> {
    // Get all keys under traefik/http/
    let kvs = client.get_with_prefix("traefik/http/").await?;

    // Create empty config structure
    let mut config = TraefikConfig {
        etcd: EtcdConfig::default(),
        middlewares: HashMap::new(),
        hosts: Vec::new(),
    };

    // Group keys by their type (middleware, router, service)
    let mut middleware_kvs: HashMap<String, Vec<KeyValue>> = HashMap::new();
    let mut router_kvs: HashMap<String, Vec<KeyValue>> = HashMap::new();
    let mut service_kvs: HashMap<String, Vec<KeyValue>> = HashMap::new();

    for kv in kvs {
        let key = std::str::from_utf8(kv.key()).unwrap();
        let parts: Vec<&str> = key.split('/').collect();

        match parts.get(3) {
            Some(&"middlewares") => {
                let name = parts[4].to_string();
                middleware_kvs.entry(name).or_default().push(kv);
            }
            Some(&"routers") => {
                let name = parts[4].to_string();
                router_kvs.entry(name).or_default().push(kv);
            }
            Some(&"services") => {
                let name = parts[4].to_string();
                service_kvs.entry(name).or_default().push(kv);
            }
            _ => continue,
        }
    }

    // Reconstruct middlewares
    for (name, kvs) in middleware_kvs {
        let middleware = reconstruct_middleware(&kvs)?;
        config.middlewares.insert(name, middleware);
    }

    // Reconstruct hosts from routers and services
    for (name, kvs) in router_kvs {
        if let Some(host) = reconstruct_host(&name, &kvs, &service_kvs)? {
            config.hosts.push(host);
        }
    }

    Ok(config)
}

fn reconstruct_middleware(kvs: &[KeyValue]) -> TraefikResult<MiddlewareConfig> {
    let mut headers = HeadersMiddleware::default();
    let mut redirect_regex = None;

    for kv in kvs {
        let key = std::str::from_utf8(kv.key()).unwrap();
        let value = std::str::from_utf8(kv.value()).unwrap();
        let parts: Vec<&str> = key.split('/').collect();

        match parts.get(5..) {
            Some(["headers", "customRequestHeaders", header_name]) => {
                headers
                    .custom_request_headers
                    .get_or_insert_with(HashMap::new)
                    .insert(header_name.to_string(), value.to_string());
            }
            Some(["headers", "customResponseHeaders", header_name]) => {
                headers
                    .custom_response_headers
                    .get_or_insert_with(HashMap::new)
                    .insert(header_name.to_string(), value.to_string());
            }
            Some(["headers", "accessControlAllowMethods"]) => {
                headers
                    .access_control_allow_methods
                    .get_or_insert(Vec::new())
                    .push(value.to_string());
            }
            Some(["headers", "accessControlAllowHeaders"]) => {
                headers
                    .access_control_expose_headers
                    .get_or_insert(Vec::new())
                    .push(value.to_string());
            }
            Some(["headers", "accessControlExposeHeaders"]) => {
                headers
                    .access_control_expose_headers
                    .get_or_insert(Vec::new())
                    .push(value.to_string());
            }
            Some(["headers", "addVaryHeader"]) => {
                headers.add_vary_header = value.parse()?;
            }
            Some(["redirectregex", field]) => {
                redirect_regex.get_or_insert(RedirectRegexMiddleware {
                    permanent: false,
                    regex: String::new(),
                    replacement: String::new(),
                });

                if let Some(redirect) = &mut redirect_regex {
                    match *field {
                        "permanent" => redirect.permanent = value.parse()?,
                        "regex" => redirect.regex = value.to_string(),
                        "replacement" => redirect.replacement = value.to_string(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(MiddlewareConfig {
        headers: Some(headers),
        redirect_regex,
        ..Default::default()
    })
}

fn reconstruct_host(
    name: &str,
    router_kvs: &[KeyValue],
    service_kvs: &HashMap<String, Vec<KeyValue>>,
) -> TraefikResult<Option<HostConfig>> {
    let mut deployments = HashMap::new();
    deployments.insert("blue".to_string(), BackendConfig::default());
    deployments.insert("green".to_string(), BackendConfig::default());

    let mut host = HostConfig {
        domain: name.replace("-", "."),
        entrypoints: Vec::new(),
        middlewares: Vec::new(),
        deployments,
        tls: false,
        paths: Vec::new(),
        test_cookie: None,
        load_balancer: None,
    };

    // Parse router configuration
    for kv in router_kvs {
        let key = std::str::from_utf8(kv.key()).unwrap();
        let value = std::str::from_utf8(kv.value()).unwrap();
        let parts: Vec<&str> = key.split('/').collect();

        match parts.get(5..) {
            Some(["rule"]) => {
                // Extract domain from Host(`domain`) rule
                if let Some(domain) = value
                    .trim_start_matches("Host(`")
                    .trim_end_matches(")`")
                    .to_string()
                    .into()
                {
                    host.domain = domain;
                }
            }
            Some(["entryPoints", _]) => {
                host.entrypoints.push(value.to_string());
            }
            Some(["middlewares", _]) => {
                host.middlewares.push(value.to_string());
            }
            _ => {}
        }
    }

    // Parse service configuration
    if let Some(service_kvs) = service_kvs.get(name) {
        for kv in service_kvs {
            let key = std::str::from_utf8(kv.key()).unwrap();
            let value = std::str::from_utf8(kv.value()).unwrap();
            let parts: Vec<&str> = key.split('/').collect();

            match parts.get(5..) {
                Some(["loadBalancer", "servers", _, "url"]) => {
                    // Parse backend URL
                    if let Ok(url) = url::Url::parse(value) {
                        host.deployments.get_mut("blue").unwrap().ip =
                            url.host_str().unwrap_or("").to_string();
                        host.deployments.get_mut("blue").unwrap().port = url.port().unwrap_or(80);
                    }
                }
                Some(["weighted", "services", idx, "weight"]) => {
                    let weight: u8 = value.parse()?;
                    match *idx {
                        "0" => host.deployments.get_mut("blue").unwrap().weight = Some(weight),
                        "1" => host.deployments.get_mut("green").unwrap().weight = Some(weight),

                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    Ok(Some(host))
}

async fn show_diff(client: &mut Etcd, new_config: &TraefikConfig) -> TraefikResult<()> {
    let current = get_current_config(client).await?;

    // Now we can compare current with new_config
    for host in &new_config.hosts {
        if let Some(current_host) = current.hosts.iter().find(|h| h.domain == host.domain) {
            show_host_changes(client, current_host).await?;
        } else {
            println!("New host: {}", host.domain);
        }
    }

    Ok(())
}
