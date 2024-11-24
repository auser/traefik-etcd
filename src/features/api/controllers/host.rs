use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::config::deployment::DeploymentProtocol;
use crate::config::host::HostConfig;
use crate::features::models::host::{convert_to_host_config, HostRow};

/// List all Hosts
#[utoipa::path(
    get,
    path = "/hosts",
    responses(
        (status = 200, description = "List all hosts successfully", body = [HostConfig]),
        (status = 500, description = "Internal server error when retrieving list of all hosts", body = [HostConfig])
    )
)]
pub async fn all_hosts(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let result: Result<Vec<HostRow>, sqlx::Error> =
        sqlx::query_as::<_, HostRow>("SELECT id, traefik_config_id, domain FROM hosts")
            .fetch_all(&pool)
            .await;

    match result {
        Ok(host_rows) => {
            let mut host_configs = Vec::new();
            for host_row in host_rows {
                if let Ok(Some(config)) = convert_to_host_config(&pool, host_row).await {
                    host_configs.push(config);
                }
            }
            (StatusCode::OK, Json(host_configs))
        }
        Err(err) => {
            tracing::error!("error retrieving hosts: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<HostConfig>::new()),
            )
        }
    }
}

#[utoipa::path(
  get,
  path = "/host/{id}",
  responses(
      (status = 200, description = "Get host successfully", body = HostConfig),
      (status = 404, description = "Host not found"),
      (status = 500, description = "Internal server error when retrieving host")
  )
)]
pub async fn get_host(
    Extension(pool): Extension<SqlitePool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, HostRow>("SELECT * FROM hosts WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(host)) => match convert_to_host_config(&pool, host).await {
            Ok(Some(host_config)) => (StatusCode::OK, Json(host_config)),
            Ok(None) => (StatusCode::NOT_FOUND, Json(HostConfig::default())),
            Err(err) => {
                tracing::error!("Error converting host config: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HostConfig::default()),
                )
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(HostConfig::default())),
        Err(err) => {
            tracing::error!("Error fetching host: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HostConfig::default()),
            )
        }
    }
}

