use std::path::PathBuf;

use clap::Args;

use crate::{
    core::{client::StoreClient, etcd_trait::EtcdPair},
    error::TraefikResult,
    features::etcd::Etcd,
    TraefikConfig,
};

#[derive(Args, Debug)]
pub struct LoadCommand {
    #[arg(long, short)]
    from_file: Option<PathBuf>,
}

pub async fn run(
    command: &LoadCommand,
    _client: &StoreClient<Etcd>,
    _traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let pairs = EtcdPair::from_file(command.from_file.as_ref().unwrap())?;
    let config = TraefikConfig::parse_etcd_to_traefik_config(pairs)?;
    println!("Loaded config: {:#?}", config);
    Ok(())
}
