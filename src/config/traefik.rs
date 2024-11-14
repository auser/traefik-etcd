use tracing::debug;

use crate::{
    config::core_traits::{ToEtcdPairs, Validate},
    error::{TraefikError, TraefikResult},
    etcd::{util::get_safe_key, Etcd},
};
use std::collections::HashSet;

use super::{
    base_structs::{HostConfig, TraefikConfig},
    core_traits::EtcdPair,
};

impl ToEtcdPairs for TraefikConfig {
    fn to_etcd_pairs(&self, base_key: &str) -> TraefikResult<Vec<EtcdPair>> {
        let mut pairs = Vec::new();

        // Initialize base configuration
        pairs.push(EtcdPair::new(format!("{}", base_key), "true".to_string()));

        // Add global middleware configurations
        self.add_default_middlewares(&mut pairs, base_key)?;

        // Add redirector service configuration
        self.add_redirector_service(&mut pairs, base_key)?;

        // Add middleware configurations
        for (name, middleware) in &self.middlewares {
            debug!("Adding middleware: {}", name);
            let middleware_pairs =
                middleware.to_etcd_pairs(&format!("{}/middlewares", base_key))?;
            pairs.extend(middleware_pairs);
        }

        // Add www redirect middleware if enabled
        if self.www_redirect.unwrap_or(false) {
            self.add_www_redirect_middleware(&mut pairs, base_key)?;
        }

        // Add host configurations
        let mut sorted_hosts = self.hosts.clone();
        // Sort in descending order
        sorted_hosts.sort_by_key(|h| h.get_host_weight());
        sorted_hosts.reverse();
        for host in sorted_hosts {
            let host_pairs = host.to_etcd_pairs(base_key)?;
            pairs.extend(host_pairs);

            // Setup www redirect for applicable hosts
            if self.www_redirect.unwrap_or(false) && !host.domain.starts_with("www.") {
                self.add_www_redirect_router(&mut pairs, base_key, &host.domain)?;
            }
        }

        Ok(pairs)
    }
}

impl Validate for TraefikConfig {
    fn validate(&self) -> TraefikResult<()> {
        // Validate middlewares
        let mut middlewares = self.middlewares.clone();
        for (name, middleware) in middlewares.iter_mut() {
            middleware.set_name(name);
            middleware.validate()?;
        }

        // Validate hosts
        let mut domain_set = HashSet::new();
        for host in &self.hosts {
            // Check for duplicate domains
            if !domain_set.insert(&host.domain) {
                return Err(TraefikError::ConfigError(format!(
                    "Duplicate domain found: {}",
                    host.domain
                )));
            }

            // Validate host configuration
            host.validate()?;

            // Validate middleware references
            self.validate_middleware_references(host)?;
        }

        // Validate redirector configuration
        self.redirector.validate()?;

        Ok(())
    }
}

