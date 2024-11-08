use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryPointConfig(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct RouterConfig {
    pub(crate) rule: String,
    pub(crate) service: String,
    pub(crate) middlewares: Vec<String>,
    pub(crate) entry_points: Vec<EntryPointConfig>,
    pub(crate) tls: bool,
    pub(crate) priority: Option<u32>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            rule: String::new(),
            service: String::new(),
            middlewares: Vec::new(),
            entry_points: vec![EntryPointConfig("websecure".to_string())],
            tls: true,
            priority: None,
        }
    }
}
