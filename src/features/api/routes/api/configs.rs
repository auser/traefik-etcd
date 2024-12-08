use axum::extract::{Path, Query};
use axum::routing::{delete, get, post, put};
use axum::{Extension, Json, Router};
use tracing::{debug, error};

use crate::config::traefik_config::{ConfigVersionHistory, TraefikConfigVersion};
use crate::features::controllers::configs::{get_database_config, SearchConfigsParams};
use crate::features::models::SaveConfigRequest;
use crate::features::routes::ApiContext;
use crate::features::{controllers, TraefikApiResult, TraefikConfigListItem};
use crate::TraefikConfig;

pub fn routes() -> Router {
    Router::new()
        .route("/configs", get(get_all_configs))
        .route("/configs", post(save_config))
        .route("/configs/search", get(search_configs))
        .route("/configs/default", get(get_default_config))
        .route("/configs/files", get(get_file_configs))
        .route("/configs/version", post(save_config_version))
        .route("/configs/backup/:id", post(create_config_backup))
        .route("/configs/id/:id", get(get_config_by_id))
        .route("/configs/update/:id", put(update_config))
        .route("/configs/history/:id", get(get_config_history))
        .route("/configs/delete/:id", delete(delete_config))
}

/// Get all configurations
#[utoipa::path(
  get,
  path = "/api/configs",
  responses(
      (status = 200, description = "List of configurations", body = Vec<TraefikConfigVersion>)
  ),
  tags = ["config"]
)]
pub(crate) async fn get_all_configs(
    ctx: Extension<ApiContext>,
) -> TraefikApiResult<Json<Vec<TraefikConfigListItem>>> {
    let base_config_path = Some(ctx.config.base_templates_path.clone());
    let all_configs = controllers::configs::list_all_configs(&ctx.db, base_config_path).await?;
    Ok(Json(all_configs))
}

/// Get all file-based configurations
#[utoipa::path(
    get,
    path = "/api/configs/files",
    responses(
        (status = 200, description = "List of configurations", body = Vec<TraefikConfigVersion>)
    ),
    tags = ["config"]
  )]
pub(crate) async fn get_file_configs(
    ctx: Extension<ApiContext>,
) -> TraefikApiResult<Json<Vec<TraefikConfigVersion>>> {
    let file_configs =
        controllers::configs::get_yaml_configs(&ctx.config.base_templates_path).await?;
    Ok(Json(file_configs))
}

#[utoipa::path(
    get,
    path = "/api/configs/search",
    params(
        ("search" = Option<String>, Query, description = "Search term for filtering configs")
    ),
    responses(
            (status = 200, description = "List of available configs", body = Vec<TraefikConfigVersion>)
    ),
    tags = ["configs"]
)]
pub(crate) async fn search_configs(
    ctx: Extension<ApiContext>,
    Query(params): Query<SearchConfigsParams>,
) -> TraefikApiResult<Json<Vec<TraefikConfigVersion>>> {
    debug!(
        "Searching templates with search term: {:?}",
        params.search_term()
    );
    let configs = controllers::configs::search_configs(&ctx.db, params.search_term()).await?;
    Ok(Json(configs))
}

/// Get default configuration
#[utoipa::path(
    get,
    path = "/api/configs/default",
    responses(
        (status = 200, description = "default config", body = TraefikConfigVersion)
    ),
    tags = ["config"]
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
  ),
  tags = ["config"]
)]
pub(crate) async fn save_config(
    ctx: Extension<ApiContext>,
    Json(request): Json<SaveConfigRequest>,
) -> TraefikApiResult<Json<TraefikConfigVersion>> {
    let result = controllers::configs::save_config(&ctx.db, request).await?;
    Ok(Json(result))
}

/// Save a new version of a configuration
#[utoipa::path(
    post,
    path = "/api/configs/version",
    request_body = SaveConfigRequest,
    responses(
        (status = 201, description = "New configuration version saved successfully", body = TraefikConfigVersion),
        (status = 400, description = "Invalid request body")
    ),
    tags = ["config"]
)]
pub(crate) async fn save_config_version(
    ctx: Extension<ApiContext>,
    Json(request): Json<SaveConfigRequest>,
) -> TraefikApiResult<Json<TraefikConfigVersion>> {
    // We increase the version number and create a new record
    let result = controllers::configs::save_config_version(&ctx.db, request).await?;
    Ok(Json(result))
}

