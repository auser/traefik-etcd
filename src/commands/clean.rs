use crate::{config::TraefikConfig, error::TraefikResult, etcd::Etcd};
use clap::Args;

#[derive(Args, Debug)]
pub struct CleanCommand {
    #[arg(short, long)]
    all: bool,
}

pub async fn run(
    command: &CleanCommand,
    etcd_client: &mut Etcd,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    traefik_config.clean_etcd(etcd_client, command.all).await?;
    Ok(())
}
