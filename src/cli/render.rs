use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct RenderCommand {}

pub async fn run(
    _command: &RenderCommand,
    _client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let yaml = serde_yaml::to_string(&traefik_config)?;
    println!("{}", yaml);
    Ok(())
}
