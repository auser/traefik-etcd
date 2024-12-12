use axum::Router;

use include_dir::include_dir;
use tower_http::services::ServeDir;

pub fn router() -> Router {
    let assets = include_dir!("frontend/build");
    let serve_dir = ServeDir::new(assets.path().to_path_buf());
    Router::new().nest_service("/frontend", serve_dir)
}

// .fallback_service(
//     ServeDir::new(FRONT_PUBLIC_PATH).not_found_service(handle_error.into_service()),
// )
