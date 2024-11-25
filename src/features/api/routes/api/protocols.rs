use axum::routing::get;
use axum::{Extension, Json, Router};

use crate::features::db;
use crate::features::models::DeploymentProtocol;
use crate::features::routes::ApiContext;

pub fn routes() -> Router {
    Router::new().route("/protocols", get(get_protocols))
}

/// Get all deployment protocols
#[utoipa::path(
  get,
  path = "/api/protocols",
  responses(
      (status = 200, description = "List of deployment protocols", body = Vec<DeploymentProtocol>)
  )
)]
// maybe_auth_user: MaybeAuthUser,
pub(crate) async fn get_protocols(ctx: Extension<ApiContext>) -> Json<Vec<DeploymentProtocol>> {
    Json(
        db::operations::get_deployment_protocols(&ctx.db)
            .await
            .unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use tracing::debug;

    use crate::features::setup_test_server;
    #[allow(unused_imports)]
    use crate::test_helpers::init_test_tracing;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn get_protocols_test() {
        // init_test_tracing();
        let server = setup_test_server().await.unwrap();
        debug!("Server and database setup");
        let _test_user = server.test_user.clone();

        debug!("Testing get protocols");
        let response = server.get("/api/protocols").await;
        debug!("Response: {:?}", response);
        assert_eq!(response.status(), StatusCode::OK);
    }
}
