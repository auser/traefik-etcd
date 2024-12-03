use clap::Args;

use crate::core::etcd_trait::ToEtcdPairs;
use crate::{
    core::client::StoreClient,
    error::TraefikResult,
    features::etcd::{compare_etcd_configs, Etcd},
    TraefikConfig,
};

#[derive(Args, Debug)]
pub struct DiffCommand {
    #[arg(short, long)]
    detailed: bool,
}

pub async fn run(
    command: &DiffCommand,
    client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    // Get the new configuration pairs
    let pairs = traefik_config.to_etcd_pairs(&traefik_config.rule_prefix)?;

    // If diff flag is set, show the differences
    let diff = compare_etcd_configs(client, pairs.clone(), &traefik_config.rule_prefix).await?;
    diff.display(command.detailed);

    Ok(())
}
