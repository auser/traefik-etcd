use std::collections::HashMap;

use crate::{error::TraefikResult, etcd::Etcd};

#[derive(Debug, Clone, PartialEq)]
struct EtcdDiff {
    to_add: Vec<EtcdPair>,
    to_remove: Vec<String>, // keys to remove
    to_update: Vec<EtcdPair>,
}

impl TraefikConfig {
    pub async fn show_diff(&self, etcd: &mut Etcd) -> TraefikResult<()> {
        // Get current configuration from etcd
        let current_pairs = self.get_current_config(etcd).await?;

        // Generate new configuration
        let new_pairs = self.to_etcd_pairs("traefik/http")?;

        // Extract and show rules diff first
        println!("=== Rules Changes ===");
        let current_rules: HashMap<_, _> = current_pairs
            .iter()
            .filter(|p| p.key().contains("/rule"))
            .map(|p| (p.key(), p.value()))
            .collect();

        let new_rules: HashMap<_, _> = new_pairs
            .iter()
            .filter(|p| p.key().contains("/rule"))
            .map(|p| (p.key(), p.value()))
            .collect();

        // Show rule changes
        let mut has_rule_changes = false;
        for (key, new_value) in new_rules.iter() {
            match current_rules.get(key) {
                Some(current_value) if current_value != new_value => {
                    has_rule_changes = true;
                    println!("~ Rule changed for {}", key);
                    println!("  - {}", current_value);
                    println!("  + {}", new_value);
                }
                None => {
                    has_rule_changes = true;
                    println!("+ New rule: {} = {}", key, new_value);
                }
                _ => {}
            }
        }

        // Show removed rules
        for key in current_rules.keys() {
            if !new_rules.contains_key(key) {
                has_rule_changes = true;
                println!("- Removed rule: {} = {}", key, current_rules[key]);
            }
        }

        if !has_rule_changes {
            println!("No rule changes detected");
        }

        // Show service changes
        println!("\n=== Service Changes ===");
        let mut has_service_changes = false;
        let current_services: HashMap<_, _> = current_pairs
            .iter()
            .filter(|p| p.key().contains("/services/"))
            .map(|p| (p.key(), p.value()))
            .collect();

        let new_services: HashMap<_, _> = new_pairs
            .iter()
            .filter(|p| p.key().contains("/services/"))
            .map(|p| (p.key(), p.value()))
            .collect();

        for (key, new_value) in new_services.iter() {
            match current_services.get(key) {
                Some(current_value) if current_value != new_value => {
                    has_service_changes = true;
                    println!("~ Service changed: {}", key);
                    println!("  - {}", current_value);
                    println!("  + {}", new_value);
                }
                None => {
                    has_service_changes = true;
                    println!("+ New service: {} = {}", key, new_value);
                }
                _ => {}
            }
        }

        for key in current_services.keys() {
            if !new_services.contains_key(key) {
                has_service_changes = true;
                println!("- Removed service: {} = {}", key, current_services[key]);
            }
        }

        if !has_service_changes {
            println!("No service changes detected");
        }

        // Show middleware changes
        println!("\n=== Middleware Changes ===");
        let mut has_middleware_changes = false;
        let current_middlewares: HashMap<_, _> = current_pairs
            .iter()
            .filter(|p| p.key().contains("/middlewares/"))
            .map(|p| (p.key(), p.value()))
            .collect();

        let new_middlewares: HashMap<_, _> = new_pairs
            .iter()
            .filter(|p| p.key().contains("/middlewares/"))
            .map(|p| (p.key(), p.value()))
            .collect();

        for (key, new_value) in new_middlewares.iter() {
            match current_middlewares.get(key) {
                Some(current_value) if current_value != new_value => {
                    has_middleware_changes = true;
                    println!("~ Middleware changed: {}", key);
                    println!("  - {}", current_value);
                    println!("  + {}", new_value);
                }
                None => {
                    has_middleware_changes = true;
                    println!("+ New middleware: {} = {}", key, new_value);
                }
                _ => {}
            }
        }

        for key in current_middlewares.keys() {
            if !new_middlewares.contains_key(key) {
                has_middleware_changes = true;
                println!(
                    "- Removed middleware: {} = {}",
                    key, current_middlewares[key]
                );
            }
        }

        if !has_middleware_changes {
            println!("No middleware changes detected");
        }

        Ok(())
    }

