use convert_case::{Case, Casing};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RenameRule {
    CamelCase,
    PascalCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
}

impl RenameRule {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "camelCase" => Some(Self::CamelCase),
            "PascalCase" => Some(Self::PascalCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "kebab-case" => Some(Self::KebabCase),
            _ => None,
        }
    }

    pub fn apply(&self, name: &str) -> String {
        match self {
            Self::CamelCase => name.to_case(Case::Camel),
            Self::PascalCase => name.to_case(Case::Pascal),
            Self::SnakeCase => name.to_case(Case::Snake),
            Self::ScreamingSnakeCase => name.to_case(Case::ScreamingSnake),
            Self::KebabCase => name.to_case(Case::Kebab),
        }
    }
}

impl From<RenameRule> for String {
    fn from(rule: RenameRule) -> Self {
        rule.apply("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_conversions() {
        let test_cases = vec![
            (
                "user_id", "userId", "UserId", "user_id", "USER_ID", "user-id",
            ),
            (
                "UserName",
                "userName",
                "UserName",
                "user_name",
                "USER_NAME",
                "user-name",
            ),
            ("ID", "id", "Id", "id", "ID", "id"),
            (
                "HTTPResponse",
                "httpResponse",
                "HttpResponse",
                "http_response",
                "HTTP_RESPONSE",
                "http-response",
            ),
        ];

        for (input, camel, pascal, snake, screaming, kebab) in test_cases {
            assert_eq!(RenameRule::CamelCase.apply(input), camel);
            assert_eq!(RenameRule::PascalCase.apply(input), pascal);
            assert_eq!(RenameRule::SnakeCase.apply(input), snake);
            assert_eq!(RenameRule::ScreamingSnakeCase.apply(input), screaming);
            assert_eq!(RenameRule::KebabCase.apply(input), kebab);
        }
    }
}
