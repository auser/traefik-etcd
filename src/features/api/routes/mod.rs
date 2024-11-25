use std::sync::Arc;

use axum::http::StatusCode;
use axum::Router;
use sqlx::{MySql, Pool};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};

use super::ServerConfig;

#[derive(Clone)]
pub(crate) struct ApiContext {
    config: Arc<ServerConfig>,
    db: Pool<MySql>,
}

pub mod api;
pub mod auth;
pub mod extractor;
pub mod frontend;
pub mod session;

pub async fn get_routes(config: ServerConfig, db: Pool<MySql>) -> Router {
    api_router()
        .layer(
            ServiceBuilder::new().layer(AddExtensionLayer::new(ApiContext {
                config: Arc::new(config),
                db,
            })),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

fn api_router() -> Router {
    Router::new()
        .merge(frontend::router())
        .merge(api::routes())
        .merge(auth::routes())
        .merge(session::routes())
}

#[allow(clippy::unused_async)]
pub(crate) async fn handle_error() -> (StatusCode, &'static str) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong accessing static files...",
    )
}