    pub async fn apply_with_diff(&mut self, etcd: &mut Etcd, dry_run: bool) -> TraefikResult<()> {
        self.validate()?;

        // Get current configuration from etcd
        let current_pairs = self.get_current_config(etcd).await?;

        // Generate new configuration
        let new_pairs = self.to_etcd_pairs("traefik/http")?;

        // Calculate diff
        let diff = self.calculate_diff(&current_pairs, &new_pairs);

        if dry_run {
            println!("Changes that would be applied:");
            println!("Additions:");
            for pair in &diff.to_add {
                println!("  + {} = {}", pair.key(), pair.value());
            }
            println!("Removals:");
            for key in &diff.to_remove {
                println!("  - {}", key);
            }
            println!("Updates:");
            for pair in &diff.to_update {
                println!("  ~ {} = {}", pair.key(), pair.value());
            }
            return Ok(());
        }

        // Apply changes
        self.apply_diff(etcd, diff).await?;

        Ok(())
    }

    async fn get_current_config(&self, etcd: &mut Etcd) -> TraefikResult<Vec<EtcdPair>> {
        // Get all keys under traefik/http
        let result = etcd.get_with_prefix("traefik/http").await?;

        let mut pairs = Vec::new();
        for (key, value) in result {
            pairs.push(EtcdPair::new(key, value));
        }

        Ok(pairs)
    }

    fn calculate_diff(&self, current: &[EtcdPair], new: &[EtcdPair]) -> EtcdDiff {
        let mut current_map: HashMap<String, String> = current
            .iter()
            .map(|p| (p.key().to_string(), p.value().to_string()))
            .collect();

        let new_map: HashMap<String, String> = new
            .iter()
            .map(|p| (p.key().to_string(), p.value().to_string()))
            .collect();

        let mut to_add = Vec::new();
        let mut to_update = Vec::new();
        let mut to_remove = Vec::new();

        // Find additions and updates
        for pair in new {
            let key = pair.key();
            match current_map.get(key) {
                Some(current_value) => {
                    if current_value != pair.value() {
                        to_update.push(pair.clone());
                    }
                }
                None => {
                    to_add.push(pair.clone());
                }
            }
            current_map.remove(key);
        }

        // Remaining keys in current_map need to be removed
        to_remove.extend(current_map.keys().cloned());

        EtcdDiff {
            to_add,
            to_remove,
            to_update,
        }
    }

    async fn apply_diff(&self, etcd: &mut Etcd, diff: EtcdDiff) -> TraefikResult<()> {
        // Process removals first
        for key in diff.to_remove {
            etcd.delete(key).await?;
        }

        // Process additions and updates
        for pair in diff.to_add.into_iter().chain(diff.to_update.into_iter()) {
            etcd.put(pair.key().to_string(), pair.value().to_string(), None)
                .await?;
        }

        Ok(())
    }

    // New method to specifically check rule changes
    pub async fn check_rule_changes(
        &self,
        etcd: &mut Etcd,
    ) -> TraefikResult<Vec<(String, String)>> {
        let current_pairs = self.get_current_config(etcd).await?;
        let new_pairs = self.to_etcd_pairs("traefik/http")?;

        let mut changes = Vec::new();

        // Create maps of router rules
        let current_rules: HashMap<_, _> = current_pairs
            .iter()
            .filter(|p| p.key().contains("/rule"))
            .map(|p| (p.key().to_string(), p.value().to_string()))
            .collect();

        let new_rules: HashMap<_, _> = new_pairs
            .iter()
            .filter(|p| p.key().contains("/rule"))
            .map(|p| (p.key().to_string(), p.value().to_string()))
            .collect();

        // Compare rules
        for (key, new_value) in new_rules {
            match current_rules.get(&key) {
                Some(current_value) if current_value != &new_value => {
                    changes.push((key, new_value));
                }
                None => {
                    changes.push((key, new_value));
                }
                _ => {}
            }
        }

        Ok(changes)
    }
}

// Example usage in the CLI
pub async fn apply_command(
    config: &mut TraefikConfig,
    etcd: &mut Etcd,
    dry_run: bool,
) -> TraefikResult<()> {
    // First check for rule changes
    let rule_changes = config.check_rule_changes(etcd).await?;
    if !rule_changes.is_empty() {
        println!("Rule changes detected:");
        for (key, new_rule) in rule_changes {
            println!("  {} -> {}", key, new_rule);
        }
    }

    // Apply all changes with diff
    config.apply_with_diff(etcd, dry_run).await?;

    Ok(())
}
