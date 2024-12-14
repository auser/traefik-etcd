use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct CleanCommand {}

pub async fn run(
    _command: &CleanCommand,
    client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    traefik_config.clean_etcd(client).await?;
    Ok(())
}
