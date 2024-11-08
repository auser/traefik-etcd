use colored::*;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    iter::zip,
};

use crate::etcd::{
    backend::BackendConfig, bluegreen::BackendDistributionConfig, host::HostConfig,
    paths::PathConfig,
};

use super::{error::TraefikResult, etcd::Etcd, rollback::get_current_config};

pub async fn show_host_changes(client: &mut Etcd, host: &HostConfig) -> TraefikResult<()> {
    let current = get_current_config(client).await?;

    // Find existing host config if any
    let current_host = current.hosts.iter().find(|h| h.domain == host.domain);

    match current_host {
        Some(current) => {
            println!("\n{} {}:", "Host".bold(), host.domain.cyan());

            // Compare middlewares
            show_middleware_changes(&current.middlewares, &host.middlewares);

            // Compare deployment config
            for (name, current_deployment) in current.deployments.iter() {
                show_deployment_changes(current_deployment, host.deployments.get(name).unwrap());
            }

            // Compare paths
            show_path_changes(&current.paths, &host.paths);

            // Compare other settings
            // show_setting_changes("TLS", current.tls, host.tls);
            // show_setting_changes("Pass Through", current.pass_through, host.pass_through);

            // Compare entrypoints
            show_array_changes("Entrypoints", &current.entrypoints, &host.entrypoints);
        }
        None => {
            println!(
                "\n{} {} ({})",
                "Host".bold(),
                host.domain.cyan(),
                "New Configuration".green()
            );
            println!("{}", serde_yaml::to_string(host)?);
        }
    }

    Ok(())
}

fn show_middleware_changes(current: &[String], new: &[String]) {
    let current_set: HashSet<_> = current.iter().collect();
    let new_set: HashSet<_> = new.iter().collect();

    let added = new_set.difference(&current_set).collect::<Vec<_>>();
    let removed = current_set.difference(&new_set).collect::<Vec<_>>();

    if !added.is_empty() || !removed.is_empty() {
        println!("\n  {}", "Middlewares:".bold());

        for &middleware in &added {
            println!("    {} {}", "+".green(), middleware);
        }

        for &middleware in &removed {
            println!("    {} {}", "-".red(), middleware);
        }
    }
}

fn show_deployment_changes(current_backend: &BackendConfig, new_backend: &BackendConfig) {
    println!("\n  {}", "Deployment:".bold());

    println!("    {}:", new_backend.name.cyan());
    if current_backend.ip != new_backend.ip {
        println!(
            "      IP: {} → {}",
            current_backend.ip.red(),
            new_backend.ip.green()
        );
    }
    if current_backend.port != new_backend.port {
        println!(
            "      Port: {} → {}",
            current_backend.port.to_string().red(),
            new_backend.port.to_string().green()
        );
    }

    if current_backend.weight != new_backend.weight {
        println!(
            "      Weight: {} → {}",
            current_backend.weight.unwrap_or(0).to_string().red(),
            new_backend.weight.unwrap_or(0).to_string().green()
        );
    }
}

fn show_path_changes(current: &[PathConfig], new: &[PathConfig]) {
    let current_map: HashMap<_, _> = current.iter().map(|p| (&p.path, p)).collect();
    let new_map: HashMap<_, _> = new.iter().map(|p| (&p.path, p)).collect();

    let all_paths: HashSet<_> = current_map.keys().chain(new_map.keys()).collect();

    if !all_paths.is_empty() {
        println!("\n  {}", "Paths:".bold());

        for &path in all_paths.iter().sorted() {
            match (current_map.get(path), new_map.get(path)) {
                (None, Some(new_path)) => {
                    println!(
                        "    {} {} (strip_prefix: {})",
                        "+".green(),
                        path,
                        new_path.strip_prefix
                    );
                }
                (Some(old_path), None) => {
                    println!(
                        "    {} {} (strip_prefix: {})",
                        "-".red(),
                        path,
                        old_path.strip_prefix
                    );
                }
                (Some(old_path), Some(new_path)) => {
                    if old_path.strip_prefix != new_path.strip_prefix {
                        println!(
                            "    {} (strip_prefix: {} → {})",
                            path,
                            old_path.strip_prefix.to_string().red(),
                            new_path.strip_prefix.to_string().green()
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

fn show_setting_changes<T: std::fmt::Display + PartialEq>(name: &str, current: T, new: T) {
    if current != new {
        println!(
            "  {}: {} → {}",
            name.bold(),
            current.to_string().red(),
            new.to_string().green()
        );
    }
}

fn show_array_changes(name: &str, current: &[String], new: &[String]) {
    let current_set: HashSet<_> = current.iter().collect();
    let new_set: HashSet<_> = new.iter().collect();

    if current_set != new_set {
        println!("\n  {}:", name.bold());

        for item in new_set.difference(&current_set) {
            println!("    {} {}", "+".green(), item);
        }

        for item in current_set.difference(&new_set) {
            println!("    {} {}", "-".red(), item);
        }
    }
}
