use clap::Args;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Yaml,
    Json,
}

#[derive(Args, Debug)]
pub struct RenderCommand {
    #[arg(short = 'F', long, default_value = "yaml")]
    pub format: Format,
}

pub async fn run(
    command: &RenderCommand,
    _client: &StoreClient<Etcd>,
    traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let rendered = match command.format {
        Format::Yaml => serde_yaml::to_string(&traefik_config)?,
        Format::Json => serde_json::to_string(&traefik_config)?,
    };

    println!("{}", rendered);
    Ok(())
}
