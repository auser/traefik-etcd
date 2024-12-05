use axum::{response::IntoResponse, Json, Router};
use serde_json::json;

pub mod configs;
pub mod protocols;
pub mod templates;

pub use configs::*;
pub use protocols::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::api;
use crate::{
    config::traefik_config::TraefikConfigVersion,
    features::{models::SaveConfigRequest, ServerConfig},
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
        api::configs::get_all_configs,
        api::configs::save_config,
        api::configs::get_default_config,
        api::configs::get_file_configs,
        api::configs::get_config_by_id,
        api::configs::update_config,
        api::configs::save_config_version,
        api::configs::delete_config,
        api::templates::list_templates,
        api::templates::get_template,
    ),
    components(
        schemas(
            ServerConfig,
            SaveConfigRequest,
            TraefikConfigVersion,
        )
    ),
    tags(
        (name = "config", description = "Configuration management endpoints"),
        (name = "protocols", description = "Protocol management endpoints"),
        (name = "templates", description = "Template management endpoints")
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
        .merge(templates::routes())
}
