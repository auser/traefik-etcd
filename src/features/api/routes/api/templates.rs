use axum::{
    extract::{Path, Query},
    routing::{delete, get},
    Extension, Json, Router,
};
use tracing::debug;

use crate::{
    config::traefik_config::TraefikConfigVersion,
    features::{
        controllers::{self, templates::SearchTemplatesParams},
        routes::ApiContext,
        TemplateInfo, TraefikApiResult,
    },
    TraefikConfig,
};

pub fn routes() -> Router {
    let router = Router::new()
        .route("/templates/search", get(search_templates))
        .route("/templates/name/:name", get(get_template_route))
        .route("/templates/delete/:id", delete(delete_template))
        .route("/templates", get(list_templates));

    router
}

/// List available configuration templates
#[utoipa::path(
  get,
  path = "/api/templates",
  responses(
      (status = 200, description = "List of available templates", body = Vec<TemplateInfo>)
  ),
  tags = ["templates"]
)]
pub(crate) async fn list_templates(
    ctx: Extension<ApiContext>,
) -> TraefikApiResult<Json<Vec<TemplateInfo>>> {
    debug!("Listing templates route");
    let templates = controllers::templates::list_templates(&ctx.file_config).await?;
    Ok(Json(templates))
}

/// Get a template by name
#[utoipa::path(
  get,
  path = "/api/templates/name/{name}",
  params(
      ("name" = String, Path, description = "Name of the template")
  ),
  responses(
      (status = 200, description = "Template content", body = TraefikConfig),
      (status = 404, description = "Template not found")
  ),
  tags = ["templates"]
)]
pub(crate) async fn get_template_route(
    ctx: Extension<ApiContext>,
    Path(name): Path<String>,
) -> TraefikApiResult<Json<TraefikConfig>> {
    debug!("Getting template: {}", name);
    debug!("Getting template: {}", name);
    debug!("File config path: {:?}", ctx.file_config.config_dir); // Add this
    let content = controllers::templates::get_template(&ctx.file_config, &name).await?;
    debug!("Content: {:?}", content); // Add this
    Ok(Json(content))
}

/// Search for available configuration templates
#[utoipa::path(
    get,
    path = "/api/templates/search",
    params(
        ("search" = Option<String>, Query, description = "Search term for filtering templates")
    ),
    responses(
        (status = 200, description = "List of available templates", body = Vec<TemplateInfo>)
    ),
    tags = ["templates"]
)]
pub(crate) async fn search_templates(
    ctx: Extension<ApiContext>,
    Query(params): Query<SearchTemplatesParams>,
) -> TraefikApiResult<Json<Vec<TraefikConfigVersion>>> {
    debug!(
        "Searching templates with search term: {:?}",
        params.search_term()
    );
    let templates = controllers::templates::search_templates(&ctx.db, params.search_term()).await?;
    Ok(Json(templates))
}

/// Delete a template configuration
#[utoipa::path(
    delete,
    path = "/api/templates/delete/{id}",
    responses(
        (status = 200, description = "Template deleted successfully"),
        (status = 404, description = "Template not found")
    ),
    tags = ["templates"]
)]
pub(crate) async fn delete_template(
    ctx: Extension<ApiContext>,
    Path(id): Path<i64>,
) -> TraefikApiResult<Json<()>> {
    debug!("Deleting template: {}", id);
    controllers::templates::delete_template(&ctx.db, id).await?;
    debug!("Template deleted");
    Ok(Json(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::test_helpers::init_test_tracing;
    #[allow(unused_imports)]
    use tracing::debug;

    #[tokio::test]
    async fn test_list_templates() {
        // init_test_tracing();
    }
}
