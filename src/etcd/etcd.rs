use std::{path::PathBuf, time::Duration};

use color_eyre::{
    eyre::{eyre, OptionExt},
    Result,
};
use etcd_client::{
    Certificate, Client, ConnectOptions, DeleteOptions, GetOptions, Identity, KeyValue as KV,
    PutOptions, TlsOptions as ECTlsOptions,
};

use serde::{Deserialize, Serialize};
use tracing::debug;

pub type KeyValue = KV;

#[derive(Clone)]
pub struct Etcd {
    pub client: Client,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EtcdConfig {
    pub endpoints: Vec<String>,
    pub timeout: u64,
    pub keep_alive: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsOptions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsOptions {
    pub domain: Option<String>,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub ca: Option<PathBuf>,
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
                let cert = std::fs::read_to_string(cert)?;
                let key = std::fs::read_to_string(tls.key.as_ref().unwrap())?;
                let ca = std::fs::read_to_string(tls.ca.as_ref().unwrap())?;
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

    pub async fn put(
        &self,
        key: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
        ttl: Option<i64>,
    ) -> Result<Option<KeyValue>> {
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
        Ok(put_rsp.prev_key().cloned())
    }

    pub async fn get(&self, key: impl Into<Vec<u8>>) -> Result<KeyValue> {
        self.client
            .to_owned()
            .get(key, Some(GetOptions::new().with_limit(1)))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .first()
            .cloned()
            .ok_or_eyre("data not found")
    }

    pub async fn get_with_prefix(&self, key: impl Into<Vec<u8>>) -> Result<Vec<KeyValue>> {
        Ok(self
            .client
            .to_owned()
            .get(key, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .to_vec())
    }

    pub async fn get_keys(&self, key: impl Into<Vec<u8>>) -> Result<Vec<KeyValue>> {
        Ok(self
            .client
            .to_owned()
            .get(key, Some(GetOptions::new().with_limit(1).with_from_key()))
            .await
            .map_err(|e| eyre!("etcd get failed: {e}"))?
            .kvs()
            .to_vec())
    }

    #[allow(dead_code)]
    pub async fn delete(&self, key: impl Into<Vec<u8>>) -> Result<i64> {
        Ok(self
            .client
            .to_owned()
            .delete(key, None)
            .await
            .map_err(|e| eyre!("etcd delete failed: {e}"))?
            .deleted())
    }

    pub async fn delete_with_prefix(&self, key: impl Into<Vec<u8>>) -> Result<i64> {
        Ok(self
            .client
            .to_owned()
            .delete(key, Some(DeleteOptions::new().with_prefix()))
            .await
            .map_err(|e| eyre!("etcd delete failed: {e}"))?
            .deleted())
    }

    #[allow(dead_code)]
    pub async fn touch(&self, key: impl Into<Vec<u8>>) -> Result<()> {
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
    pub async fn put_or_touch(
        &self,
        key: &str,
        value: impl Into<Vec<u8>>,
        ttl: Option<i64>,
    ) -> Result<()> {
        let mut client = self.client.clone();
        if let Some(prev) = client
            .get(key, Some(GetOptions::new().with_limit(1)))
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
