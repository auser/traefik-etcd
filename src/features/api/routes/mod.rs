use std::sync::Arc;

use axum::{http::Method, Router};
use sqlx::{MySql, Pool};
use tower::ServiceBuilder;
use tower_http::{
    add_extension::AddExtensionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use super::{file_loader::FileConfig, ServerConfig};

#[derive(Clone)]
pub(crate) struct ApiContext {
    config: Arc<ServerConfig>,
    db: Pool<MySql>,
    file_config: FileConfig,
}

pub mod api;
pub mod auth;
pub mod extractor;
pub mod frontend;
pub mod session;

pub async fn get_routes(config: ServerConfig, db: Pool<MySql>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .expose_headers(Any);

    api_router()
        .layer(
            ServiceBuilder::new().layer(AddExtensionLayer::new(ApiContext {
                config: Arc::new(config),
                db,
                file_config: FileConfig::default(),
            })),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

fn api_router() -> Router {
    Router::new()
        .merge(api::routes())
        .merge(auth::routes())
        .merge(session::routes())
        .merge(frontend::router())
}

// #[allow(clippy::unused_async)]
// pub(crate) async fn handle_error() -> (StatusCode, &'static str) {
//     (
//         StatusCode::INTERNAL_SERVER_ERROR,
//         "Something went wrong accessing static files...",
//     )
// }
