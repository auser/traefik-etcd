use clap::Args;

use crate::{
    core::client::StoreClient,
    error::TraefikResult,
    features::{api::ServerConfig, etcd::Etcd},
    TraefikConfig,
};

#[derive(Args, Debug)]
pub struct ServeCommand {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,
    #[arg(short, long, default_value = "9090")]
    port: u16,
}

pub async fn run(
    command: &ServeCommand,
    _client: &StoreClient<Etcd>,
    _traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let server_config = ServerConfig {
        host: command.host.clone(),
        port: command.port,
        database_url: None,
    };

    crate::features::api::run(server_config).await?;

    Ok(())
}
