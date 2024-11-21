use serde::{Deserialize, Serialize};

use crate::features::etcd;

use super::host::HostConfig;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TraefikConfig {
    #[cfg(feature = "etcd")]
    pub etcd: etcd::EtcdConfig,
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
}
