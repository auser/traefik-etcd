use axum::Router;
// use tower_http::{cors::CorsLayer, trace::TraceLayer};

use axum_embed::ServeEmbed;
use rust_embed::RustEmbed;

#[derive(RustEmbed, Clone)]
#[folder = "frontend/build"]
struct Assets;

pub fn router() -> Router {
    let serve_assets = ServeEmbed::<Assets>::new();
    Router::new().nest_service("/frontend", serve_assets)
}

// .fallback_service(
//     ServeDir::new(FRONT_PUBLIC_PATH).not_found_service(handle_error.into_service()),
// )
