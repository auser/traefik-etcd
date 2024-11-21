use clap::Args;
use tracing::debug;

use crate::{
    core::client::{StoreClient, StoreClientActor},
    error::TraefikResult,
    features::etcd::Etcd,
    TraefikConfig,
};

#[derive(Args, Debug)]
pub struct GetCommand {
    #[arg(index(1))]
    name: String,

    #[arg(short, long)]
    keys: bool,
}

pub async fn run(
    command: &GetCommand,
    client: &StoreClient<Etcd>,
    _traefik_config: &TraefikConfig,
) -> TraefikResult<()> {
    let key = format!("{}", command.name);
    let values: Vec<_> = if command.keys {
        client.actor.get_keys(key).await?
    } else {
        client.actor.get_with_prefix(key).await?
    };

    debug!("values: {:?}", values);

    let values = values
        .iter()
        .map(|v| v.value_str().unwrap_or_default())
        .collect::<Vec<_>>();
    let value_str = values.join("\n");
    println!("{}", value_str);

    Ok(())
}
