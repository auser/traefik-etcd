use std::path::Path;

use chrono::{DateTime, Utc};
use sqlx::{Acquire, MySql, Pool};
use tokio::fs;
use tracing::error;
use walkdir::WalkDir;

use crate::{
    config::traefik_config::TraefikConfigVersion,
    features::{
        db,
        file_loader::{save_file_config, FileConfig},
        models::SaveConfigRequest,
        TraefikApiError, TraefikApiResult, TraefikConfigListItem, TraefikConfigSource,
    },
    TraefikConfig,
};

pub async fn get_yaml_configs(base_path: &str) -> TraefikApiResult<Vec<TraefikConfigVersion>> {
    let mut configs = Vec::new();
    let base_path = Path::new(base_path);

    // Only proceed if directory exists
    if base_path.exists() && base_path.is_dir() {
        for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        let content = fs::read_to_string(path).await?;
                        let relative_path = path.strip_prefix(base_path).unwrap_or(path);

                        let created_at = match fs::metadata(path).await?.created() {
                            Ok(created_at) => created_at,
                            Err(e) => {
                                error!("error in get_yaml_configs: {:?}", e);
                                Utc::now().into()
                            }
                        }
                        .into();
                        let updated_at = match fs::metadata(path).await?.modified() {
                            Ok(updated_at) => updated_at,
                            Err(e) => {
                                error!("error in get_yaml_configs: {:?}", e);
                                Utc::now().into()
                            }
                        }
                        .into();

                        configs.push(TraefikConfigVersion {
                            id: 0, // Use 0 or negative IDs for file-based configs
                            name: relative_path.display().to_string(),
                            config: serde_json::Value::String(content),
                            created_at,
                            updated_at,
                            version: 1,
                        });
                    }
                }
            }
        }
    }
    Ok(configs)
}

pub async fn get_default_config() -> TraefikApiResult<TraefikConfigVersion> {
    let default_config = TraefikConfig::generate_config(None);
    let serialized = match serde_yaml::to_string(&default_config) {
        Ok(serialized) => serialized,
        Err(e) => {
            error!("error in get_default_config: {:?}", e);
            return Err(TraefikApiError::InternalServerError);
        }
    };

    let created_at = match DateTime::from_timestamp(0, 0) {
        Some(created_at) => created_at,
        None => {
            error!("error in get_default_config");
            Utc::now()
        }
    };
    let updated_at = created_at;

    Ok(TraefikConfigVersion {
        id: -1,
        name: "default".to_string(),
        config: serde_json::Value::String(serialized),
        created_at,
        updated_at,
        version: 1,
    })
}

pub async fn save_config(
    db: &Pool<MySql>,
    request: SaveConfigRequest,
) -> TraefikApiResult<TraefikConfigVersion> {
    let result = db::operations::configs::save_config(db, request.name, request.config).await?;
    Ok(result)
}

