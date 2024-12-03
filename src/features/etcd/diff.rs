use crate::core::client::StoreClient;
use crate::core::etcd_trait::EtcdPair;
use crate::error::TraefikResult;
use crate::features::etcd::Etcd;
use colored::Colorize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct EtcdDiff {
    pub added: Vec<EtcdPair>,
    pub removed: Vec<EtcdPair>,
    pub modified: Vec<(EtcdPair, EtcdPair)>, // (old, new)
    pub unchanged: Vec<EtcdPair>,
}

impl EtcdDiff {
    pub fn display(&self, detailed: bool) {
        if detailed {
            println!("\nDetailed Configuration Changes:");
            println!("==============================\n");
        }

        // Sort all entries by key for consistent display
        let mut added = self.added.clone();
        let mut removed = self.removed.clone();
        let mut modified = self.modified.clone();
        let mut unchanged = self.unchanged.clone();

        added.sort_by(|a, b| a.key().cmp(b.key()));
        removed.sort_by(|a, b| a.key().cmp(b.key()));
        modified.sort_by(|a, b| a.0.key().cmp(b.0.key()));
        unchanged.sort_by(|a, b| a.key().cmp(b.key()));

        // Print all entries in lexicographical order
        if detailed {
            println!("Added Entries:");
            println!("-------------");
            if added.is_empty() {
                println!("No new entries added");
            } else {
                for pair in &added {
                    println!("{} {} = {}", "+".green(), pair.key(), pair.value());
                }
            }
            println!();
        } else {
            // Print additions
            if !self.added.is_empty() {
                println!("\nAdditions:");
                for pair in &self.added {
                    println!("{} {} = {}", "+".green(), pair.key(), pair.value());
                }
            }
        }

        if detailed {
            println!("Removed Entries:");
            println!("---------------");
            if removed.is_empty() {
                println!("No entries removed");
            } else {
                for pair in &removed {
                    println!("{} {} = {}", "-".red(), pair.key(), pair.value());
                }
            }
            println!();
        } else {
            // Print removals
            if !self.removed.is_empty() {
                println!("\nRemovals:");
                for pair in &self.removed {
                    println!("{} {} = {}", "-".red(), pair.key(), pair.value());
                }
            }
        }

        if detailed {
            println!("Modified Entries:");
            println!("----------------");
            if modified.is_empty() {
                println!("No entries modified");
            } else {
                for (old, new) in &modified {
                    println!("{} {} ", "M".yellow(), old.key());
                    println!("  - Old: {}", old.value());
                    println!("  + New: {}", new.value());
                }
            }
        } else {
            // Print modifications
            if !self.modified.is_empty() {
                println!("\nModifications:");
                for (old, new) in &self.modified {
                    println!("{} {} = {}", "M".yellow(), old.key(), old.value());
                    println!("  → {}", new.value());
                }
            }
        }

        if detailed {
            println!("Unchanged Entries:");
            println!("----------------");
            if unchanged.is_empty() {
                println!("No unchanged entries");
            } else {
                for pair in &unchanged {
                    println!("{} {} = {}", "=".bright_black(), pair.key(), pair.value());
                }
            }
        }

        // Print summary
        println!("Summary:");
        println!("--------");
        println!("  Added entries:     {}", added.len());
        println!("  Removed entries:   {}", removed.len());
        println!("  Modified entries:  {}", modified.len());
        println!("  Unchanged entries: {}", unchanged.len());
        println!(
            "  Total entries:     {}",
            added.len() + removed.len() + modified.len() + unchanged.len()
        );
    }
}

pub async fn compare_etcd_configs(
    client: &StoreClient<Etcd>,
    new_pairs: Vec<EtcdPair>,
    base_key: &str,
) -> TraefikResult<EtcdDiff> {
    // Get current configuration from etcd
    let current_pairs = client.get_with_prefix(base_key).await?;

    // Convert current pairs to EtcdPair format
    let current_pairs: Vec<EtcdPair> = current_pairs
        .into_iter()
        .map(|kv| {
            EtcdPair::new(
                String::from_utf8_lossy(kv.key.as_slice()).to_string(),
                String::from_utf8_lossy(kv.value.as_slice()).to_string(),
            )
        })
        .collect();

    // Create maps for easier comparison
    let current_map: HashMap<_, _> = current_pairs.iter().map(|p| (p.key(), p.value())).collect();

    let new_map: HashMap<_, _> = new_pairs.iter().map(|p| (p.key(), p.value())).collect();

    // Get all keys
    let current_keys: HashSet<_> = current_map.keys().collect();
    let new_keys: HashSet<_> = new_map.keys().collect();

    let mut diff = EtcdDiff::default();

    // Find additions
    for key in new_keys.difference(&current_keys) {
        if let Some(value) = new_map.get(&**key) {
            diff.added
                .push(EtcdPair::new((*key).to_string(), (*value).to_string()));
        }
    }

    // Find removals
    for key in current_keys.difference(&new_keys) {
        if let Some(value) = current_map.get(&**key) {
            diff.removed
                .push(EtcdPair::new((*key).to_string(), (*value).to_string()));
        }
    }

    // Find modifications and unchanged
    for key in new_keys.intersection(&current_keys) {
        let current_value = current_map.get(&**key).unwrap();
        let new_value = new_map.get(&**key).unwrap();

        if current_value != new_value {
            diff.modified.push((
                EtcdPair::new((*key).to_string(), (*current_value).to_string()),
                EtcdPair::new((*key).to_string(), (*new_value).to_string()),
            ));
        } else {
            diff.unchanged.push(EtcdPair::new(
                (*key).to_string(),
                (*current_value).to_string(),
            ));
        }
    }

    Ok(diff)
}