impl TraefikConfig {
    fn add_default_middlewares(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
    ) -> TraefikResult<()> {
        // Enable headers middleware
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/enable-headers/headers/accessControlAllowMethods",
                base_key
            ),
            "GET, POST, OPTIONS, PUT, DELETE".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/middlewares/enable-headers/headers/accessControlAllowHeaders", base_key),
            "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With, Host, Location".to_string(),
        ));

        // Handle redirects middleware
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/handle-redirects/headers/customRequestHeaders/Location",
                base_key
            ),
            "".to_string(),
        ));

        // Add cookie cleanup middleware
        self.add_cookie_cleanup(pairs, base_key)?;

        Ok(())
    }

    fn add_cookie_cleanup(&self, pairs: &mut Vec<EtcdPair>, base_key: &str) -> TraefikResult<()> {
        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/cookie-cleanup/headers/customResponseHeaders/Set-Cookie",
                base_key
            ),
            "BLUEGREEN=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!(
                "{}/middlewares/cookie-cleanup/headers/customResponseHeaders/Set-Cookie",
                base_key
            ),
            "GREENSRV=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT".to_string(),
        ));

        Ok(())
    }

    pub async fn initialize(&self, etcd: &Etcd) -> TraefikResult<()> {
        // Delete existing configuration
        etcd.delete_with_prefix("traefik").await?;

        // Add base traefik key
        etcd.put("traefik", "true", None).await?;

        // Set up initial configuration
        let mut pairs = Vec::new();

        // Add default middlewares
        self.add_default_middlewares(&mut pairs, "traefik/http")?;

        // Add redirector service
        self.add_redirector_service(&mut pairs, "traefik/http")?;

        // Apply configuration
        for pair in pairs {
            etcd.put(pair.key(), pair.value(), None).await?;
        }

        Ok(())
    }

    fn add_redirector_service(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
    ) -> TraefikResult<()> {
        // Basic service configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/servers/0/url",
                base_key
            ),
            self.redirector.url.clone(),
        ));

        // passHostHeader configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/passHostHeader",
                base_key
            ),
            "true".to_string(),
        ));

        // Response forwarding configuration
        pairs.push(EtcdPair::new(
            format!(
                "{}/services/redirector/loadBalancer/responseForwarding/flushInterval",
                base_key
            ),
            "100ms".to_string(),
        ));

        // Health check configuration
        let health_check = &self.redirector.health_check;
        health_check.to_etcd_pairs(base_key)?;
        // pairs.push(EtcdPair::new(
        //     format!(
        //         "{}/services/redirector/loadBalancer/healthCheck/path",
        //         base_key
        //     ),
        //     self.redirector.health_check.path.clone(),
        // ));

        // pairs.push(EtcdPair::new(
        //     format!(
        //         "{}/services/redirector/loadBalancer/healthCheck/interval",
        //         base_key
        //     ),
        //     self.redirector.health_check.interval.clone(),
        // ));

        // pairs.push(EtcdPair::new(
        //     format!(
        //         "{}/services/redirector/loadBalancer/healthCheck/timeout",
        //         base_key
        //     ),
        //     self.redirector.health_check.timeout.clone(),
        // ));

        Ok(())
    }

    fn add_www_redirect_router(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
        domain: &str,
    ) -> TraefikResult<()> {
        let safe_name = format!("to-www-{}", get_safe_key(domain));

        // Add router to catch non-www version
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/rule", base_key, safe_name),
            format!("Host(`{}`)", domain),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/entrypoints/0", base_key, safe_name),
            "websecure".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/middlewares/0", base_key, safe_name),
            "add-www".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/tls", base_key, safe_name),
            "true".to_string(),
        ));

        // Set higher priority for the redirect router
        pairs.push(EtcdPair::new(
            format!("{}/routers/{}/priority", base_key, safe_name),
            "200".to_string(),
        ));

        Ok(())
    }

    fn add_www_redirect_middleware(
        &self,
        pairs: &mut Vec<EtcdPair>,
        base_key: &str,
    ) -> TraefikResult<()> {
        // Add the basic www redirect middleware configuration
        pairs.push(EtcdPair::new(
            format!("{}/middlewares/add-www/redirectregex/permanent", base_key),
            "true".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/middlewares/add-www/redirectregex/regex", base_key),
            "^https://([^.]+\\.[^.]+\\.[^.]+)(.*)".to_string(),
        ));

        pairs.push(EtcdPair::new(
            format!("{}/middlewares/add-www/redirectregex/replacement", base_key),
            "https://www.${1}${2}".to_string(),
        ));

        Ok(())
    }

    pub async fn apply_to_etcd(&mut self, etcd: &mut Etcd, dry_run: bool) -> TraefikResult<()> {
        // First validate the configuration
        self.validate()?;

        // Generate all etcd pairs
        let pairs = self.to_etcd_pairs("traefik/http")?;

        if dry_run {
            // Just print what would be applied
            for pair in pairs {
                println!("Would set: {} = {}", pair.key(), pair.value());
            }
            return Ok(());
        }

        // Clean existing configuration first
        self.clean_etcd(etcd, false).await?;

        // Apply all pairs to etcd
        for pair in pairs {
            etcd.put(pair.key(), pair.value(), None)
                .await
                .map_err(|e| TraefikError::EtcdError(e.into()))?;
        }

        Ok(())
    }

    pub async fn clean_etcd(&self, etcd: &mut Etcd, all: bool) -> TraefikResult<()> {
        if all {
            etcd.delete_with_prefix("traefik/http").await?;
        } else {
            for host in &self.hosts {
                host.clean_etcd(etcd).await?;
            }
            // Clean middleware configuration if needed
            if !self.middlewares.is_empty() {
                etcd.delete_with_prefix("traefik/http/middlewares").await?;
            }
        }
        Ok(())
    }

    pub fn validate(&mut self) -> TraefikResult<()> {
        // Validate middlewares
        for (name, middleware) in self.middlewares.iter_mut() {
            middleware.set_name(name);
            middleware.validate()?;
        }

        // Validate hosts
        let mut domain_set = HashSet::new();
        for host in &self.hosts {
            // Check for duplicate domains
            let host_rule = host.get_host_rule();
            let host_rule_str = String::from(host_rule.rule_str());
            if !domain_set.insert(host_rule_str) {
                return Err(TraefikError::ConfigError(format!(
                    "Duplicate domain: {}",
                    host.domain
                )));
            }

            // Validate host configuration
            host.validate()?;

            // Validate middleware references
            self.validate_middleware_references(host)?;
        }

        // Validate redirector configuration
        self.redirector.validate()?;

        Ok(())
    }

    fn validate_middleware_references(&self, host: &HostConfig) -> TraefikResult<()> {
        // Check host middlewares
        for middleware_name in &host.middlewares {
            if !self.middlewares.contains_key(middleware_name) {
                return Err(TraefikError::MiddlewareConfig(format!(
                    "Undefined middleware '{}' referenced in host '{}'",
                    middleware_name, host.domain
                )));
            }
        }

        // Check path middlewares
        for path in &host.paths {
            for middleware_name in &path.middlewares {
                if !self.middlewares.contains_key(middleware_name) {
                    return Err(TraefikError::MiddlewareConfig(format!(
                        "Undefined middleware '{}' referenced in path '{}' of host '{}'",
                        middleware_name, path.path, host.domain
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{config::WithCookieConfig, test_helpers::create_test_config};

    #[test]
    fn test_default_middleware_generation() {
        let config = create_test_config(None);
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify default headers middleware
        let has_default_headers = pairs.iter().any(|p| {
            p.key().contains("enable-headers")
                && p.key().contains("accessControlAllowMethods")
                && p.value().contains("GET, POST")
        });
        assert!(has_default_headers, "Default headers middleware not found");

        // Verify redirect handler
        let has_redirect_handler = pairs
            .iter()
            .any(|p| p.key().contains("handle-redirects") && p.key().contains("Location"));
        assert!(
            has_redirect_handler,
            "Redirect handler middleware not found"
        );
    }

    #[test]
    fn test_host_weight_sorting() {
        let mut host_config_with_cookie = HostConfig::default();
        host_config_with_cookie.domain = "test2.example.com".to_string();
        host_config_with_cookie.with_cookie = Some(WithCookieConfig {
            name: "BLUEGREEN".to_string(),
            value: Some("true".to_string()),
        });
        let config = create_test_config(Some(vec![HostConfig::default(), host_config_with_cookie]));
        let mut sorted_hosts = config.hosts.clone();
        sorted_hosts.sort_by_key(|h| h.get_host_weight());
        sorted_hosts.reverse();
        assert_eq!(sorted_hosts[0].domain, "test2.example.com");
    }

    #[test]
    fn test_cookie_cleanup_middleware() {
        let config = create_test_config(None);
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify cookie cleanup headers
        for cookie in ["BLUEGREEN", "GREENSRV"] {
            let has_cookie_cleanup = pairs.iter().any(|p| {
                p.key().contains("cookie-cleanup")
                    && p.value().contains(cookie)
                    && p.value().contains("deleted")
            });
            assert!(
                has_cookie_cleanup,
                "Cookie cleanup for {} not found",
                cookie
            );
        }
    }

    #[test]
    fn test_www_redirect_configuration() {
        let config = create_test_config(None);
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify www redirect middleware
        let has_redirect_regex = pairs.iter().any(|p| {
            p.key().contains("add-www/redirectregex") && p.value().contains("https://www")
        });
        assert!(has_redirect_regex, "WWW redirect middleware not found");

        // Verify router configuration
        let has_redirect_router = pairs.iter().any(|p| {
            p.key().contains("/routers/to-www-") && p.value().contains("Host(`test.example.com`)")
        });
        assert!(has_redirect_router, "WWW redirect router not found");
    }

    #[test]
    fn test_validation_middleware_references() {
        let mut config = create_test_config(None);

        // Add invalid middleware reference
        config.hosts[0].middlewares.push("non-existent".to_string());

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(TraefikError::MiddlewareConfig(msg)) => {
                assert!(msg.contains("Undefined middleware"));
            }
            other => {
                debug!("other: {:?}", other);
                panic!("Expected middleware reference error")
            }
        }
    }

    #[test]
    fn test_validation_duplicate_domains() {
        let mut config = create_test_config(None);

        // Add duplicate domain
        config.hosts.push(config.hosts[0].clone());

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(TraefikError::ConfigError(msg)) => {
                assert!(msg.contains("Duplicate domain"));
            }
            _ => panic!("Expected duplicate domain error"),
        }
    }

    #[test]
    fn test_validation_path_middleware_references() {
        let mut config = create_test_config(None);

        // Add invalid middleware reference to path
        config.hosts[0].paths[0]
            .middlewares
            .push("non-existent".to_string());

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(TraefikError::MiddlewareConfig(msg)) => {
                assert!(msg.contains("Undefined middleware"));
            }
            _ => panic!("Expected middleware reference error"),
        }
    }

    #[test]
    fn test_dry_run() {
        let config = create_test_config(None);
        let pairs = config.to_etcd_pairs("traefik/http").unwrap();

        // Verify all required configuration is generated
        assert!(!pairs.is_empty(), "No pairs generated");
    }
}
