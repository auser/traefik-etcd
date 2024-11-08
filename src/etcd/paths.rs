use serde::{Deserialize, Serialize};

use crate::common::{
    error::TraefikResult,
    etcd::{EtcdPair, ToEtcdPairs},
};

use super::util::get_safe_key;

#[derive(Serialize, Deserialize, Debug)]
pub struct PathConfig {
    pub path: String,
    #[serde(default)]
    pub strip_prefix: bool,
    #[serde(default)]
    pub middlewares: Option<Vec<String>>,
    #[serde(default)]
    pub entrypoints: Option<Vec<String>>,
}

impl ToEtcdPairs for PathConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];

        let backend_key = format!("{}", base_key);

        if let Some(middlewares) = &self.middlewares {
            for (i, middleware) in middlewares.iter().enumerate() {
                pairs.push(EtcdPair::new(
                    format!("{}/middlewares/{}", backend_key, i),
                    middleware.clone(),
                ));
            }
        }

        pairs.push(EtcdPair::new(
            format!("{}/path", base_key),
            self.path.clone(),
        ));
        Ok(pairs)
    }
}
