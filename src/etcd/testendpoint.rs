use serde::{Deserialize, Serialize};

use super::backend::BackendConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct TestConfig {
    pub enabled: bool,
    pub cookie: TestCookieConfig,
    pub backend: BackendConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestCookieConfig {
    pub enabled: bool,
    pub name: String,
    pub backends: TestBackends,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestBackends {
    pub yellow: BackendConfig,
    pub green: BackendConfig,
}
