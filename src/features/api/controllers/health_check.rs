use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::{MySql, Pool};

use crate::{
    config::health_check::HealthCheckConfig, features::models::health_check::NewHealthCheckRequest,
};

#[utoipa::path(
    post,
    path = "/health_check",
    request_body = NewHealthCheckRequest,
    responses(
        (status = 201, description = "Health check created successfully", body = HealthCheckConfig),
        (status = 400, description = "Invalid selection configuration"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn new_health_check(
    Extension(pool): Extension<Pool<MySql>>,
    Json(health_check): Json<NewHealthCheckRequest>,
) -> impl IntoResponse {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("Failed to start transaction: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HealthCheckConfig::default()),
            );
        }
    };

    // In MySQL, we need to use INSERT without DEFAULT VALUES
    let result = sqlx::query!(
        r#"
        INSERT INTO health_checks (deployment_id, path, check_interval, check_timeout)
        VALUES (?, ?, ?, ?)
        "#,
        health_check.deployment_id,
        health_check.path,
        health_check.check_interval,
        health_check.check_timeout
    )
    .execute(&mut *tx)
    .await;

    let health_check_id = match result {
        Ok(result) => result.last_insert_id() as i64,
        Err(err) => {
            tracing::error!("Failed to create health check: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HealthCheckConfig::default()),
            );
        }
    };

    if let Err(err) = tx.commit().await {
        tracing::error!("Failed to create health check: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HealthCheckConfig::default()),
        );
    }

    (StatusCode::CREATED, Json(HealthCheckConfig::default()))
}

#[utoipa::path(
    get,
    path = "/selection/{id}",
    responses(
        (status = 200, description = "Health check fetched successfully", body = HealthCheckConfig),
    )
)]
pub async fn get_health_check(
    Extension(pool): Extension<Pool<MySql>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("Failed to start transaction: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HealthCheckConfig::default()),
            );
        }
    };

    // In MySQL, we need to use INSERT without DEFAULT VALUES
    let result = sqlx::query!(
        r#"
        SELECT * FROM health_checks WHERE id = ?
        "#,
        id
    )
    .fetch_one(&mut *tx)
    .await;

    if let Err(err) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HealthCheckConfig::default()),
        );
    }
    (StatusCode::OK, Json(HealthCheckConfig::default()))
}
