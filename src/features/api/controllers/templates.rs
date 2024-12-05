use serde::Deserialize;
use sqlx::{MySql, Pool};
use tracing::{debug, error};
use walkdir::WalkDir;

use crate::{
    config::traefik_config::TraefikConfigVersion,
    features::{file_loader::FileConfig, TemplateInfo, TraefikApiError, TraefikApiResult},
    TraefikConfig,
};

/// List available configuration templates
pub async fn list_templates(file_config: &FileConfig) -> TraefikApiResult<Vec<TemplateInfo>> {
    debug!("Listing templates");
    let templates_dir = &file_config.config_dir;
    let mut templates = Vec::new();

    if templates_dir.exists() && templates_dir.is_dir() {
        for (idx, entry) in WalkDir::new(templates_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "yaml" || ext == "yml")
            })
            .enumerate()
        {
            let path = entry.path();
            let name = path
                .strip_prefix(templates_dir)
                .unwrap_or(path)
                .display()
                .to_string()
                .replace(".yaml", "")
                .replace(".yml", "");

            // Optionally read first line of file for description
            let description = tokio::fs::read_to_string(path)
                .await
                .ok()
                .and_then(|content| {
                    content
                        .lines()
                        .next()
                        .map(|line| line.trim_start_matches('#').trim().to_string())
                });

            let metadata = entry.metadata().expect("Failed to get metadata");
            let updated_at = metadata.modified().unwrap_or(std::time::UNIX_EPOCH).into();
            let created_at = metadata.created().unwrap_or(std::time::UNIX_EPOCH).into();

            templates.push(TemplateInfo {
                id: idx as i64,
                name,
                path: path.display().to_string(),
                description,
                file_template: true,
                updated_at,
                created_at,
            });
        }
    }

    Ok(templates)
}

/// Get a template by path
pub async fn get_template(file_config: &FileConfig, name: &str) -> TraefikApiResult<TraefikConfig> {
    debug!("Getting template: {}", name);
    let templates_dir = &file_config.config_dir;
    debug!("templates_dir: {:?}", templates_dir);
    let template_path = templates_dir.join(format!("{}.yml", name));
    debug!("template_path: {:?}", template_path);

    // Read and parse the template file
    let content = tokio::fs::read_to_string(template_path).await?;

    // Parse the YAML content into a TraefikConfig
    let config: TraefikConfig = match serde_yaml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to parse template: {}", e);
            return Err(TraefikApiError::NotFound(format!(
                "Template not found: {}",
                name
            )));
        }
    };

    Ok(config)
}

#[derive(Debug, Deserialize)]
pub struct SearchTemplatesParams {
    q: Option<String>,
}

impl SearchTemplatesParams {
    pub fn search_term(&self) -> Option<String> {
        self.q.as_deref().map(|s| s.to_string())
    }
}

/// Search for available configuration templates
pub async fn search_templates(
    pool: &Pool<MySql>,
    search_term: Option<String>,
) -> TraefikApiResult<Vec<TraefikConfigVersion>> {
    let query = match &search_term {
        Some(search) => sqlx::query_as::<_, TraefikConfigVersion>(
            r#"
                SELECT 
                    id,
                    name,
                    config,
                    created_at,
                    updated_at,
                    version
                FROM config_versions 
                WHERE name LIKE ?
                ORDER BY created_at DESC
                "#,
        )
        .bind(format!("%{}%", search)),
        None => sqlx::query_as::<_, TraefikConfigVersion>(
            r#"
                SELECT 
                    id,
                    name,
                    config,
                    created_at,
                    updated_at,
                    version
                FROM config_versions 
                ORDER BY created_at DESC
                "#,
        ),
    };

    match query.fetch_all(pool).await {
        Ok(templates) => Ok(templates),
        Err(e) => {
            error!("Error searching templates: {:?}", e);
            Err(TraefikApiError::InternalServerError)
        }
    }
}

pub async fn delete_template(pool: &Pool<MySql>, id: i64) -> TraefikApiResult<()> {
    println!("Deleting template: {}", id);
    let result = sqlx::query(
        r#"
        DELETE FROM config_versions 
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    println!("Rows affected: {}", result.rows_affected());
    if result.rows_affected() == 0 {
        return Err(TraefikApiError::NotFound(format!(
            "Template not found: {}",
            id
        )));
    }

    Ok(())
}
