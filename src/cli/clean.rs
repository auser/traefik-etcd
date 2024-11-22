use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct CleanCommand {
    #[arg(short, long)]
    all: bool,
}

pub async fn run(
    command: &CleanCommand,
    client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    traefik_config.clean_etcd(client, command.all).await?;
    Ok(())
}
