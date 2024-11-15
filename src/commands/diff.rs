use crate::{config::TraefikConfig, error::TraefikResult, etcd::Etcd};
use clap::Args;

#[derive(Args, Debug)]
pub struct DiffCommand {
    #[arg(short, long)]
    all: bool,

    #[arg(short, long)]
    dry_run: bool,
}

pub async fn run(
    command: &DiffCommand,
    etcd_client: &mut Etcd,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    if command.dry_run {
        traefik_config.diff(etcd_client).await?;
    } else {
        traefik_config
            .apply_with_diff(etcd_client, command.dry_run)
            .await?;
    }
    Ok(())
}
