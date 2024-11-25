use axum::{
    routing::{get, post},
    Extension, Router,
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::debug;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::error::{TraefikError, TraefikResult};

use super::{
    controllers, db,
    models::{ConfigVersion, DeploymentProtocol},
};
use crate::features::models::SaveConfigRequest;

// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::get_protocols,
        controllers::get_configs,
        controllers::save_config
    ),
    components(
        schemas(
            ConfigVersion,
            DeploymentProtocol,
            ServerConfig,
            SaveConfigRequest
        )
    ),
    tags(
        (name = "configs", description = "Configuration management endpoints"),
        (name = "protocols", description = "Protocol management endpoints")
    )
)]
struct ApiDoc;

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
}

fn default_port() -> u16 {
    9090
}

fn default_host() -> String {
    "localhost".to_string()
}

pub(crate) async fn create_database(config: &ServerConfig) -> TraefikResult<Pool<MySql>> {
    let database_url = config.database_url.clone();
    let pool = db::prepare_database(database_url).await?;
    Ok(pool)
}

pub(crate) async fn create_app(config: &ServerConfig, pool: Pool<MySql>) -> TraefikResult<Router> {
    dotenv().ok();
    debug!("Loading environment variables");

    debug!("Building application");
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/protocols", get(controllers::get_protocols))
        .route("/api/configs", get(controllers::get_configs))
        .route("/api/configs", post(controllers::save_config))
        .layer(Extension(pool))
        .layer(Extension(config.clone()))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
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
    let pool = create_database(&config).await?;
    let app = create_app(&config, pool).await?;
    let listener = create_listener(&config).await?;
    let addr = listener.local_addr().unwrap();
    debug!("Binding to {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::api::test_api_helpers::setup_test_server;
    #[allow(unused_imports)]
    use crate::test_helpers::init_test_tracing;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_run_server() {
        let server = setup_test_server().await.unwrap();
        debug!("Server and database setup");
        let _test_user = server.test_user.clone();

        debug!("Testing get protocols");
        let response = server.get("/api/protocols").await;
        debug!("Response: {:?}", response);
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_protocols() {
        let server = setup_test_server().await.unwrap();
        let response = server.get("/api/protocols").await;
        assert_eq!(response.status(), StatusCode::OK);
        let response_json = response.json().await.unwrap();
        let protocols: Vec<DeploymentProtocol> = serde_json::from_value(response_json).unwrap();
        assert!(!protocols.is_empty());
    }

    // #[tokio::test]
    // async fn test_get_configs() {
    //     let server = setup_test_server().await;
    //     let response = server.get("/api/configs").await;
    //     assert_eq!(response.status_code(), StatusCode::OK);
    //     let configs: Vec<ConfigVersion> = response.json();
    //     assert!(configs.is_empty());
    // }

    // #[tokio::test]
    // async fn test_save_config() {
    //     let server = setup_test_server().await;
    //     let request = SaveConfigRequest {
    //         name: "test".to_string(),
    //         config: json!({
    //             "rulePrefix": "test",
    //             "hosts": [],
    //             "middlewares": {}
    //         }),
    //     };

    //     let response = server.post("/api/configs").json(&request).await;
    //     assert_eq!(response.status_code(), StatusCode::OK);
    //     let config: ConfigVersion = response.json();
    //     assert_eq!(config.name, "test");
    // }
}
