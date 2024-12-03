#![allow(dead_code)]

use crate::error::TSTypeResult;
use crate::exporter::{Output, OutputKind, ToOutput, Type};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// Track all types for single file generation, using HashSet to prevent duplicates
static COLLECTED_TYPES: Lazy<Mutex<HashMap<PathBuf, HashSet<Output>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct TSExporter {
    output: Output,
    indent: usize,
    generics: Vec<String>,
}

impl TSExporter {
    pub fn new(output: Output, indent: Option<usize>, generics: Vec<String>) -> Self {
        Self {
            output,
            indent: indent.unwrap_or(2),
            generics,
        }
    }

    fn type_to_ts(&self, ty: &Type) -> String {
        match ty {
            Type::String => "string".to_string(),
            Type::Number => "number".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::HashMap(key, value) => {
                format!(
                    "Record<{}, {}>",
                    self.type_to_ts(key),
                    self.type_to_ts(value)
                )
            }
            Type::Array(inner) => format!("{}[]", self.type_to_ts(inner)),
            Type::Optional(inner) => format!("{} | undefined", self.type_to_ts(inner)),
            Type::Custom(name) => name.clone(),
        }
    }

    fn format_generic_params(&self) -> String {
        if self.output.generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", self.output.generics.join(", "))
        }
    }

    fn generate_type_definition(&self) -> String {
        let generic_params = self.format_generic_params();
        match &self.output.kind {
            OutputKind::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|f| {
                        format!(
                            "    {}{}: {};",
                            f.name,
                            if f.optional { "?" } else { "" },
                            self.type_to_ts(&f.ty)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "export interface {}{} {{\n{}\n}}",
                    self.output.name, generic_params, fields
                )
            }
            OutputKind::Enum(variants) => {
                let variants = variants
                    .iter()
                    .map(|v| match &v.fields {
                        None => format!("    | \"{}\"", v.name),
                        Some(fields) => {
                            let fields = fields
                                .iter()
                                .map(|f| format!("    {}: {};", f.name, self.type_to_ts(&f.ty)))
                                .collect::<Vec<_>>()
                                .join("\n");
                            format!("    | {{ type: \"{}\"; {} }}", v.name, fields)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "export type {}{} =\n{};",
                    self.output.name, generic_params, variants
                )
            }
        }
    }

    pub fn write_single_file(path: &PathBuf) -> TSTypeResult<()> {
        let collected = COLLECTED_TYPES.lock().unwrap();
        if let Some(types) = collected.get(path) {
            let mut content = String::new();

            // Convert HashSet to Vec for sorting
            let mut sorted_types: Vec<_> = types.iter().cloned().collect();
            sorted_types.sort_by(|a, b| a.name.cmp(&b.name));

            for output in sorted_types {
                let exporter = TSExporter::new(output, None, vec![]);
                content.push_str(&exporter.generate_type_definition());
                content.push_str("\n\n");
            }

            fs::create_dir_all(path.parent().unwrap_or(&PathBuf::from("")))?;
            fs::write(path.join("types.ts"), content.trim())?;
        }
        Ok(())
    }
}

impl ToOutput for TSExporter {
    fn to_output(&self) -> proc_macro2::TokenStream {
        quote::quote! {}
    }

    fn to_file(&self, path: Option<PathBuf>) -> TSTypeResult<()> {
        let path = path.unwrap_or_else(|| PathBuf::from("generated"));

        // Add type to collection using HashSet
        COLLECTED_TYPES
            .lock()
            .unwrap()
            .entry(path.clone())
            .or_default()
            .insert(self.output.clone());

        // Write single file
        Self::write_single_file(&path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{exporter::Field, RenameRule};
    use tempfile::TempDir;

    #[test]
    fn test_typescript_generic_struct() -> TSTypeResult<()> {
        let temp_dir = TempDir::new()?;
        let output = Output {
            lang: "typescript".to_string(),
            rename_all: Some(RenameRule::CamelCase),
            name: "Container".to_string(),
            kind: OutputKind::Struct(vec![Field {
                name: "data".to_string(),
                ty: Type::Custom("T".to_string()),
                optional: false,
            }]),
            generics: vec!["T".to_string()],
            export_path: Some(temp_dir.path().to_path_buf()),
        };

        let exporter = TSExporter::new(output, Some(2), vec![]);
        exporter.to_file(Some(temp_dir.path().to_path_buf()))?;

        let content = fs::read_to_string(temp_dir.path().join("types.ts"))?;
        assert_eq!(content, "export interface Container<T> {\n    data: T;\n}");

        Ok(())
    }
}
