use std::path::Path;

use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};
use tokio::fs;
use tracing::error;
use walkdir::WalkDir;

use crate::{
    config::traefik_config::TraefikConfigVersion,
    features::{db, models::SaveConfigRequest, TraefikApiError, TraefikApiResult},
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
        id: u64::MIN, // Special ID for default config
        name: "Default Configuration".to_string(),
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
