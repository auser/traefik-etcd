use axum::{extract::Path, routing::get, Extension, Json, Router};

use crate::{
    features::{controllers, routes::ApiContext, TemplateInfo, TraefikApiResult},
    TraefikConfig,
};

pub fn routes() -> Router {
    Router::new()
        .route("/templates/:path", get(get_template))
        .route("/templates", get(list_templates))
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
    let templates = controllers::templates::list_templates(&ctx.file_config).await?;
    Ok(Json(templates))
}

#[utoipa::path(
  get,
  path = "/api/templates/{path}",
  responses(
      (status = 200, description = "Template content", body = TraefikConfig),
      (status = 404, description = "Template not found")
  ),
  tags = ["templates"]
)]
pub(crate) async fn get_template(
    ctx: Extension<ApiContext>,
    Path(name): Path<String>,
) -> TraefikApiResult<Json<TraefikConfig>> {
    let content = controllers::templates::get_template(&ctx.file_config, &name).await?;
    Ok(Json(content))
}
