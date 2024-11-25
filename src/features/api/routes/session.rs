// print out session

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use tower_sessions::Session;

pub fn routes() -> Router {
    Router::new()
        .route("/session", get(handler))
        .route("/data", get(data_handler))
}

/// output entire session object
#[allow(clippy::unused_async)]
pub async fn handler(session: Session) -> impl IntoResponse {
    tracing::info!("Seeking session info");
    Json(json!({ "session": format!("{:?}", session) }))
}

/// output session data in json
#[allow(clippy::unused_async)]
pub async fn data_handler(session: Session) -> impl IntoResponse {
    tracing::info!("Seeking session data");
    let user_id = session.get_value("user_id").await.unwrap_or_default();
    Json(json!({ "user_id": user_id }))
}
