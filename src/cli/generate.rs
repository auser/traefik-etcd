use std::path::PathBuf;

use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct GenerateCommand {
    #[arg(short, long)]
    all: bool,
    #[arg(short, long, default_value = "traefik_config.yaml")]
    output: PathBuf,
    #[arg(short, long, required = false)]
    domain: Option<String>,
}

pub async fn run(
    command: &GenerateCommand,
    _client: &StoreClient<Etcd>,
    _traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let config = TraefikConfig::generate_config(command.domain.clone());
    let serialized = serde_yaml::to_string(&config)?;
    std::fs::write(command.output.clone(), serialized)?;

    Ok(())
}
