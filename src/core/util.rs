use crate::error::{TraefikError, TraefikResult};
use color_eyre::eyre::eyre;
use config::Case;
use convert_case::Casing;

pub fn validate_is_alphanumeric(path: &str) -> TraefikResult<()> {
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.')
    {
        return Err(TraefikError::ParsingError(eyre!(format!(
            "Invalid characters in path: {}",
            path
        ))));
    }

    Ok(())
}

pub fn validate_ip(ip: &str) -> TraefikResult<()> {
    // IP address validation
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        let valid = parts.iter().all(|part| {
            if let Ok(_num) = part.parse::<u8>() {
                !part.is_empty() && part.len() <= 3
            } else {
                false
            }
        });
        if !valid {
            return Err(TraefikError::DeploymentConfig(format!(
                "Invalid IP '{}' in deployment",
                ip
            )));
        }
    }
    Ok(())
}

pub fn validate_hostname(hostname: &str) -> TraefikResult<()> {
    fn is_valid_char(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.'
    }

    if hostname.bytes().any(|byte| !is_valid_char(byte))
        || hostname
            .split('.')
            .any(|label| label.is_empty() || label.len() > 63 || label.starts_with('-'))
        || hostname.is_empty()
        || hostname.len() > 255
    {
        return Err(TraefikError::DeploymentConfig(format!(
            "Invalid hostname '{}' in deployment",
            hostname
        )));
    }
    Ok(())
}

#[allow(dead_code)]
pub fn validate_protocol(protocol: &str) -> TraefikResult<()> {
    if protocol != "http" && protocol != "https" {
        return Err(TraefikError::DeploymentConfig(format!(
            "Invalid protocol '{}' in deployment",
            protocol
        )));
    }
    Ok(())
}

pub fn validate_port(port: u16) -> TraefikResult<()> {
    if !(1..=65535).contains(&port) {
        return Err(TraefikError::DeploymentConfig(format!(
            "port must be between 1 and 65535, got {}",
            port
        )));
    }
    Ok(())
}

pub fn format_header_key(key: &str) -> String {
    key.to_case(Case::Pascal)
}

pub fn format_list_value(values: &[String]) -> String {
    values.join(", ")
}

pub fn format_etcd_value(value: &str) -> String {
    if value.is_empty() {
        r#""""#.to_string()
    } else {
        value.to_string()
    }
}

pub fn get_safe_key(key: &str) -> String {
    key.replace(".", "-").replace("/", "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_is_alphanumeric() {
        assert!(validate_is_alphanumeric("/test").is_ok());
    }

    #[test]
    fn test_validate_is_alphanumeric_with_invalid_characters() {
        assert!(validate_is_alphanumeric("$#test/").is_err());
    }

    #[test]
    fn test_format_header_key() {
        assert_eq!(format_header_key("test"), "Test");
    }

    #[test]
    fn test_format_header_key_with_spaces() {
        assert_eq!(format_header_key("test with spaces"), "TestWithSpaces");
    }

    #[test]
    fn test_format_header_key_with_dashes() {
        assert_eq!(format_header_key("test-with-dashes"), "TestWithDashes");
    }

    #[test]
    fn test_format_header_key_with_underscores() {
        assert_eq!(
            format_header_key("test_with_underscores"),
            "TestWithUnderscores"
        );
    }

    #[test]
    fn test_format_list_value() {
        assert_eq!(
            format_list_value(&["test".to_string(), "test2".to_string()]),
            "test, test2"
        );
    }

    #[test]
    fn test_get_safe_key() {
        assert_eq!(get_safe_key("test.com"), "test-com");
    }
}
