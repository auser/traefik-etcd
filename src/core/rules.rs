use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Rule {
    key: String,
    value: String,
    rule_type: RuleType,
}

impl Rule {
    fn new(key: &str, value: &str, rule_type: RuleType) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            rule_type,
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let rule_str = match self.rule_type {
            RuleType::Other => format!("{}(`{}`)", self.key, self.value),
            RuleType::Header => format!("HeaderRegexp(`{}`, `{}`)", self.key, self.value),
            RuleType::Host => format!("Host(`{}`)", self.value),
            RuleType::ClientIp => format!("ClientIP(`{}`)", self.value),
            RuleType::TcpHost => format!("HostSNI(`{}`)", self.value),
        };
        write!(f, "{}", rule_str)
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self::new("", "", RuleType::Other)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuleConfig {
    rules: HashSet<Rule>,
}

impl RuleConfig {
    pub fn add_rule(&mut self, key: &str, value: &str, rule_type: RuleType) {
        let rule = Rule::new(key, value, rule_type);
        self.rules.insert(rule);
    }

    pub fn add_default_rule(&mut self, key: &str, value: &str) {
        self.add_rule(key, value, RuleType::Other);
    }

    pub fn add_header_rule(&mut self, header: &str, value: &str) -> &mut Self {
        self.add_rule(header, value, RuleType::Header);
        self
    }

    pub fn add_client_ip_rule(&mut self, ip: Option<&str>, range: Option<&str>) -> &mut Self {
        if let Some(ip) = ip {
            self.add_rule("ip", ip, RuleType::ClientIp);
        }
        if let Some(range) = range {
            self.add_rule("range", range, RuleType::ClientIp);
        };
        self
    }

    pub fn add_host_rule(&mut self, domain: &str) -> &mut Self {
        self.add_rule("Host", domain, RuleType::Host);
        self
    }

    pub fn add_tcp_rule(&mut self, service: &str) -> &mut Self {
        self.add_rule("HostSNI", service, RuleType::TcpHost);
        self
    }

    pub fn rule_str(&self) -> String {
        // Sort rules to ensure consistent ordering
        let mut rules: Vec<_> = self.rules.iter().collect();
        rules.sort_by_key(|rule| {
            match rule.rule_type {
                RuleType::Host => 0,     // Host rules first
                RuleType::Header => 1,   // Then Header rules
                RuleType::ClientIp => 2, // Then ClientIP rules
                RuleType::TcpHost => 3,  // TCP Host rules
                RuleType::Other => 4,    // Other rules last
            }
        });

        rules
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<String>>()
            .join(" && ")
    }

    // Weight is now determined by the number of rules
    pub fn get_weight(&self) -> usize {
        self.rules.len()
    }
}

/// Rules can be of different types
/// Host rules are used to match the host of the request
/// Header rules are used to match the headers of the request
/// ClientIP rules are used to match the client IP of the request
/// TcpHost rules are used to match the SNI of the request
/// Other rules are used to match other types of rules
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum RuleType {
    Host,
    Header,
    ClientIp,
    TcpHost,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_config_host_valid_with_host_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_host_rule("example.com");
        assert_eq!(rule_config.rule_str(), "Host(`example.com`)");
    }

    #[test]
    fn test_rule_config_client_ip_valid_with_ip_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_client_ip_rule(Some("192.168.1.1"), None);
        assert_eq!(rule_config.rule_str(), "ClientIP(`192.168.1.1`)");
    }

    #[test]
    fn test_rule_config_client_ip_valid_with_range_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(rule_config.rule_str(), "ClientIP(`192.168.1.1/24`)");
    }

    #[test]
    fn test_rule_config_host_and_client_ip_valid() {
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_host_rule("example.com")
            .add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(
            rule_config.rule_str(),
            "Host(`example.com`) && ClientIP(`192.168.1.1/24`)"
        );
    }

    #[test]
    fn test_rule_config_weight_with_two_rules() {
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_host_rule("example.com")
            .add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(rule_config.get_weight(), 2);
    }

    #[test]
    fn test_rule_config_valid_with_header_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_header_rule("X-Forwarded-Proto", "https");
        assert_eq!(
            rule_config.rule_str(),
            "HeaderRegexp(`X-Forwarded-Proto`, `https`)"
        );
    }

    #[test]
    fn test_rule_config_with_valid_header_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_header_rule("X-Forwarded-Proto", "https");
        assert_eq!(
            rule_config.rule_str(),
            "HeaderRegexp(`X-Forwarded-Proto`, `https`)"
        );
    }

    #[test]
    fn test_rule_config_valid_with_other_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_default_rule("key", "value");
        assert_eq!(rule_config.rule_str(), "key(`value`)");
    }

    #[test]
    fn test_rule_config_with_valid_tcp_rule() {
        let mut rule_config = RuleConfig::default();
        rule_config.add_tcp_rule("service");
        assert_eq!(rule_config.rule_str(), "HostSNI(`service`)");
    }

    #[test]
    fn test_rule_get_weight() {
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_host_rule("example.com")
            .add_client_ip_rule(None, Some("192.168.1.1/24"));
        assert_eq!(rule_config.get_weight(), 2);
        let mut rule_config = RuleConfig::default();
        rule_config
            .add_header_rule("X-Forwarded-Proto", "https")
            .add_client_ip_rule(None, Some("192.168.1.1/24"))
            .add_header_rule("X-Forwarded-Port", "443");
        assert_eq!(rule_config.get_weight(), 3);
    }
}
