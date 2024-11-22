use crate::error::{TraefikError, TraefikResult};

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
}
