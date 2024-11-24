use axum::{
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::debug;

use crate::{error::TraefikResult, features::db::prepare_database, TraefikConfig};

use crate::config::host::HostConfig;

// openAPI doc
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::controllers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_url: Option<String>,
}

fn default_port() -> u16 {
    9090
}

fn default_host() -> String {
    "localhost".to_string()
}

pub async fn run(config: ServerConfig) -> TraefikResult<()> {
    debug!("Setting up API server");
    #[derive(OpenApi, Debug)]
    #[openapi(
        paths(
            controllers::deployments::all_deployments,
            controllers::selection::new_selection,
            controllers::selection::get_selection,
            controllers::health_check::new_health_check,
            controllers::health_check::get_health_check,
        ),
        components(
            schemas(HostConfig)
        ),
        tags(
            (name = "host", description = "Hosts management API"),
            (name = "selection", description = "Selection management API"),
            (name = "health_check", description = "Health check management API"),
        )
    )]
    struct ApiDoc;

    debug!("Preparing database");
    let pool = prepare_database(config.database_url).await?;

    let traefik_config = TraefikConfig::default();

    debug!("Building application");
    // build our application with a route
    let app = Router::new()
        // openAPI doc under: http://127.0.0.1:3000/swagger-ui
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .route(
            "/deployments",
            get(controllers::deployments::all_deployments),
        )
        .route(
            "/deployments",
            post(controllers::deployments::new_deployment),
        )
        .route("/selection", post(controllers::selection::new_selection))
        .route("/selection/:id", get(controllers::selection::get_selection))
        .route(
            "/health_check",
            post(controllers::health_check::new_health_check),
        )
        .route(
            "/health_check/:id",
            get(controllers::health_check::get_health_check),
        )
        .layer(Extension(pool))
        .layer(Extension(traefik_config))
        .layer(TraceLayer::new_for_http());

    debug!("Running server on {}:{}", config.host, config.port);
    let addr = SocketAddr::from((Ipv4Addr::from_str(&config.host).unwrap(), config.port));
    debug!("Binding to {}", addr);
    let listener = TcpListener::bind(addr).await?;
    debug!("Listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn root() -> &'static str {
    "OK"
}
