use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct SelectionConfig {
    #[serde(default)]
    pub with_cookie: Option<WithCookieConfig>,
    #[serde(default)]
    pub from_client_ip: Option<FromClientIpConfig>,
}

/// The configuration for the with cookie selection
/// This is used to select a deployment based on a cookie.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WithCookieConfig {
    /// The name of the cookie
    pub name: String,
    /// The expected value of the cookie
    #[serde(default)]
    pub value: Option<String>,
}

impl Default for WithCookieConfig {
    fn default() -> Self {
        Self {
            name: "TEST_COOKIE".to_string(),
            value: None,
        }
    }
}

/// The configuration for the from client ip selection
/// This is used to select a deployment based on the client's ip address
/// or a range of ip addresses.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FromClientIpConfig {
    /// The range of the ip address
    pub range: Option<String>,
    /// The specific ip address to select
    pub ip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let selection = SelectionConfig::default();
        assert_eq!(selection.with_cookie, None);
        assert_eq!(selection.from_client_ip, None);
    }

    #[test]
    fn test_with_cookie_config_default_values() {
        let with_cookie = WithCookieConfig::default();
        assert_eq!(with_cookie.name, "TEST_COOKIE".to_string());
        assert_eq!(with_cookie.value, None);
    }

    #[test]
    fn test_from_client_ip_config_default_values() {
        let from_client_ip = FromClientIpConfig::default();
        assert_eq!(from_client_ip.range, None);
        assert_eq!(from_client_ip.ip, None);
    }
}
