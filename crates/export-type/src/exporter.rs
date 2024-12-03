use proc_macro2::TokenStream;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::error::TSTypeResult;
use crate::{get_exporter_from_lang, RenameRule};

pub static GENERATE_STRUCTS_AND_ENUMS: Lazy<Mutex<HashSet<(String, Output)>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

pub fn add_struct_or_enum(name: String, output: Output) -> TSTypeResult<()> {
    GENERATE_STRUCTS_AND_ENUMS
        .lock()
        .unwrap()
        .insert((name, output));
    Ok(())
}

pub fn create_exporter_files(export_path: PathBuf) -> TSTypeResult<()> {
    let mut index_content = String::new();

    for (_name, output) in GENERATE_STRUCTS_AND_ENUMS.lock().unwrap().iter() {
        let exporter = get_exporter_from_lang(
            output.lang.as_str(),
            output.clone(),
            output.generics.clone(),
        )?;

        // Collect the output content rather than writing to file
        if let Some(content) = exporter.to_output().to_string().strip_prefix("\"") {
            if let Some(content) = content.strip_suffix("\"") {
                index_content.push_str(content);
                index_content.push_str("\n\n");
            }
        }
    }

    // Write single index.ts file
    if !index_content.is_empty() {
        std::fs::write(export_path.join("index.ts"), index_content)?;
    }

    Ok(())
}

pub trait ToOutput {
    #[allow(unused)]
    fn to_output(&self) -> TokenStream;
    #[allow(unused)]
    fn to_file(&self, path: Option<PathBuf>) -> TSTypeResult<()>;
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Output {
    pub name: String,
    pub kind: OutputKind,
    pub generics: Vec<String>,
    #[allow(unused)]
    pub lang: String,
    #[allow(unused)]
    pub rename_all: Option<RenameRule>,
    pub export_path: Option<PathBuf>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum OutputKind {
    Struct(Vec<Field>),
    Enum(Vec<Variant>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub optional: bool,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: Option<Vec<Field>>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Type {
    String,
    Number,
    Boolean,
    HashMap(Box<Type>, Box<Type>),
    Array(Box<Type>),
    Optional(Box<Type>),
    Custom(String),
}
