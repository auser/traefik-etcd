#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TSTypeResult;

    #[test]
    fn test_output_struct() {
        let output = Output {
            name: "User".to_string(),
            kind: OutputKind::Struct(vec![
                Field {
                    name: "id".to_string(),
                    ty: Type::Number,
                    optional: false,
                },
                Field {
                    name: "name".to_string(),
                    ty: Type::String,
                    optional: false,
                },
                Field {
                    name: "age".to_string(),
                    ty: Type::Optional(Box::new(Type::Number)),
                    optional: true,
                },
            ]),
            generics: vec![],
        };

        assert_eq!(output.name, "User");
        if let OutputKind::Struct(fields) = output.kind {
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].name, "id");
            assert!(matches!(fields[0].ty, Type::Number));
            assert!(!fields[0].optional);
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_output_enum() {
        let output = Output {
            name: "Status".to_string(),
            kind: OutputKind::Enum(vec![
                Variant {
                    name: "Active".to_string(),
                    fields: None,
                },
                Variant {
                    name: "Pending".to_string(),
                    fields: Some(vec![Field {
                        name: "reason".to_string(),
                        ty: Type::String,
                        optional: false,
                    }]),
                },
            ]),
            generics: vec![],
        };

        assert_eq!(output.name, "Status");
        if let OutputKind::Enum(variants) = output.kind {
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Active");
            assert!(variants[0].fields.is_none());
            assert_eq!(variants[1].name, "Pending");
            assert!(variants[1].fields.is_some());
        } else {
            panic!("Expected Enum");
        }
    }
}
