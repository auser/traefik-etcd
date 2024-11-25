use axum::{response::IntoResponse, Json, Router};
use serde_json::json;

pub mod configs;
pub mod protocols;

pub use configs::*;
pub use protocols::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::api;
use crate::features::{
    models::{ConfigVersion, DeploymentProtocol, SaveConfigRequest},
    ServerConfig,
};

/// imitating an API response
#[allow(clippy::unused_async)]
pub async fn handler() -> impl IntoResponse {
    tracing::info!("Seeking api data");
    Json(
        json!({"result": "ok", "message": "You've reached the backend API by using a valid token."}),
    )
}

// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        api::protocols::get_protocols,
        api::configs::get_configs,
        api::configs::save_config
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
pub struct ApiDoc;

pub fn routes() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_router())
}

fn api_router() -> Router {
    Router::new()
        .merge(protocols::routes())
        .merge(configs::routes())
}
