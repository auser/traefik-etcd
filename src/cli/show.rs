use clap::Args;
use tracing::debug;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct ShowCommand {
    #[arg(short, long)]
    resource: String,
    #[arg(short, long)]
    output: Option<String>,
}

pub async fn run(
    command: &ShowCommand,
    client: &StoreClient<Etcd>,
    _traefik_config: &TraefikConfig,
) -> TraefikResult<()> {
    debug!("Show command");
    let res = client.get_with_prefix(command.resource.as_str()).await?;

    for kv in res {
        println!("{:?}", kv.value_json::<serde_json::Value>()?);
    }
    Ok(())
}
