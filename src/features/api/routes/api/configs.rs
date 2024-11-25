use axum::routing::{get, post};
use axum::{Extension, Json, Router};

use crate::features::db;
use crate::features::models::{ConfigVersion, SaveConfigRequest};
use crate::features::routes::ApiContext;

pub fn routes() -> Router {
    Router::new()
        .route("/configs", get(get_configs))
        .route("/configs", post(save_config))
}

/// Get all configurations
#[utoipa::path(
  get,
  path = "/api/configs",
  responses(
      (status = 200, description = "List of configurations", body = Vec<ConfigVersion>)
  )
)]
pub(crate) async fn get_configs(ctx: Extension<ApiContext>) -> Json<Vec<ConfigVersion>> {
    Json(
        db::operations::get_configs(&ctx.db)
            .await
            .unwrap_or_default(),
    )
}

/// Save a new configuration
#[utoipa::path(
  post,
  path = "/api/configs",
  request_body = SaveConfigRequest,
  responses(
      (status = 201, description = "Configuration saved successfully", body = ConfigVersion),
      (status = 400, description = "Invalid request body")
  )
)]
pub(crate) async fn save_config(
    ctx: Extension<ApiContext>,
    Json(request): Json<SaveConfigRequest>,
) -> Json<ConfigVersion> {
    Json(
        db::operations::save_config(&ctx.db, request.name, request.config)
            .await
            .unwrap(),
    )
}