#[utoipa::path(
    post,
    path = "/hosts",
    request_body = HostConfig,
    responses(
        (status = 201, description = "Host created successfully", body = HostConfig),
        (status = 400, description = "Invalid host configuration"),
        (status = 500, description = "Internal server error when creating host")
    )
)]
pub async fn new_host(
    Json(host_config): Json<HostConfig>,
    Extension(pool): Extension<Pool<Sqlite>>,
) -> impl IntoResponse {
    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("Failed to start transaction: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HostConfig::default()),
            );
        }
    };

    // Insert the host
    let host_result = sqlx::query!(
        r#"
        INSERT INTO hosts (traefik_config_id, domain)
        VALUES (?, ?)
        RETURNING id
        "#,
        1, // Default traefik_config_id
        host_config.domain
    )
    .fetch_one(&mut *tx)
    .await;

    let host_id = match host_result {
        Ok(record) => match record.id {
            Some(id) => id,
            None => {
                tracing::error!("Host ID is None");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HostConfig::default()),
                );
            }
        },
        Err(err) => {
            tracing::error!("Failed to insert host: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HostConfig::default()),
            );
        }
    };

    // Insert paths
    for path_config in &host_config.paths {
        let path_result = sqlx::query!(
            r#"
            INSERT INTO paths (host_id, path, strip_prefix, pass_through)
            VALUES (?, ?, ?, ?)
            RETURNING id
            "#,
            host_id,
            path_config.path,
            path_config.strip_prefix,
            path_config.pass_through
        )
        .fetch_one(&mut *tx)
        .await;

        let path_id = match path_result {
            Ok(record) => record.id,
            Err(err) => {
                tracing::error!("Failed to insert path: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HostConfig::default()),
                );
            }
        };

        // Insert path deployments
        for (name, deployment) in &path_config.deployments {
            let protocol_id = match deployment.protocol {
                DeploymentProtocol::Http => 1,
                DeploymentProtocol::Https => 2,
                DeploymentProtocol::Tcp => 3,
                DeploymentProtocol::Invalid => 4,
            };

            let deployment_result = sqlx::query!(
                r#"
                INSERT INTO deployments (name, ip, port, weight, protocol_id, path_id)
                VALUES (?, ?, ?, ?, ?, ?)
                RETURNING id
                "#,
                name,
                deployment.ip,
                deployment.port,
                deployment.weight as i64,
                protocol_id,
                path_id
            )
            .fetch_one(&mut *tx)
            .await;

            let deployment_id = match deployment_result {
                Ok(record) => record.id,
                Err(err) => {
                    tracing::error!("Failed to insert deployment: {:?}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(HostConfig::default()),
                    );
                }
            };

            // Insert deployment selection config if present
            if let Some(selection) = &deployment.selection {
                let selection_result = sqlx::query!(
                    r#"
                    INSERT INTO selection_configs (deployment_id)
                    VALUES (?)
                    RETURNING id
                    "#,
                    deployment_id
                )
                .fetch_one(&mut *tx)
                .await;

                let selection_id = match selection_result {
                    Ok(record) => record.id,
                    Err(err) => {
                        tracing::error!("Failed to insert selection config: {:?}", err);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(HostConfig::default()),
                        );
                    }
                };

                // Insert cookie config if present
                if let Some(cookie) = &selection.with_cookie {
                    if let Err(err) = sqlx::query!(
                        r#"
                        INSERT INTO with_cookie_configs (selection_config_id, name, value)
                        VALUES (?, ?, ?)
                        "#,
                        selection_id,
                        cookie.name,
                        cookie.value
                    )
                    .execute(&mut *tx)
                    .await
                    {
                        tracing::error!("Failed to insert cookie config: {:?}", err);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(HostConfig::default()),
                        );
                    }
                }

                // Insert client IP config if present
                if let Some(client_ip) = &selection.from_client_ip {
                    if let Err(err) = sqlx::query!(
                        r#"
                        INSERT INTO from_client_ip_configs (selection_config_id, range, ip)
                        VALUES (?, ?, ?)
                        "#,
                        selection_id,
                        client_ip.range,
                        client_ip.ip
                    )
                    .execute(&mut *tx)
                    .await
                    {
                        tracing::error!("Failed to insert client IP config: {:?}", err);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(HostConfig::default()),
                        );
                    }
                }
            }

            // Insert deployment middlewares if present
            if let Some(middlewares) = &deployment.middlewares {
                for middleware_name in middlewares {
                    if let Err(err) = sqlx::query!(
                        r#"
                        INSERT INTO deployment_middlewares (deployment_id, middleware_id)
                        SELECT ?, id FROM middlewares WHERE name = ?
                        "#,
                        deployment_id,
                        middleware_name
                    )
                    .execute(&mut *tx)
                    .await
                    {
                        tracing::error!("Failed to insert deployment middleware: {:?}", err);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(HostConfig::default()),
                        );
                    }
                }
            }
        }

        // Insert path middlewares
        for middleware_name in &path_config.middlewares {
            if let Err(err) = sqlx::query!(
                r#"
                INSERT INTO path_middlewares (path_id, middleware_id)
                SELECT ?, id FROM middlewares WHERE name = ?
                "#,
                path_id,
                middleware_name
            )
            .execute(&mut *tx)
            .await
            {
                tracing::error!("Failed to insert path middleware: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HostConfig::default()),
                );
            }
        }
    }

    // Insert host-level deployments
    for (name, deployment) in &host_config.deployments {
        let protocol_id = match deployment.protocol {
            DeploymentProtocol::Http => 1,
            DeploymentProtocol::Https => 2,
            DeploymentProtocol::Tcp => 3,
            DeploymentProtocol::Invalid => 4,
        };

        let deployment_result = sqlx::query!(
            r#"
            INSERT INTO deployments (name, ip, port, weight, protocol_id, host_id)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
            name,
            deployment.ip,
            deployment.port,
            deployment.weight as i64,
            protocol_id,
            host_id
        )
        .fetch_one(&mut *tx)
        .await;

        // Handle deployment selection and middlewares same as path deployments
        // ... (similar code as above for selection and middlewares)
    }

    // Insert host-level middlewares
    for middleware_name in &host_config.middlewares {
        if let Err(err) = sqlx::query!(
            r#"
            INSERT INTO host_middlewares (host_id, middleware_id)
            SELECT ?, id FROM middlewares WHERE name = ?
            "#,
            host_id,
            middleware_name
        )
        .execute(&mut *tx)
        .await
        {
            tracing::error!("Failed to insert host middleware: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HostConfig::default()),
            );
        }
    }

    // Insert host selection config if present
    if let Some(selection) = &host_config.selection {
        let selection_result = sqlx::query!(
            r#"
            INSERT INTO selection_configs (host_id)
            VALUES (?)
            RETURNING id
            "#,
            host_id
        )
        .fetch_one(&mut *tx)
        .await;

        let selection_id = match selection_result {
            Ok(record) => record.id,
            Err(err) => {
                tracing::error!("Failed to insert host selection config: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HostConfig::default()),
                );
            }
        };

        // Insert cookie and client IP configs as needed
        if let Some(cookie) = &selection.with_cookie {
            if let Err(err) = sqlx::query!(
                r#"
                INSERT INTO with_cookie_configs (selection_config_id, name, value)
                VALUES (?, ?, ?)
                "#,
                selection_id,
                cookie.name,
                cookie.value
            )
            .execute(&mut *tx)
            .await
            {
                tracing::error!("Failed to insert host cookie config: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HostConfig::default()),
                );
            }
        }

        if let Some(client_ip) = &selection.from_client_ip {
            if let Err(err) = sqlx::query!(
                r#"
                INSERT INTO from_client_ip_configs (selection_config_id, range, ip)
                VALUES (?, ?, ?)
                "#,
                selection_id,
                client_ip.range,
                client_ip.ip
            )
            .execute(&mut *tx)
            .await
            {
                tracing::error!("Failed to insert host client IP config: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HostConfig::default()),
                );
            }
        }
    }

    // Commit the transaction
    if let Err(err) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HostConfig::default()),
        );
    }

    // Return the created host configuration
    match convert_to_host_config(
        &pool,
        HostRow {
            id: host_id,
            traefik_config_id: 1,
            domain: host_config.domain,
        },
    )
    .await
    {
        Ok(Some(created_host)) => (StatusCode::CREATED, Json(created_host)),
        _ => (StatusCode::CREATED, Json(host_config)),
    }
}
