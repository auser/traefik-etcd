use axum::Router;
// use tower_http::{cors::CorsLayer, trace::TraceLayer};

use include_dir::include_dir;
// use axum_embed::ServeEmbed;
// use rust_embed::RustEmbed;
use tower_http::services::ServeDir;

// #[derive(RustEmbed, Clone)]
// #[folder = "$CARGO_MANIFEST_DIR/frontend/build"]
// struct Assets;

pub fn router() -> Router {
    let assets = include_dir!("frontend/build");
    let serve_dir = ServeDir::new(assets.path());
    Router::new().nest_service("/frontend", serve_dir)
}

// .fallback_service(
//     ServeDir::new(FRONT_PUBLIC_PATH).not_found_service(handle_error.into_service()),
// )
