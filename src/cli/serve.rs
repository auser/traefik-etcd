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
    pub host: String,
    #[arg(short, long, default_value = "9090")]
    pub port: u16,
    #[arg(long, env, default_value = "default_hmac_key")]
    pub hmac_key: String,
    #[arg(long, env, default_value = "default_database_url")]
    pub database_url: String,
    #[arg(long, env, default_value = "./frontend/templates")]
    pub base_config_path: String,
}

pub async fn run(
    command: &ServeCommand,
    _client: &StoreClient<Etcd>,
    _traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let server_config = ServerConfig {
        host: command.host.clone(),
        port: command.port,
        database_url: Some(command.database_url.clone()),
        hmac_key: command.hmac_key.clone(),
        base_templates_path: command.base_config_path.clone(),
    };

    crate::features::api::run(server_config).await?;

    Ok(())
}
