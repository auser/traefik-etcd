use crate::error::{TraefikError, TraefikResult};
use config::Case;
use convert_case::Casing;

pub fn validate_is_alphanumeric(path: &str) -> TraefikResult<()> {
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.')
    {
        return Err(TraefikError::ParseError(format!(
            "Invalid characters in path: {}",
            path
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
}
