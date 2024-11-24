use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};

use sqlx::{MySql, Pool};
use std::str::FromStr;

use crate::{
    config::deployment::{DeploymentConfig, DeploymentProtocol},
    features::models::deployments::NewDeploymentRequest,
};

#[utoipa::path(
    post,
    path = "/deployments",
    request_body = NewDeploymentRequest,
    responses(
        (status = 201, description = "Deployment created successfully", body = DeploymentConfig),
        (status = 400, description = "Invalid deployment configuration"),
        (status = 500, description = "Internal server error")
    ),
    tag = "deployments"
)]
pub async fn new_deployment(
    Extension(pool): Extension<Pool<MySql>>,
    Json(deployment): Json<NewDeploymentRequest>,
) -> impl IntoResponse {
    let protocol_id = match deployment.protocol {
        DeploymentProtocol::Http => 1,
        DeploymentProtocol::Https => 2,
        DeploymentProtocol::Tcp => 3,
        DeploymentProtocol::Invalid => 4,
    };

    if std::net::IpAddr::from_str(&deployment.ip).is_err() {
        return (StatusCode::BAD_REQUEST, Json(DeploymentConfig::default()));
    }

    let name = deployment.name.clone();
    let ip = deployment.ip.clone();
    let port = deployment.port;
    let weight = deployment.weight as i64;

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("Failed to start transaction: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeploymentConfig::default()),
            );
        }
    };

    // Insert the deployment
    let result = sqlx::query!(
        r#"
        INSERT INTO deployments (name, ip, port, weight, protocol_id)
        VALUES (?, ?, ?, ?, ?)
        "#,
        name,
        ip,
        port,
        weight,
        protocol_id
    )
    .execute(&mut *tx)
    .await;

    match result {
        Ok(result) => {
            // Get the last inserted ID
            let id = result.last_insert_id();

            // Fetch the inserted record
            let record = sqlx::query!(
                r#"
                SELECT id, name, ip, port, weight, protocol_id
                FROM deployments
                WHERE id = ?
                "#,
                id
            )
            .fetch_one(&mut *tx)
            .await;

            // Commit the transaction
            if let Err(err) = tx.commit().await {
                tracing::error!("Failed to commit transaction: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DeploymentConfig::default()),
                );
            }

            match record {
                Ok(record) => {
                    let config = DeploymentConfig {
                        name: record.name,
                        ip: record.ip,
                        port: record.port as u16,
                        weight: record.weight as usize,
                        protocol: DeploymentProtocol::from(record.protocol_id),
                        selection: None,
                        middlewares: None,
                    };
                    (StatusCode::CREATED, Json(config))
                }
                Err(err) => {
                    tracing::error!("Failed to fetch created deployment: {:?}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(DeploymentConfig::default()),
                    )
                }
            }
        }
        Err(err) => {
            tracing::error!("Failed to create deployment: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeploymentConfig::default()),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/deployments",
    responses(
        (status = 200, description = "Deployments fetched successfully", body = [DeploymentConfig]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn all_deployments(Extension(pool): Extension<Pool<MySql>>) -> impl IntoResponse {
    match sqlx::query!(
        r#"
        SELECT id, name, ip, port, weight, protocol_id 
        FROM deployments
        "#
    )
    .fetch_all(&pool)
    .await
    {
        Ok(records) => {
            let deployments: Vec<DeploymentConfig> = records
                .into_iter()
                .map(|record| DeploymentConfig {
                    name: record.name,
                    ip: record.ip,
                    port: record.port as u16,
                    weight: record.weight as usize,
                    protocol: DeploymentProtocol::from(record.protocol_id),
                    selection: None,
                    middlewares: None,
                })
                .collect();
            (StatusCode::OK, Json(deployments))
        }
        Err(err) => {
            tracing::error!("Failed to fetch deployments: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<DeploymentConfig>::new()),
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/deployments/{id}",
    responses(
        (status = 200, description = "Deployment fetched successfully", body = DeploymentConfig),
        (status = 404, description = "Deployment not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "deployments"
)]
pub async fn get_deployment(
    Extension(pool): Extension<Pool<MySql>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"
        SELECT id, name, ip, port, weight, protocol_id 
        FROM deployments
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(record)) => {
            let deployment = DeploymentConfig {
                name: record.name,
                ip: record.ip,
                port: record.port as u16,
                weight: record.weight as usize,
                protocol: DeploymentProtocol::from(record.protocol_id),
                selection: None,
                middlewares: None,
            };
            (StatusCode::OK, Json(deployment))
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(DeploymentConfig::default())),
        Err(err) => {
            tracing::error!("Failed to fetch deployment: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeploymentConfig::default()),
            )
        }
    }
}
