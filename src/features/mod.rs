#[derive(Debug, Clone)]
pub struct KeyValue {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[cfg(feature = "etcd")]
pub mod etcd;
#[cfg(feature = "etcd")]
pub use etcd_client::KeyValue as KV;
use serde::de::DeserializeOwned;

use crate::error::{TraefikError, TraefikResult};

impl From<KV> for KeyValue {
    fn from(kv: KV) -> Self {
        Self {
            key: kv.key().to_vec(),
            value: kv.value().to_vec(),
        }
    }
}

impl KeyValue {
    pub fn key_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.key).ok()
    }

    pub fn value_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.value).ok()
    }

    pub fn value_json<T: DeserializeOwned>(&self) -> TraefikResult<T> {
        serde_json::from_slice(&self.value).map_err(TraefikError::JsonError)
    }
}
