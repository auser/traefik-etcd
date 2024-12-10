use export_type::ExportType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        etcd_trait::{EtcdPair, ToEtcdPairs},
        util::format_list_value,
    },
    error::TraefikResult,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct TlsDomainConfig {
    pub main: String,
    pub sans: Vec<String>,
}

impl ToEtcdPairs for TlsDomainConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        pairs.push(EtcdPair::new(
            format!("{}/main", base_key),
            self.main.clone(),
        ));
        pairs.push(EtcdPair::new(
            format!("{}/sans", base_key),
            format_list_value(&self.sans),
        ));
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct TlsConfig {
    pub domains: Vec<TlsDomainConfig>,
}

impl ToEtcdPairs for TlsConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let domains_base_key = format!("{}/domains", base_key);
        for domain in self.domains.iter() {
            let domain_base_key = format!("{}/{}", domains_base_key, domain.main);
            let domain_pairs = domain.to_etcd_pairs(&domain_base_key)?;
            pairs.extend(domain_pairs);
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RedirectionConfig {
    pub to: String,
    pub scheme: Option<String>,
}

impl ToEtcdPairs for RedirectionConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        pairs.push(EtcdPair::new(format!("{}/to", base_key), self.to.clone()));
        if let Some(scheme) = &self.scheme {
            pairs.push(EtcdPair::new(
                format!("{}/scheme", base_key),
                scheme.clone(),
            ));
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct RedirectionsConfig {
    pub entry_point: Option<RedirectionConfig>,
}

impl ToEtcdPairs for RedirectionsConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        if let Some(entry_point) = &self.entry_point {
            let entry_point_base_key = format!("{}/entryPoint", base_key);
            let entry_point_pairs = entry_point.to_etcd_pairs(&entry_point_base_key)?;
            pairs.extend(entry_point_pairs);
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct HttpConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirections: Option<RedirectionsConfig>,
}

impl ToEtcdPairs for HttpConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        if let Some(tls) = &self.tls {
            let tls_base_key = format!("{}/tls", base_key);
            let tls_pairs = tls.to_etcd_pairs(&tls_base_key)?;
            pairs.extend(tls_pairs);
        }
        if let Some(redirections) = &self.redirections {
            let redirections_base_key = format!("{}/redirections", base_key);
            let redirections_pairs = redirections.to_etcd_pairs(&redirections_base_key)?;
            pairs.extend(redirections_pairs);
        }
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct EntryPoint {
    pub address: String,
    pub http: HttpConfig,
}

impl ToEtcdPairs for EntryPoint {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        pairs.push(EtcdPair::new(
            format!("{}/address", base_key),
            self.address.clone(),
        ));
        let http_base_key = format!("{}/http", base_key);
        let http_pairs = self.http.to_etcd_pairs(&http_base_key)?;
        pairs.extend(http_pairs);
        Ok(pairs)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, JsonSchema)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema, sqlx::FromRow))]
#[cfg_attr(feature = "codegen", derive(ExportType))]
#[export_type(rename_all = "camelCase", path = "generated/types")]
pub struct EntryPointsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web: Option<EntryPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub websecure: Option<EntryPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<EntryPoint>,
}

impl ToEtcdPairs for EntryPointsConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = vec![];
        let entrypoints_base_key = format!("{}/entryPoints", base_key);
        if let Some(web) = &self.web {
            let web_base_key = format!("{}/web", entrypoints_base_key);
            let web_pairs = web.to_etcd_pairs(&web_base_key)?;
            pairs.extend(web_pairs);
        }

        if let Some(websecure) = &self.websecure {
            let websecure_base_key = format!("{}/websecure", entrypoints_base_key);
            let websecure_pairs = websecure.to_etcd_pairs(&websecure_base_key)?;
            pairs.extend(websecure_pairs);
        }

        if let Some(metrics) = &self.metrics {
            let metrics_base_key = format!("{}/metrics", entrypoints_base_key);
            let metrics_pairs = metrics.to_etcd_pairs(&metrics_base_key)?;
            pairs.extend(metrics_pairs);
        }

        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::assert_contains_pair;

    use super::*;

    #[test]
    fn test_entry_points_config() {
        let config = EntryPointsConfig {
            web: Some(EntryPoint {
                address: "0.0.0.0:80".to_string(),
                http: HttpConfig {
                    tls: Some(TlsConfig {
                        domains: vec![TlsDomainConfig {
                            main: "ari.io".to_string(),
                            sans: vec!["*.ari.io".to_string()],
                        }],
                    }),
                    redirections: Some(RedirectionsConfig {
                        entry_point: Some(RedirectionConfig {
                            to: "https://ari.io".to_string(),
                            scheme: Some("https".to_string()),
                        }),
                    }),
                },
            }),
            websecure: None,
            metrics: None,
        };
        assert_eq!(config.web.unwrap().address, "0.0.0.0:80");
    }

    #[test]
    fn test_entry_points_config_to_etcd_pairs() {
        let config = create_test_entry_points_config();
        let pairs = config.to_etcd_pairs("test");
        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();
        assert_contains_pair(&pairs, "test/entryPoints/web/address 0.0.0.0:80");
        assert_contains_pair(&pairs, "test/entryPoints/websecure/address 0.0.0.0:443");
        assert_contains_pair(
            &pairs,
            "test/entryPoints/web/http/tls/domains/ari.io/main ari.io",
        );
        assert_contains_pair(
            &pairs,
            "test/entryPoints/websecure/http/tls/domains/ari.io/sans *.ari.io",
        );
    }

    fn create_test_entry_points_config() -> EntryPointsConfig {
        EntryPointsConfig {
            web: Some(EntryPoint {
                address: "0.0.0.0:80".to_string(),
                http: HttpConfig {
                    tls: Some(TlsConfig {
                        domains: vec![TlsDomainConfig {
                            main: "ari.io".to_string(),
                            sans: vec!["*.ari.io".to_string()],
                        }],
                    }),
                    redirections: Some(RedirectionsConfig {
                        entry_point: Some(RedirectionConfig {
                            to: "https://ari.io".to_string(),
                            scheme: Some("https".to_string()),
                        }),
                    }),
                },
            }),
            websecure: Some(EntryPoint {
                address: "0.0.0.0:443".to_string(),
                http: HttpConfig {
                    tls: Some(TlsConfig {
                        domains: vec![TlsDomainConfig {
                            main: "ari.io".to_string(),
                            sans: vec!["*.ari.io".to_string()],
                        }],
                    }),
                    redirections: None,
                },
            }),
            metrics: None,
        }
    }
}
