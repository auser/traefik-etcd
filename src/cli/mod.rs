use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::{debug, instrument};

use crate::{
    config::traefik_config::TraefikConfig,
    core::client::StoreClient,
    error::TraefikResult,
    features::etcd::Etcd,
    tracing::{init_tracing, LogConfig},
    NAME,
};

mod apply;
mod clean;
mod codegen;
#[cfg(feature = "etcd")]
mod diff;
mod generate;
mod get;
#[cfg(feature = "api")]
pub(crate) mod serve;
mod show;
mod validate;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None, name = NAME)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, short = 'l', default_value = "info", global = true)]
    pub log_level: String,

    #[arg(
        long,
        short = 'f',
        global = true,
        default_value = "/etc/traefikctl/traefikctl.yaml"
    )]
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Get specific key or prefix
    Get(get::GetCommand),
    /// Show the current traefik configuration
    Show(show::ShowCommand),
    /// Apply the current traefik configuration
    Apply(apply::ApplyCommand),
    /// Clean the current traefik configuration
    Clean(clean::CleanCommand),
    /// Validate the current traefik configuration
    Validate,
    /// Generate a starter traefik configuration
    Generate(generate::GenerateCommand),
    #[cfg(feature = "api")]
    /// Serve the API
    Serve(serve::ServeCommand),
    /// Generate the typescript types
    Codegen(codegen::CodegenCommand),
    /// Diff the current traefik configuration
    Diff(diff::DiffCommand),
}

#[instrument]
pub async fn run() -> TraefikResult<()> {
    color_eyre::install()?;
    let cli: Cli = Cli::parse();
    let log_level = cli.log_level.clone();
    let log_config = LogConfig {
        max_level: log_level.clone(),
        filter: format!("{}={}", NAME, &log_level),
        rolling_file_path: None,
    };
    init_tracing(NAME, &log_config)?;

    debug!("Reading config file: {:?}", &cli.config_file);
    let config_file = cli.config_file.unwrap_or_default();

    let config = std::fs::read_to_string(&config_file).unwrap_or_default();
    let mut traefik_config: TraefikConfig = serde_yaml::from_str(&config).unwrap_or_default();
    let etcd_client = Etcd::new(&traefik_config.etcd).await?;

    #[cfg(feature = "etcd")]
    let client = StoreClient::new(etcd_client);

    match cli.command {
        Commands::Get(get_command) => {
            get::run(&get_command, &client, &traefik_config).await?;
        }
        Commands::Show(show_command) => {
            show::run(&show_command, &client, &traefik_config).await?;
        }
        Commands::Apply(apply_command) => {
            apply::run(&apply_command, &client, &mut traefik_config).await?;
        }
        Commands::Clean(clean_command) => {
            clean::run(&clean_command, &client, &mut traefik_config).await?;
        }
        Commands::Validate => {
            validate::run(&client, &mut traefik_config).await?;
        }
        Commands::Generate(generate_command) => {
            generate::run(&generate_command, &client, &mut traefik_config).await?;
        }
        #[cfg(feature = "api")]
        Commands::Serve(serve_command) => {
            serve::run(&serve_command, &client, &mut traefik_config).await?;
        }
        Commands::Codegen(codegen_command) => {
            codegen::run(&codegen_command, &client, &mut traefik_config).await?;
        }
        #[cfg(feature = "etcd")]
        Commands::Diff(diff_command) => {
            diff::run(&diff_command, &client, &mut traefik_config).await?;
        }
    }

    Ok(())
}
