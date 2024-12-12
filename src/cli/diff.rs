use crate::core::etcd_trait::{EtcdPair, ToEtcdPairs};
use crate::{
    core::client::StoreClient,
    error::TraefikResult,
    features::etcd::{Etcd, EtcdDiff},
    TraefikConfig,
};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DiffCommand {
    #[arg(short, long)]
    detailed: bool,

    #[arg(short = 'F', long)]
    from_file: Option<PathBuf>,
}

pub async fn run(
    command: &DiffCommand,
    client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    // Get the pairs from our current config
    let mut resolver = traefik_config.resolver()?;
    let context = traefik_config.context()?;
    let current_pairs =
        traefik_config.to_etcd_pairs(&traefik_config.rule_prefix, &mut resolver, &context)?;

    // Get the pairs to compare against (either from etcd or file)
    let comparison_pairs = if let Some(file_path) = &command.from_file {
        // Load and parse the file-based config
        let file_pairs = EtcdPair::from_file(file_path)?;
        let file_config = TraefikConfig::parse_etcd_to_traefik_config(file_pairs)?;
        let mut file_resolver = file_config.resolver()?;
        file_config.to_etcd_pairs(&traefik_config.rule_prefix, &mut file_resolver, &context)?
    } else {
        // Get pairs from live etcd
        let live_pairs = client
            .get_with_prefix(traefik_config.rule_prefix.as_bytes())
            .await?;
        live_pairs
            .into_iter()
            .map(|kv| {
                EtcdPair::new(
                    String::from_utf8_lossy(&kv.key).to_string(),
                    String::from_utf8_lossy(&kv.value).to_string(),
                )
            })
            .collect()
    };

    // Create diff
    let diff = EtcdDiff::create(&current_pairs, &comparison_pairs);
    diff.display(command.detailed);

    Ok(())
}
