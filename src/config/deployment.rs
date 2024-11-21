use serde::{Deserialize, Serialize};

use super::selections::SelectionConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentConfig {
    pub ip: String,
    pub port: u16,
    #[serde(default)]
    pub weight: u8,
    #[serde(default, flatten)]
    pub selection: Option<SelectionConfig>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "http".to_string()
}
