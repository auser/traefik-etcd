use clap::{arg, Parser, Subcommand};
use tracing::instrument;

use crate::{
    common::{
        etcd::Etcd,
        log::{init_tracing, LogConfig},
    },
    config::TraefikConfig,
};
pub(crate) mod apply;
pub(crate) mod get;
pub(crate) mod show;
pub(crate) mod validate;

#[derive(Parser)]
#[command(name = "traefik-ctl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, short = 'c')]
    pub config: String,

    #[arg(long, short = 'l')]
    pub log_level: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Apply configuration
    Apply(apply::ApplyCommand),
    /// Show current configuration
    Show(show::ShowCommand),
    /// Validate configuration
    Validate,
    /// Get specific configuration
    Get(get::GetCommand),
}

#[instrument]
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let log_config = LogConfig {
        max_level: cli.log_level.clone().unwrap_or("info".to_owned()),
        filter: cli.log_level.clone().unwrap_or("info".to_owned()),
        ..Default::default()
    };
    init_tracing("traefik-ctl", &log_config)?;

    let config = std::fs::read_to_string(&cli.config)?;
    let mut traefik_config: TraefikConfig = serde_yaml::from_str(&config)?;
    let mut etcd_client = Etcd::new(&traefik_config.etcd).await?;

    match cli.command {
        Commands::Apply(apply_command) => {
            apply::run(&apply_command, &mut etcd_client, &mut traefik_config).await?;
        }
        Commands::Show(show_command) => {
            show::run(&show_command, &mut etcd_client, &mut traefik_config).await?;
        }
        Commands::Validate => {
            validate::run(&mut etcd_client, &mut traefik_config).await?;
        }
        Commands::Get(get_command) => {
            get::run(&get_command, &mut etcd_client, &mut traefik_config).await?;
        }
    }

    Ok(())
}