pub async fn save_config_version(
    pool: &Pool<MySql>,
    request: SaveConfigRequest,
) -> TraefikApiResult<TraefikConfigVersion> {
    let mut conn = pool.acquire().await?;
    let mut tx = conn.begin().await?;

    // Insert the new record
    sqlx::query(
        r#"
        INSERT INTO config_versions (name, config)
        VALUES (?, ?)
        "#,
    )
    .bind(&request.name)
    .bind(&request.config)
    .execute(&mut *tx)
    .await?;

    // Get the newly inserted ID
    let id: u64 = sqlx::query_scalar(
        r#"
        SELECT LAST_INSERT_ID()
        "#,
    )
    .fetch_one(&mut *tx)
    .await?;

    let id = id as i64;

    // Fetch the complete record
    let config = sqlx::query_as::<_, TraefikConfigVersion>(
        r#"
        SELECT id, name, config, created_at, updated_at, version
        FROM config_versions 
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(config)
}

pub async fn get_configs(db: &Pool<MySql>) -> TraefikApiResult<Vec<TraefikConfigVersion>> {
    let configs = db::operations::configs::get_configs(db).await?;
    Ok(configs)
}

pub async fn update_database_config(
    pool: &Pool<MySql>,
    id: i64,
    request: SaveConfigRequest,
) -> TraefikApiResult<TraefikConfigVersion> {
    // First check if config exists
    let _existing = sqlx::query_as::<_, TraefikConfigVersion>(
        r#"
      SELECT id, name, config, created_at, updated_at, version
      FROM config_versions 
      WHERE id = ?
      "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| TraefikApiError::NotFound(format!("Configuration {} not found", id)))?;

    // Update the configuration
    sqlx::query(
        r#"
      UPDATE config_versions 
      SET config = ?, name = ?, version = version + 1
      WHERE id = ?
      "#,
    )
    .bind(&request.config)
    .bind(&request.name)
    .bind(id)
    .execute(pool)
    .await?;

    // Fetch the updated record
    let result = sqlx::query_as::<_, TraefikConfigVersion>(
        r#"
      SELECT id, name, config, created_at, updated_at, version
      FROM config_versions 
      WHERE id = ?
      "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn update_config_by_option(
    pool: &Pool<MySql>,
    id: i64,
    request: SaveConfigRequest,
    file_config: &FileConfig,
) -> TraefikApiResult<TraefikConfigVersion> {
    let config = if id < -1 {
        // File-based config
        save_file_config(id, &request.config.to_string(), file_config).await?
    } else {
        // Database config
        update_database_config(pool, id, request).await?
    };
    Ok(config)
}

pub async fn list_all_configs(
    pool: &Pool<MySql>,
    config_dir: Option<String>,
) -> TraefikApiResult<Vec<TraefikConfigListItem>> {
    let mut configs = Vec::new();

    // Add "New Configuration" option
    configs.push(TraefikConfigListItem {
        id: -1,
        name: "New Configuration".to_string(),
        source: TraefikConfigSource::New,
        updated_at: Utc::now(),
    });

    // Add default configuration
    configs.push(TraefikConfigListItem {
        id: -2,
        name: "Default Configuration".to_string(),
        source: TraefikConfigSource::Default,
        updated_at: Utc::now(),
    });

    // Add file-based configs if directory exists
    if let Some(dir) = config_dir {
        let dir = Path::new(&dir);
        if dir.exists() && dir.is_dir() {
            for entry in walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map_or(false, |ext| ext == "yaml" || ext == "yml")
                })
            {
                let meta = entry.metadata()?;
                let id = -(configs.len() as i64 + 1); // Unique negative ID
                configs.push(TraefikConfigListItem {
                    id,
                    name: entry
                        .path()
                        .strip_prefix(dir)
                        .unwrap_or(entry.path())
                        .display()
                        .to_string(),
                    source: TraefikConfigSource::File,
                    updated_at: meta.modified()?.into(),
                });
            }
        }
    }

    // Add database configs
    let db_configs = get_configs(pool).await?;
    for config in db_configs {
        configs.push(TraefikConfigListItem {
            id: config.id,
            name: config.name,
            source: TraefikConfigSource::Database,
            updated_at: config.updated_at,
        });
    }

    Ok(configs)
}

/// Delete a configuration
pub async fn delete_config(pool: &Pool<MySql>, id: i64) -> TraefikApiResult<()> {
    println!("deleting config: {}", id);
    let result = sqlx::query(
        r#"
        DELETE FROM config_versions 
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    println!("rows_affected: {}", result.rows_affected());

    if result.rows_affected() == 0 {
        return Err(TraefikApiError::NotFound(format!(
            "Configuration {} not found",
            id
        )));
    }

    println!("config deleted");

    Ok(())
}

pub async fn get_file_config_route(
    id: i64,
    file_config: &FileConfig,
) -> TraefikApiResult<TraefikConfigVersion> {
    // Implementation for file-based configs
    // You'll need to implement this based on your file storage logic
    file_config.get_config_by_id(id).await
}

pub async fn get_database_config(
    pool: &Pool<MySql>,
    id: i64,
) -> TraefikApiResult<TraefikConfigVersion> {
    sqlx::query_as::<_, TraefikConfigVersion>(
        r#"
        SELECT id, name, config, created_at, updated_at, version
        FROM config_versions 
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| TraefikApiError::NotFound(format!("Configuration {} not found", id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::test_helpers::init_test_tracing;
    #[allow(unused_imports)]
    use tracing::debug;

    #[tokio::test]
    async fn get_yaml_configs_test() {
        // init_test_tracing();
        let root_dir = env!("CARGO_MANIFEST_DIR");
        let configs = get_yaml_configs(&format!("{}/config", root_dir)).await;
        assert!(configs.is_ok());
        let configs = configs.unwrap();
        assert!(!configs.is_empty());
        let config_names = configs.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        assert!(config_names.contains(&"herringbank.yml".to_string()));
    }
}
