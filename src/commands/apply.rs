use clap::Args;

use crate::{
    common::{error::TraefikResult, etcd::Etcd},
    config::TraefikConfig,
};

#[derive(Args, Debug)]
pub struct ApplyCommand {
    #[arg(short, long)]
    dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    clean: bool,
}

pub async fn run(
    command: &ApplyCommand,
    etcd_client: &mut Etcd,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    if command.clean {
        traefik_config.clean_etcd(etcd_client).await?;
    }
    traefik_config
        .apply_to_etcd(etcd_client, command.dry_run)
        .await?;

    Ok(())
}
