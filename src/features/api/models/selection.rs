use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::config::selections::{FromClientIpConfig, WithCookieConfig};

// Selection Controllers
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewSelectionRequest {
    pub with_cookie: Option<WithCookieConfig>,
    pub from_client_ip: Option<FromClientIpConfig>,
}
