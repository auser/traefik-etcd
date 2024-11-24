use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::{MySql, Pool};

use crate::{
    config::selections::SelectionConfig, features::models::selection::NewSelectionRequest,
};

#[utoipa::path(
    post,
    path = "/selections",
    request_body = NewSelectionRequest,
    responses(
        (status = 201, description = "Selection config created successfully", body = SelectionConfig),
        (status = 400, description = "Invalid selection configuration"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn new_selection(
    Extension(pool): Extension<Pool<MySql>>,
    Json(selection): Json<NewSelectionRequest>,
) -> impl IntoResponse {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("Failed to start transaction: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SelectionConfig::default()),
            );
        }
    };

    // In MySQL, we need to use INSERT without DEFAULT VALUES
    let result = sqlx::query!(
        r#"
        INSERT INTO selection_configs (host_id, deployment_id) 
        VALUES (NULL, NULL)
        "#
    )
    .execute(&mut *tx)
    .await;

    let selection_id = match result {
        Ok(result) => result.last_insert_id() as i64,
        Err(err) => {
            tracing::error!("Failed to create selection config: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SelectionConfig::default()),
            );
        }
    };

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
            tracing::error!("Failed to create cookie config: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SelectionConfig::default()),
            );
        }
    }

    if let Some(client_ip) = &selection.from_client_ip {
        if let Err(err) = sqlx::query!(
            r#"
            INSERT INTO from_client_ip_configs (selection_config_id, ip_range, ip)
            VALUES (?, ?, ?)
            "#,
            selection_id,
            client_ip.range,
            client_ip.ip
        )
        .execute(&mut *tx)
        .await
        {
            tracing::error!("Failed to create client IP config: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SelectionConfig::default()),
            );
        }
    }

    if let Err(err) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SelectionConfig::default()),
        );
    }

    // Fetch the created config to ensure it matches what was inserted
    let cookie_config = if let Some(cookie) = &selection.with_cookie {
        match sqlx::query!(
            r#"
            SELECT name, value 
            FROM with_cookie_configs 
            WHERE selection_config_id = ?
            "#,
            selection_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(_record)) => Some(cookie.clone()),
            _ => None,
        }
    } else {
        None
    };

    let client_ip_config = if let Some(client_ip) = &selection.from_client_ip {
        match sqlx::query!(
            r#"
            SELECT ip_range, ip 
            FROM from_client_ip_configs 
            WHERE selection_config_id = ?
            "#,
            selection_id
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(record)) => {
                println!("record: {:?}", record);
                Some(client_ip.clone())
            }
            _ => None,
        }
    } else {
        None
    };

    let config = SelectionConfig {
        with_cookie: cookie_config,
        from_client_ip: client_ip_config,
    };

    (StatusCode::CREATED, Json(config))
}

#[utoipa::path(
    get,
    path = "/selection/{id}",
    responses(
        (status = 200, description = "Selection config fetched successfully", body = SelectionConfig),
    )
)]
pub async fn get_selection(
    Extension(pool): Extension<Pool<MySql>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            tracing::error!("Failed to start transaction: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SelectionConfig::default()),
            );
        }
    };

    // In MySQL, we need to use INSERT without DEFAULT VALUES
    let result = sqlx::query!(
        r#"
        INSERT INTO selection_configs (host_id, deployment_id) 
        VALUES (NULL, NULL)
        "#
    )
    .execute(&mut *tx)
    .await;

    if let Err(err) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SelectionConfig::default()),
        );
    }
    (StatusCode::OK, Json(SelectionConfig::default()))
}
