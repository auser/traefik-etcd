use std::{collections::BTreeMap, path::PathBuf};

use clap::{Args, ValueEnum};
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use tracing::debug;

use crate::{
    core::{client::StoreClient, templating::TemplateOr},
    error::TraefikResult,
    features::etcd::Etcd,
    TraefikConfig,
};

#[derive(ValueEnum, Debug, Clone, PartialEq, Eq)]
pub enum Format {
    Yaml,
    Json,
}

#[derive(Args, Debug)]
pub struct ToFileCommand {
    #[arg(long, short = 'o')]
    pub output_file: Option<PathBuf>,
    #[arg(long, short = 'F', default_value = "yaml")]
    pub format: Format,
    #[arg(long, short, default_value = "traefik")]
    pub prefix: String,
}

pub async fn run(
    command: &ToFileCommand,
    client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let tree = build_tree(client, &command.prefix, traefik_config).await?;
    let res = match command.format {
        Format::Yaml => {
            let yaml = serde_yaml::to_string(&tree)?;
            debug!("{}", yaml);
            yaml
        }
        Format::Json => {
            let json = serde_json::to_string(&tree)?;
            debug!("{}", json);
            json
        }
    };

    if let Some(output_file) = &command.output_file {
        std::fs::write(output_file, res)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Node {
    value: Option<String>,
    children: BTreeMap<String, Node>,
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(value) = &self.value {
            if !value.is_empty() {
                return serializer.serialize_str(value);
            }
        }

        let mut map = serializer.serialize_map(None)?;

        for (k, v) in &self.children {
            // Check if all child keys are numeric
            let is_sequence = v.children.keys().all(|key| key.parse::<usize>().is_ok());

            if is_sequence && !v.children.is_empty() {
                let values: Vec<_> = v.children.values().collect();
                map.serialize_entry(k, &values)?;
            } else if k.parse::<usize>().is_ok() {
                continue;
            } else {
                map.serialize_entry(k, v)?;
            }
        }

        map.end()
    }
}

async fn build_tree(
    client: &StoreClient<Etcd>,
    prefix: &str,
    config: &TraefikConfig,
) -> TraefikResult<Node> {
    let mut resolver = config.resolver()?;
    let mut context = config.context()?;

    let resp = client.get_with_prefix(prefix).await?;

    let mut root = Node {
        value: None,
        children: BTreeMap::new(),
    };

    for kv in resp.iter() {
        let key = String::from_utf8(kv.key.clone()).expect("Failed to convert key to string");
        let value = String::from_utf8(kv.value.clone()).expect("Failed to convert value to string");
        let path: Vec<&str> = key
            .trim_start_matches(prefix)
            .trim_start_matches('/')
            .split('/')
            .collect();

        let mut current = &mut root;
        for (i, segment) in path.iter().enumerate() {
            if i == path.len() - 1 {
                if value.is_empty() {
                    current.children.insert(
                        segment.to_string(),
                        Node {
                            value: None,
                            children: BTreeMap::new(),
                        },
                    );
                } else {
                    let template_value = if value.contains("{{") {
                        TemplateOr::Template(value.clone())
                    } else {
                        TemplateOr::Static(value.clone())
                    };

                    let resolved_value = template_value.resolve(&mut resolver, &mut context)?;

                    current.children.insert(
                        segment.to_string(),
                        Node {
                            value: Some(resolved_value),
                            children: BTreeMap::new(),
                        },
                    );
                }
            } else {
                current = current.children.entry(segment.to_string()).or_insert(Node {
                    value: None,
                    children: BTreeMap::new(),
                });
            }
        }
    }

    // Clean up the tree by removing the root node if it's just a wrapper
    let cloned_root = root.clone();
    if cloned_root.value.is_none() && cloned_root.children.len() == 1 {
        if let Some((_, child)) = cloned_root.children.into_iter().next() {
            return Ok(child);
        }
    }

    Ok(root)
}
