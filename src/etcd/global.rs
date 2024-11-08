use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_middlewares: Option<Vec<String>>,
    #[serde(default = "default_service_key_prefix")]
    pub service_key_prefix: String,
}

fn default_service_key_prefix() -> String {
    "traefik/http".to_string()
}
