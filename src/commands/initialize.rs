use clap::Args;

use crate::{config::TraefikConfig, error::TraefikResult, etcd::Etcd};

#[derive(Args, Debug)]
pub struct InitializeCommand {}

pub async fn run(
    _command: &InitializeCommand,
    etcd_client: &Etcd,
    traefik_config: &TraefikConfig,
) -> TraefikResult<()> {
    println!("Initialize command");
    traefik_config.initialize(etcd_client).await?;
    Ok(())
}
