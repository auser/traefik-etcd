use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::{info, instrument};

use crate::{
    config::traefik_config::TraefikConfig,
    core::client::StoreClient,
    error::TraefikResult,
    features::etcd::Etcd,
    tracing::{init_tracing, LogConfig},
    NAME,
};

mod get;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None, name = NAME)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, default_value = "info", global = true)]
    pub log_level: String,

    #[arg(long, global = true, default_value = "/etc/traefikctl/traefikctl.yaml")]
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Get specific key or prefix
    Get(get::GetCommand),
}

#[instrument]
pub async fn run() -> TraefikResult<()> {
    color_eyre::install()?;
    let cli: Cli = Cli::parse();
    let log_config = LogConfig {
        max_level: cli.log_level,
        filter: format!("{}=info", NAME),
        rolling_file_path: None,
    };
    init_tracing(NAME, &log_config)?;

    info!("Reading config file: {:?}", &cli.config_file);
    let config_file = cli.config_file.unwrap_or_default();

    let config = std::fs::read_to_string(&config_file)?;
    let mut traefik_config: TraefikConfig = serde_yaml::from_str(&config)?;
    let etcd_client = Etcd::new(&traefik_config.etcd).await?;

    #[cfg(feature = "etcd")]
    let client = StoreClient::new(etcd_client);

    match cli.command {
        Commands::Get(get_command) => {
            get::run(&get_command, &client, &mut traefik_config).await?;
        }
    }

    Ok(())
}
