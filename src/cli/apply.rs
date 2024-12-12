use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct ApplyCommand {
    #[arg(short, long)]
    dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    clean: bool,

    #[arg(short, long, default_value_t = false)]
    all: bool,

    #[arg(short, long, default_value_t = false)]
    rules: bool,
}

pub async fn run(
    command: &ApplyCommand,
    client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    if command.clean && !command.dry_run {
        traefik_config.clean_etcd(client, command.all).await?;
    }
    traefik_config
        .apply_to_etcd(client, command.dry_run, command.rules, command.clean)
        .await?;

    Ok(())
}
