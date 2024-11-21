use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SelectionConfig {
    #[serde(default)]
    pub with_cookie: Option<WithCookieConfig>,
    #[serde(default)]
    pub from_client_ip: Option<FromClientIpConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WithCookieConfig {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
}

impl Default for WithCookieConfig {
    fn default() -> Self {
        Self {
            name: "TEST_COOKIE".to_string(),
            value: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FromClientIpConfig {
    pub range: Option<String>,
    pub ip: Option<String>,
}
