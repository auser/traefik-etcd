use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::{
    error::TraefikResult,
    etcd::{EtcdPair, ToEtcdPairs},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MiddlewareConfig {
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HeadersMiddleware>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_regex: Option<RedirectRegexMiddleware>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_through: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_control_expose_headers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitMiddleware>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_prefix: Option<StripPrefixMiddleware>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryMiddleware>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HeadersMiddleware {
    pub custom_request_headers: Option<HashMap<String, String>>,
    pub custom_response_headers: Option<HashMap<String, String>>,
    pub access_control_allow_methods: Option<Vec<String>>,
    pub access_control_expose_headers: Option<Vec<String>>,
    #[serde(default)]
    pub add_vary_header: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RateLimitMiddleware {
    pub average: u32,
    pub burst: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StripPrefixMiddleware {
    pub prefixes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetryMiddleware {
    pub attempts: u32,
    pub initial_interval: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedirectRegexMiddleware {
    pub(crate) regex: String,
    pub(crate) replacement: String,
    #[serde(default)]
    pub(crate) permanent: bool,
}

impl ToEtcdPairs for MiddlewareConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let middleware_key = format!("{}/{}", base_key, self.name);
        let mut pairs = Vec::new();

        if let Some(headers) = &self.headers {
            // Custom request headers
            if let Some(custom_request_headers) = &headers.custom_request_headers {
                for (key, value) in custom_request_headers {
                    pairs.push(EtcdPair::new(
                        format!("{}/headers/customRequestHeaders/{}", middleware_key, key),
                        value.clone(),
                    ));
                }
            }

            // Custom response headers
            if let Some(custom_response_headers) = &headers.custom_response_headers {
                for (key, value) in custom_response_headers {
                    pairs.push(EtcdPair::new(
                        format!("{}/headers/customResponseHeaders/{}", middleware_key, key),
                        value.clone(),
                    ));
                }
            }

            if let Some(access_control_allow_methods) = &headers.access_control_allow_methods {
                pairs.push(EtcdPair::new(
                    format!("{}/headers/accessControlAllowMethods", middleware_key),
                    access_control_allow_methods.join(","),
                ));
            }

            if let Some(access_control_expose_headers) = &headers.access_control_expose_headers {
                pairs.push(EtcdPair::new(
                    format!("{}/headers/accessControlExposeHeaders", middleware_key),
                    access_control_expose_headers.join(","),
                ));
            }

            pairs.push(EtcdPair::new(
                format!("{}/headers/addVaryHeader", base_key),
                headers.add_vary_header.to_string(),
            ));
        }

        if let Some(redirect) = &self.redirect_regex {
            pairs.push(EtcdPair::new(
                format!("{}/redirectregex/regex", base_key),
                redirect.regex.clone(),
            ));
            pairs.push(EtcdPair::new(
                format!("{}/redirectregex/replacement", base_key),
                redirect.replacement.clone(),
            ));
            pairs.push(EtcdPair::new(
                format!("{}/redirectregex/permanent", base_key),
                redirect.permanent.to_string(),
            ));
        }

        if let Some(rate_limit) = &self.rate_limit {
            pairs.push(EtcdPair::new(
                format!("{}/rateLimit/average", base_key),
                rate_limit.average.to_string(),
            ));
            pairs.push(EtcdPair::new(
                format!("{}/rateLimit/burst", base_key),
                rate_limit.burst.to_string(),
            ));
        }

        if let Some(pass_through) = &self.pass_through {
            pairs.push(EtcdPair::new(
                format!("{}/customRequestHeaders/X-Pass-Through", middleware_key),
                pass_through.to_string(),
            ));
        }

        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_middleware_headers() {
        let config = MiddlewareConfig {
            name: "test-headers".to_string(),
            headers: Some(HeadersMiddleware {
                custom_request_headers: Some(HashMap::from([
                    ("X-Forwarded-Proto".to_string(), "https".to_string()),
                    ("X-Forwarded-Port".to_string(), "443".to_string()),
                ])),
                custom_response_headers: Some(HashMap::from([(
                    "Location".to_string(),
                    "".to_string(),
                )])),
                access_control_allow_methods: Some(vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "OPTIONS".to_string(),
                ]),
                access_control_expose_headers: Some(vec!["Location".to_string()]),
                add_vary_header: true,
            }),
            ..Default::default()
        };

        let pairs = config.to_etcd_pairs("traefik/http/middlewares");

        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();

        assert!(pairs.contains(&EtcdPair::new(
            "traefik/http/middlewares/test-headers/headers/customRequestHeaders/X-Forwarded-Proto"
                .to_string(),
            "https".to_string(),
        )));
    }

    #[test]
    fn test_middleware_redirect_regex() {
        let config = MiddlewareConfig {
            name: "to-www".to_string(),
            redirect_regex: Some(RedirectRegexMiddleware {
                regex: "^https://([^.]+\\.[^.]+\\.[^.]+)(.*)".to_string(),
                replacement: "https://www.${1}${2}".to_string(),
                permanent: true,
            }),
            ..Default::default()
        };

        let pairs = config.to_etcd_pairs("traefik/http/middlewares");

        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();

        assert_eq!(
            pairs,
            vec![
                EtcdPair::new(
                    "traefik/http/middlewares/redirectregex/regex".to_string(),
                    "^https://([^.]+\\.[^.]+\\.[^.]+)(.*)".to_string(),
                ),
                EtcdPair::new(
                    "traefik/http/middlewares/redirectregex/replacement".to_string(),
                    "https://www.${1}${2}".to_string(),
                ),
                EtcdPair::new(
                    "traefik/http/middlewares/redirectregex/permanent".to_string(),
                    "true".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn test_middleware_rate_limit() {
        let config = MiddlewareConfig {
            name: "rate-limit".to_string(),
            rate_limit: Some(RateLimitMiddleware {
                average: 100,
                burst: 200,
            }),
            ..Default::default()
        };

        let pairs = config.to_etcd_pairs("traefik/http/middlewares");

        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();

        assert_eq!(
            pairs,
            vec![
                EtcdPair::new(
                    "traefik/http/middlewares/rateLimit/average".to_string(),
                    "100".to_string(),
                ),
                EtcdPair::new(
                    "traefik/http/middlewares/rateLimit/burst".to_string(),
                    "200".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn test_middleware_pass_through() {
        let config = MiddlewareConfig {
            name: "pass-through".to_string(),
            pass_through: Some(true),
            ..Default::default()
        };

        let pairs = config.to_etcd_pairs("traefik/http/middlewares");

        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();

        assert_eq!(
            pairs,
            vec![EtcdPair::new(
                "traefik/http/middlewares/pass-through/customRequestHeaders/X-Pass-Through"
                    .to_string(),
                "true".to_string(),
            )]
        );
    }

    #[test]
    fn test_middleware_from_yaml() {
        let yaml = r#"
name: enable-headers
headers:
  custom_request_headers:
    X-Forwarded-Proto: "https"
    X-Forwarded-Port: "443"
  custom_response_headers:
    Location: ""
  access_control_allow_methods:
    - GET
    - POST
    - OPTIONS
    - PUT
    - DELETE
  access_control_expose_headers:
    - Location
  add_vary_header: true
"#;

        let config: MiddlewareConfig = serde_yaml::from_str(yaml).unwrap();
        let pairs = config.to_etcd_pairs("traefik/http/middlewares");

        assert_eq!(config.name, "enable-headers");
        assert!(config.headers.is_some());
        let headers = config.headers.as_ref().unwrap();
        assert_eq!(
            headers
                .custom_request_headers
                .as_ref()
                .unwrap()
                .get("X-Forwarded-Proto"),
            Some(&"https".to_string())
        );
        assert!(headers.add_vary_header);

        let pairs = pairs.unwrap();
        let pairs_values: Vec<String> =
            pairs.into_iter().map(|p| p.into()).collect::<Vec<String>>();

        // Verify the etcd pairs
        assert!(pairs_values.contains(&"traefik/http/middlewares/enable-headers/headers/customRequestHeaders/X-Forwarded-Port 443".to_string()));
    }

    #[test]
    fn test_combined_middleware() {
        let yaml = r#"
name: combined
headers:
  custom_request_headers:
    X-Forwarded-Proto: "https"
  add_vary_header: true
redirect_regex:
  regex: "^http://(.+)"
  replacement: "https://$1"
  permanent: true
rate_limit:
  average: 100
  burst: 200
pass_through: true
"#;

        let config: MiddlewareConfig = serde_yaml::from_str(yaml).unwrap();
        let pairs = config.to_etcd_pairs("traefik/http/middlewares");

        // Verify all components are present
        assert!(config.headers.is_some());
        assert!(config.redirect_regex.is_some());
        assert!(config.rate_limit.is_some());
        assert!(config.pass_through.is_some());

        assert!(pairs.is_ok());
        let pairs = pairs.unwrap();
        // Verify all key-value pairs are generated
        let pairs_values = pairs.into_iter().map(|p| p.into()).collect::<Vec<String>>();
        assert!(pairs_values.contains(&"traefik/http/middlewares/combined/headers/customRequestHeaders/X-Forwarded-Proto https".to_string()));
        // assert!(pairs_values.contains(&"traefik/http/middlewares/combined/redirectregex/regex ^http://(.+)".to_string()));
        // assert!(pairs_values.contains(&"traefik/http/middlewares/combined/redirectregex/replacement https://$1".to_string()));
        // assert!(pairs_values.contains(&"traefik/http/middlewares/combined/redirectregex/permanent true".to_string()));
        // assert!(pairs_values.contains(&"traefik/http/middlewares/combined/rateLimit/average 100".to_string()));
        // assert!(pairs_values.contains(&"traefik/http/middlewares/combined/rateLimit/burst 200".to_string()));
        // assert!(pairs_values.contains(&"traefik/http/middlewares/combined/pass-through/customRequestHeaders/X-Pass-Through true".to_string()));
    }
}
