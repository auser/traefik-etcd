#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporter::{Field, Output, OutputKind, Type, Variant};
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }

    #[test]
    fn test_typescript_struct_export() -> TSTypeResult<()> {
        let temp_dir = create_temp_dir();
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
            ]),
            generics: vec![],
        };

        let exporter = TSExporter::new(output, Some(2));
        exporter.to_file(Some(temp_dir.path().to_path_buf()))?;

        let content = fs::read_to_string(temp_dir.path().join("user.ts"))?;
        assert_eq!(
            content,
            "export interface User {\n  id: number;\n  name: string;\n}"
        );

        Ok(())
    }

    #[test]
    fn test_typescript_enum_export() -> TSTypeResult<()> {
        let temp_dir = create_temp_dir();
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

        let exporter = TSExporter::new(output, Some(2));
        exporter.to_file(Some(temp_dir.path().to_path_buf()))?;

        let content = fs::read_to_string(temp_dir.path().join("status.ts"))?;
        assert_eq!(
            content,
            "export type Status =\n  | \"Active\"\n  | { type: \"Pending\"; reason: string; };"
        );

        Ok(())
    }

    #[test]
    fn test_type_conversion() {
        let exporter = TSExporter::new(
            Output {
                name: "Test".to_string(),
                kind: OutputKind::Struct(vec![]),
                generics: vec![],
            },
            None,
        );

        assert_eq!(exporter.type_to_ts(&Type::String), "string");
        assert_eq!(exporter.type_to_ts(&Type::Number), "number");
        assert_eq!(exporter.type_to_ts(&Type::Boolean), "boolean");
        assert_eq!(
            exporter.type_to_ts(&Type::Array(Box::new(Type::String))),
            "string[]"
        );
        assert_eq!(
            exporter.type_to_ts(&Type::Optional(Box::new(Type::Number))),
            "number | undefined"
        );
        assert_eq!(
            exporter.type_to_ts(&Type::Custom("CustomType".to_string())),
            "CustomType"
        );
    }
}
