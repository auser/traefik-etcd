// pub mod deployments;
// // pub mod host;
// pub mod health_check;
// pub mod selection;

use axum::{Extension, Json};
use sqlx::{MySql, Pool};

use super::{
    db,
    models::{ConfigVersion, DeploymentProtocol, SaveConfigRequest},
};

/// Get all deployment protocols
#[utoipa::path(
  get,
  path = "/api/protocols",
  responses(
      (status = 200, description = "List of deployment protocols", body = Vec<DeploymentProtocol>)
  )
)]
pub async fn get_protocols(
    Extension(pool): Extension<Pool<MySql>>,
) -> Json<Vec<DeploymentProtocol>> {
    Json(
        db::operations::get_deployment_protocols(&pool)
            .await
            .unwrap_or_default(),
    )
}

/// Get all configurations
#[utoipa::path(
  get,
  path = "/api/configs",
  responses(
      (status = 200, description = "List of configurations", body = Vec<ConfigVersion>)
  )
)]
pub async fn get_configs() -> Json<Vec<ConfigVersion>> {
    Json(db::operations::get_configs().await.unwrap_or_default())
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
pub async fn save_config(Json(request): Json<SaveConfigRequest>) -> Json<ConfigVersion> {
    Json(
        db::operations::save_config(request.name, request.config)
            .await
            .unwrap(),
    )
}