/// Update an existing configuration
#[utoipa::path(
    put,
    path = "/api/configs/{id}",
    request_body = SaveConfigRequest,
    params(
        ("id" = i64, Path, description = "Configuration ID to update")
    ),
    responses(
        (status = 200, description = "Configuration updated successfully", body = TraefikConfigVersion),
        (status = 400, description = "Invalid request body"),
        (status = 404, description = "Configuration not found")
    ),
    tags = ["config"]
)]
pub(crate) async fn update_config(
    ctx: Extension<ApiContext>,
    Path(id): Path<i64>,
    Json(request): Json<SaveConfigRequest>,
) -> TraefikApiResult<Json<TraefikConfigVersion>> {
    let result =
        controllers::configs::update_config_by_option(&ctx.db, id, request, &ctx.file_config)
            .await?;
    Ok(Json(result))
}

/// Get configuration by ID
#[utoipa::path(
    get,
    path = "/api/configs/id/{id}",
    params(
        ("id" = i64, Path, description = "Configuration ID. Positive for database configs, negative for files, -1 for default, 0 for new")
    ),
    responses(
        (status = 200, description = "Configuration found", body = TraefikConfigVersion),
        (status = 404, description = "Configuration not found")
    ),
    tags = ["config"]
)]
pub(crate) async fn get_config_by_id(
    ctx: Extension<ApiContext>,
    Path(id): Path<i64>,
) -> TraefikApiResult<Json<TraefikConfigVersion>> {
    let config = match id {
        -1 => {
            // Return default config
            let default_config = TraefikConfig::generate_config(None);
            let serialized = serde_yaml::to_string(&default_config)?;

            TraefikConfigVersion {
                id: -1,
                name: "Default Configuration".to_string(),
                config: serde_json::Value::String(serialized),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
            }
        }
        0 => {
            // Return empty/new config template
            let empty_config = TraefikConfig::default();
            let serialized = serde_yaml::to_string(&empty_config)?;

            TraefikConfigVersion {
                id: 0,
                name: "New Configuration".to_string(),
                config: serde_json::Value::String(serialized),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
            }
        }
        id if id < -1 => {
            // Handle file-based configs
            controllers::configs::get_file_config_route(id, &ctx.file_config).await?
        }
        id => {
            // Handle database configs
            get_database_config(&ctx.db, id).await?
        }
    };

    Ok(Json(config))
}

/// Get version history for a config
#[utoipa::path(
    get,
    path = "/api/configs/history/{id}",
    responses(
        (status = 200, description = "Version history retrieved successfully", body = Vec<ConfigVersionHistory>),
        (status = 404, description = "Configuration not found")
    ),
    tags = ["config"]
)]
pub(crate) async fn get_config_history(
    ctx: Extension<ApiContext>,
    Path(id): Path<i64>,
) -> TraefikApiResult<Json<Vec<ConfigVersionHistory>>> {
    let history = controllers::configs::get_config_history(&ctx.db, id).await?;
    Ok(Json(history))
}

/// Create backup of current config state
#[utoipa::path(
    post,
    path = "/api/configs/backup/{id}",
    request_body = SaveConfigRequest,
    responses(
        (status = 201, description = "Backup created successfully", body = ConfigVersionHistory),
        (status = 404, description = "Configuration not found")
    ),
    tags = ["config"]
)]
pub(crate) async fn create_config_backup(
    ctx: Extension<ApiContext>,
    Path(id): Path<i64>,
    Json(request): Json<SaveConfigRequest>,
) -> TraefikApiResult<Json<ConfigVersionHistory>> {
    let backup = controllers::configs::create_config_backup(&ctx.db, id, request).await?;
    Ok(Json(backup))
}

/// Delete a configuration
#[utoipa::path(
    delete,
    path = "/api/configs/delete/{id}",
    params(
        ("id" = i64, Path, description = "Configuration ID to delete")
    ),
    responses(
        (status = 200, description = "Configuration deleted successfully"),
        (status = 404, description = "Configuration not found")
    ),
    tags = ["config"]
)]
pub(crate) async fn delete_config(
    ctx: Extension<ApiContext>,
    Path(id): Path<i64>,
) -> TraefikApiResult<Json<()>> {
    controllers::configs::delete_config(&ctx.db, id).await?;
    Ok(Json(()))
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
        // init_test_tracing();
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
        let body: Vec<TraefikConfigVersion> = serde_json::from_slice(&body).unwrap();
        assert!(!body.is_empty());
        assert!(body.iter().any(|c| c.name == "default"));
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
        assert_eq!(body.name, "default");
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
