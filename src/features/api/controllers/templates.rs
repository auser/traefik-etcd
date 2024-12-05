use tracing::error;
use walkdir::WalkDir;

use crate::{
    features::{file_loader::FileConfig, TemplateInfo, TraefikApiError, TraefikApiResult},
    TraefikConfig,
};

/// List available configuration templates
pub async fn list_templates(file_config: &FileConfig) -> TraefikApiResult<Vec<TemplateInfo>> {
    let templates_dir = &file_config.config_dir;
    let mut templates = Vec::new();

    if templates_dir.exists() && templates_dir.is_dir() {
        for entry in WalkDir::new(templates_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "yaml" || ext == "yml")
            })
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

            templates.push(TemplateInfo {
                name,
                path: path.display().to_string(),
                description,
            });
        }
    }

    Ok(templates)
}

/// Get a template by path
pub async fn get_template(file_config: &FileConfig, name: &str) -> TraefikApiResult<TraefikConfig> {
    let templates_dir = &file_config.config_dir;
    let template_path = templates_dir.join(format!("{}.yml", name));

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
