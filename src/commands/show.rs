use clap::Args;

use crate::{config::TraefikConfig, error::TraefikResult, etcd::Etcd};

#[derive(Args, Debug)]
pub struct ShowCommand {
    #[arg(short, long)]
    resource: String,
    #[arg(short, long)]
    output: Option<String>,
}

pub async fn run(
    command: &ShowCommand,
    etcd_client: &Etcd,
    _traefik_config: &TraefikConfig,
) -> TraefikResult<()> {
    println!("Show command");
    etcd_client.get(command.resource.as_str()).await?;
    Ok(())
}
