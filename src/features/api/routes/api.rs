use axum::{response::IntoResponse, Json, Router};
use serde_json::json;

pub mod configs;
pub mod templates;

pub use configs::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::api;
use crate::{
    config::traefik_config::{ConfigVersionHistory, TraefikConfigVersion},
    features::{models::SaveConfigRequest, ServerConfig, TemplateInfo},
    TraefikConfig,
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
        api::configs::get_config_history,
        api::configs::create_config_backup,
        // Templates
        api::templates::list_templates,
        api::templates::get_template_route,
        api::templates::delete_template,
        api::templates::search_templates,
    ),
    components(
        schemas(
            ServerConfig,
            SaveConfigRequest,
            TraefikConfigVersion,
            ConfigVersionHistory,
            TemplateInfo,
            TraefikConfig,
        )
    ),
    tags(
        (name = "config", description = "Configuration management endpoints"),
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
        .merge(configs::routes())
        .merge(templates::routes())
}
