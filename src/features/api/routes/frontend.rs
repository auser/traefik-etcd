use super::handle_error;
use axum::handler::HandlerWithoutStateExt;
use axum::Router;
use tower_http::{services::ServeDir, trace::TraceLayer};

const FRONT_PUBLIC_PATH: &str = "frontend/dist";

pub fn router() -> Router {
    Router::new()
        .fallback_service(
            ServeDir::new(FRONT_PUBLIC_PATH).not_found_service(handle_error.into_service()),
        )
        .layer(TraceLayer::new_for_http())
}
