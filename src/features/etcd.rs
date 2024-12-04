use async_trait::async_trait;
use color_eyre::eyre::{eyre, Result};
use etcd_client::{
    Certificate, Client, ConnectOptions, DeleteOptions, GetOptions, Identity, PutOptions,
    TlsOptions as ECTlsOptions,
};
use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

mod diff;

pub use diff::*;

use crate::{
    core::client::StoreClientActor,
    error::{TraefikError, TraefikResult},
};

use super::KeyValue;

/// The configuration for the etcd client
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "src/generated/types")]
pub struct EtcdConfig {
    pub endpoints: Vec<String>,
    pub timeout: u64,
    pub keep_alive: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsOptions>,
}

/// The configuration for the TLS options
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "src/generated/types")]
pub struct TlsOptions {
    pub domain: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub ca: Option<String>,
}

impl Default for EtcdConfig {
    fn default() -> Self {
        Self {
            endpoints: vec!["http://127.0.0.1:2379".to_owned()],
            timeout: 2000,
            keep_alive: 300,
            tls: None,
        }
    }
}

impl From<String> for EtcdConfig {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

#[derive(Clone)]
pub struct Etcd {
    pub client: Client,
}

impl std::fmt::Debug for Etcd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Etcd {{ client: <hidden> }}")
    }
}

#[async_trait]
impl StoreClientActor for Etcd {
    async fn put(
        &self,
        key: impl Into<Vec<u8>> + Send,
        value: impl Into<Vec<u8>> + Send,
        ttl: Option<i64>,
    ) -> TraefikResult<Option<KeyValue>> {
        let mut client = self.client.clone();
        let ttl = ttl.unwrap_or(0);
        let option = if ttl == 0 {
            PutOptions::new().with_prev_key()
        } else {
            let lease = client
                .lease_grant(ttl, None)
                .await
                .map_err(|e| eyre!("etcd lease_grant failed: {e}"))?;
            PutOptions::new().with_lease(lease.id()).with_prev_key()
        };
        let put_rsp = client
            .put(key, value, Some(option))
            .await
            .map_err(|e| eyre!("etcd put failed: {e}"))?;
        Ok(put_rsp.prev_key().cloned().map(Into::into))
    }

    async fn get(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<KeyValue> {
        self.client
            .to_owned()
            .get(key, Some(GetOptions::new().with_limit(1)))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .first()
            .cloned()
            .ok_or_else(|| TraefikError::NotFound("data not found".into()))
            .map(Into::into)
    }

    async fn get_with_prefix(
        &self,
        key: impl Into<Vec<u8>> + Send,
    ) -> TraefikResult<Vec<KeyValue>> {
        Ok(self
            .client
            .to_owned()
            .get(key, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .iter()
            .cloned()
            .map(Into::into)
            .collect())
    }

    async fn get_keys(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<Vec<KeyValue>> {
        Ok(self
            .client
            .to_owned()
            .get(key, Some(GetOptions::new().with_limit(1).with_from_key()))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .iter()
            .cloned()
            .map(Into::into)
            .collect())
    }

    async fn delete(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<i64> {
        Ok(self
            .client
            .to_owned()
            .delete(key, None)
            .await
            .map_err(|e| eyre!("etcd delete failed: {e}"))?
            .deleted())
    }

    async fn delete_with_prefix(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<i64> {
        Ok(self
            .client
            .to_owned()
            .delete(key, Some(DeleteOptions::new().with_prefix()))
            .await
            .map_err(|e| eyre!("etcd delete failed: {e}"))?
            .deleted())
    }

    #[allow(dead_code)]
    async fn touch(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<()> {
        let mut client = self.client.clone();
        let lease = client
            .get(key, Some(GetOptions::new().with_limit(1)))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .first()
            .map(|kv| kv.lease())
            .unwrap_or(0);
        if lease != 0 {
            client
                .lease_keep_alive(lease)
                .await
                .map_err(|e| eyre!("etcd lease_keep_alive failed: {e}"))?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    async fn put_or_touch(
        &self,
        key: impl Into<Vec<u8>> + Send,
        value: impl Into<Vec<u8>> + Send,
        ttl: Option<i64>,
    ) -> TraefikResult<()> {
        let mut client = self.client.clone();
        let key = key.into();
        if let Some(prev) = client
            .get(key.clone(), Some(GetOptions::new().with_limit(1)))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .first()
        {
            client
                .lease_keep_alive(prev.lease())
                .await
                .map_err(|e| eyre!("etcd lease_keep_alive failed: {e}"))?;
        } else {
            self.put(key, value, ttl).await?;
        }
        Ok(())
    }
}

impl Etcd {
    pub async fn new(config: &EtcdConfig) -> Result<Self> {
        debug!("Connecting to etcd with config: {:?}", config);
        let mut connect_options = ConnectOptions::new()
            .with_connect_timeout(Duration::from_millis(config.timeout))
            .with_keep_alive(
                Duration::from_secs(config.keep_alive),
                Duration::from_millis(config.timeout),
            )
            .with_keep_alive_while_idle(true)
            .with_timeout(Duration::from_millis(config.timeout));

        if let Some(tls) = &config.tls {
            if let Some(cert) = &tls.cert {
                // --cacert=/etc/etcd/ca.pem --cert=/etc/etcd/server.pem --key=/etc/etcd/server-key.pem"
                let cert = std::fs::read_to_string(cert.trim())?;
                let key = std::fs::read_to_string(tls.key.as_ref().unwrap().trim())?;
                let ca = std::fs::read_to_string(tls.ca.as_ref().unwrap().trim())?;
                let domain = tls.domain.clone().unwrap_or_default();

                let ca = Certificate::from_pem(ca);
                let cert = Certificate::from_pem(cert);
                let key = Identity::from_pem(cert, key);
                let tls_config = ECTlsOptions::new()
                    .ca_certificate(ca)
                    .identity(key)
                    .domain_name(domain);

                connect_options = connect_options.with_tls(tls_config);
            }
        }

        let client = Client::connect(&config.endpoints, Some(connect_options))
            .await
            .map_err(|e| eyre!("etcd connect failed: {e}"))?;
        Ok(Self { client })
    }
}

impl EtcdConfig {
    pub fn merge(self, other: PartialEtcdConfig) -> Self {
        Self {
            endpoints: other.endpoints.unwrap_or(self.endpoints),
            timeout: other.timeout.unwrap_or(self.timeout),
            keep_alive: other.keep_alive.unwrap_or(self.keep_alive),
            tls: other.tls.or(self.tls),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(default)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "src/generated/types")]
pub struct PartialEtcdConfig {
    pub endpoints: Option<Vec<String>>,
    pub timeout: Option<u64>,
    pub keep_alive: Option<u64>,
    pub tls: Option<TlsOptions>,
}

impl From<String> for PartialEtcdConfig {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etcd_config_default_values() {
        let config = EtcdConfig::default();
        assert_eq!(config.endpoints, vec!["http://127.0.0.1:2379".to_string()]);
    }

    #[tokio::test]
    async fn test_etcd_new_returns_error_if_connect_fails() {
        let config = EtcdConfig {
            endpoints: vec!["http://1.2.3.4:2380".to_string()],
            ..Default::default()
        };
        let result = Etcd::new(&config).await;
        assert!(result.is_ok());
    }
}
