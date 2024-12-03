use std::path::PathBuf;

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::error::TSTypeResult;
use crate::{get_exporter_from_lang, RenameRule};

// Track all types for single file generation, using HashSet to prevent duplicates
pub static COLLECTED_TYPES: Lazy<Mutex<HashMap<PathBuf, HashSet<Output>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn add_struct_or_enum(path: PathBuf, output: Output) -> TSTypeResult<()> {
    COLLECTED_TYPES
        .lock()
        .unwrap()
        .entry(path)
        .or_insert_with(HashSet::new)
        .insert(output);
    Ok(())
}

pub fn create_exporter_files(export_path: PathBuf) -> TSTypeResult<()> {
    let mut index_content = String::new();

    for (_path, outputs) in COLLECTED_TYPES.lock().unwrap().iter() {
        for output in outputs {
            let exporter = get_exporter_from_lang(
                output.lang.as_str(),
                output.clone(),
                output.generics.clone(),
            )?;

            let output = exporter.to_output();

            index_content.push_str(&output);
            index_content.push_str("\n\n");
        }
    }

    // Write single index.ts file
    if !index_content.is_empty() {
        std::fs::create_dir_all(&export_path)?;
        std::fs::write(export_path.join("index.ts"), index_content)?;
    }

    Ok(())
}

pub trait ToOutput {
    #[allow(unused)]
    fn to_output(&self) -> String;
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
