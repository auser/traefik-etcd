use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthCheckConfig {
    pub path: String,
    #[serde(default = "default_health_check_interval")]
    pub interval: String,
    #[serde(default = "default_health_check_timeout")]
    pub timeout: String,
}

fn default_health_check_interval() -> String {
    "30s".to_string()
}

fn default_health_check_timeout() -> String {
    "5s".to_string()
}
