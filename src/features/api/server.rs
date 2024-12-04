use axum::Router;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};
use tokio::net::TcpListener;
use tracing::debug;
use utoipa::ToSchema;

use crate::{
    error::{TraefikError, TraefikResult},
    features::routes,
};

use super::db;

static CONFIG_BASE_PATH: &str = "/etc/traefik/configs";

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerConfig {
    #[schema(default = "localhost")]
    #[serde(default = "default_host")]
    pub host: String,
    #[schema(default = 9090)]
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_url: Option<String>,
    #[schema(default = "default_hmac_key")]
    #[serde(default = "default_hmac_key")]
    pub hmac_key: String,
    #[schema(default = "./configs")]
    #[serde(default = "default_base_config_path")]
    pub base_config_path: String,
}

fn default_hmac_key() -> String {
    "default_hmac_key".to_string()
}

fn default_port() -> u16 {
    9090
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_base_config_path() -> String {
    if std::env::var("CONFIG_BASE_PATH").is_ok() {
        std::env::var("CONFIG_BASE_PATH").unwrap()
    } else {
        CONFIG_BASE_PATH.to_string()
    }
}

#[cfg(feature = "cli")]
impl From<crate::cli::serve::ServeCommand> for ServerConfig {
    fn from(command: crate::cli::serve::ServeCommand) -> Self {
        ServerConfig {
            host: command.host,
            port: command.port,
            database_url: Some(command.database_url),
            hmac_key: command.hmac_key,
            base_config_path: command.base_config_path,
        }
    }
}

pub(crate) async fn create_database(config: &ServerConfig) -> TraefikResult<Pool<MySql>> {
    let database_url = config.database_url.clone();
    let pool = db::prepare_database(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub(crate) async fn create_app(config: &ServerConfig, pool: Pool<MySql>) -> TraefikResult<Router> {
    dotenv().ok();
    debug!("Loading environment variables");

    debug!("Building application");
    let app = routes::get_routes(config.clone(), pool).await;
    Ok(app)
}

pub(crate) async fn create_listener(config: &ServerConfig) -> TraefikResult<TcpListener> {
    let host = config.host.clone();
    let port = config.port;
    debug!("Running server on {}:{}", host, port);
    let addr = SocketAddr::from((Ipv4Addr::from_str(&host).unwrap(), port));
    TcpListener::bind(addr).await.map_err(TraefikError::from)
}

pub async fn run(config: ServerConfig) -> TraefikResult<()> {
    debug!("Running server on {:#?}", config);
    let pool = create_database(&config).await?;
    let app = create_app(&config, pool).await?;
    let listener = create_listener(&config).await?;
    let addr = listener.local_addr().unwrap();
    debug!("Binding to {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
