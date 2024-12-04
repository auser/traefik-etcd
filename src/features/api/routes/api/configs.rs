use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use tracing::error;

use crate::config::traefik_config::TraefikConfigVersion;
use crate::features::models::SaveConfigRequest;
use crate::features::routes::ApiContext;
use crate::features::{controllers, db, TraefikApiResult};

pub fn routes() -> Router {
    Router::new()
        .route("/configs", get(get_configs))
        .route("/configs", post(save_config))
        .route("/configs/default", get(get_default_config))
        .route("/configs/files", get(get_file_configs))
}

/// Get all configurations
#[utoipa::path(
  get,
  path = "/api/configs",
  responses(
      (status = 200, description = "List of configurations", body = Vec<TraefikConfigVersion>)
  )
)]
pub(crate) async fn get_configs(
    ctx: Extension<ApiContext>,
) -> TraefikApiResult<Json<Vec<TraefikConfigVersion>>> {
    let configs = db::operations::configs::get_configs(&ctx.db).await?;
    Ok(Json(configs))
}

#[utoipa::path(
    get,
    path = "/api/configs/files",
    responses(
        (status = 200, description = "List of configurations", body = Vec<TraefikConfigVersion>)
    )
  )]
pub(crate) async fn get_file_configs(
    ctx: Extension<ApiContext>,
) -> TraefikApiResult<Json<Vec<TraefikConfigVersion>>> {
    let file_configs = controllers::configs::get_yaml_configs(&ctx.config.base_config_path).await?;
    Ok(Json(file_configs))
}

/// Get default configuration
#[utoipa::path(
    get,
    path = "/api/configs/default",
    responses(
        (status = 200, description = "Default configuration", body = TraefikConfigVersion)
    )
  )]
pub(crate) async fn get_default_config(
    _ctx: Extension<ApiContext>,
) -> TraefikApiResult<Json<TraefikConfigVersion>> {
    match controllers::configs::get_default_config().await {
        Ok(config) => Ok(Json(config)),
        Err(e) => {
            error!("Error getting default config: {:?}", e);
            Err(e)
        }
    }
}

/// Save a new configuration
#[utoipa::path(
  post,
  path = "/api/configs",
  request_body = SaveConfigRequest,
  responses(
      (status = 201, description = "Configuration saved successfully", body = TraefikConfigVersion),
      (status = 400, description = "Invalid request body")
  )
)]
pub(crate) async fn save_config(
    ctx: Extension<ApiContext>,
    Json(request): Json<SaveConfigRequest>,
) -> TraefikApiResult<Json<TraefikConfigVersion>> {
    let result = controllers::configs::save_config(&ctx.db, request).await?;
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use tracing::debug;

    #[allow(unused_imports)]
    use crate::test_helpers::init_test_tracing;
    use http_body_util::BodyExt;

    use crate::{
        config::traefik_config::TraefikConfigVersion,
        features::{create_db_test_config, models::SaveConfigRequest, setup_test_server},
        TraefikConfig,
    };
    use axum::http::StatusCode;

    #[tokio::test]
    async fn get_all_configs_test() {
        init_test_tracing();
        let server = setup_test_server().await.unwrap();
        create_db_test_config(&server.db, None).await.unwrap();
        debug!("Server and database setup");
        let _test_user = server.test_user.clone();

        debug!("Testing get all configs");
        let response = server.get("/api/configs").await;
        debug!("Response: {:?}", response);
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        println!("Raw Body: {:?}", body);
        let body: Vec<TraefikConfigVersion> = serde_json::from_slice(&body).unwrap();
        println!("Body: {:?}", body);
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn get_file_configs_test() {
        // init_test_tracing();
        let server = setup_test_server().await.unwrap();
        debug!("Testing get file configs");
        let response = server.get("/api/configs/files").await;
        debug!("Response: {:?}", response);
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Vec<TraefikConfigVersion> = serde_json::from_slice(&body).unwrap();
        assert!(!body.is_empty());
        let config_names = body.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        assert!(config_names.contains(&"herringbank.yml".to_string()));
    }

    #[tokio::test]
    async fn test_get_default_config() {
        let server = setup_test_server().await.unwrap();
        let response = server.get("/api/configs/default").await;
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: TraefikConfigVersion = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.name, "Default Configuration");
    }

    #[tokio::test]
    async fn test_save_config() {
        let server = setup_test_server().await.unwrap();
        let default_traefik_config = TraefikConfig::default();
        let request = SaveConfigRequest {
            name: "test_config".to_string(),
            config: default_traefik_config.into(),
        };
        let response = server.post("/api/configs", request.into()).await;
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: TraefikConfigVersion = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.name, "test_config");
    }
}
