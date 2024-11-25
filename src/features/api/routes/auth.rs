use axum::{response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
}

/// route to handle log in
#[allow(clippy::unused_async)]
#[allow(clippy::missing_panics_doc)]
pub async fn login(session: Session, Json(login): Json<Login>) -> impl IntoResponse {
    tracing::info!("Logging in user: {}", login.username);

    if check_password(&login.username, &login.password) {
        session.insert("user_id", login.username).await.unwrap();
        Json(json!({"result": "ok"}))
    } else {
        Json(json!({"result": "error"}))
    }
}

/// route to handle log out
#[allow(clippy::unused_async)]
pub async fn logout(session: Session) -> impl IntoResponse {
    let user = session.get_value("user_id").await.unwrap_or_default();
    tracing::info!("Logging out user: {:?}", user);
    // drop session
    match session.flush().await {
        Ok(_) => Json(json!({"result": "ok"})),
        Err(e) => {
            tracing::error!("Error flushing session: {:?}", e);
            Json(json!({"result": "error"}))
        }
    }
}

// assume all passwords work
const fn check_password(_username: &str, _password: &str) -> bool {
    true
}

#[derive(Deserialize)]
pub struct Login {
    username: String,
    password: String,
}
