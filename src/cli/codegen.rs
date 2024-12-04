use std::{
    env,
    path::{Path, PathBuf},
};

use clap::Args;
use tracing::debug;

use crate::{core::client::StoreClient, error::TraefikResult, features::etcd::Etcd, TraefikConfig};

#[derive(Args, Debug)]
pub struct CodegenCommand {
    #[arg(short, long, default_value = "generated/types")]
    output_dir: Option<String>,

    #[arg(short = 'L', long, default_value = "typescript")]
    language: Option<String>,
}

pub async fn run(
    command: &CodegenCommand,
    _client: &StoreClient<Etcd>,
    _traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let output = command.output_dir.as_ref().map(PathBuf::from);

    handle_codegen(output).await?;

    Ok(())
}

async fn handle_codegen(output: Option<PathBuf>) -> anyhow::Result<()> {
    let types_path =
        output.unwrap_or_else(|| PathBuf::from(env::var("TYPES_OUT_DIR").unwrap_or_default()));

    let target_dir = match env::var("OUT_DIR") {
        Ok(out_dir) => PathBuf::from(out_dir).join("frontend/src/lib/types"),
        Err(_) => Path::new("frontend/src/lib/types").to_path_buf(),
    };

    // Only copy if the types were generated
    if types_path.exists() {
        // Create target directory if it doesn't exist
        std::fs::create_dir_all(target_dir.clone()).expect("Failed to create target directory");

        // Copy the generated index.ts file
        if let Ok(entries) = std::fs::read_dir(&types_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "ts") {
                    let file_name = path.file_name().unwrap();
                    let target_path = target_dir.join(file_name);

                    debug!("copying: {:?} to {:?}", path, target_path);
                    std::fs::copy(&path, &target_path)
                        .unwrap_or_else(|e| panic!("Failed to copy {:?}: {}", file_name, e));
                }
            }
        }
    }

    Ok(())
}

// fn gen_schema<T: JsonSchema + Default + Serialize>(output_dir: PathBuf) -> TraefikResult<PathBuf> {
//     let root_schema = schema_for_value!(T::default());

//     let schema_with_meta = serde_json::json!({
//         "$schema": "http://json-schema.org/draft-07/schema#",
//         "title": std::any::type_name::<T>(),
//         "definitions": root_schema.definitions,
//         "properties": root_schema.schema.object.unwrap().properties,
//         "type": "object"
//     });

//     let schema_str = serde_json::to_string_pretty(&schema_with_meta).unwrap();
//     let output = output_dir.join(format!("{}.schema.json", std::any::type_name::<T>()));
//     std::fs::write(&output, schema_str)?;
//     Ok(output)
// }

// async fn gen_schema_file(
//     input_file: PathBuf,
//     output_dir: PathBuf,
//     name: &str,
// ) -> TraefikResult<PathBuf> {
//     let mut cmd = tokio::process::Command::new("quicktype");
//     // Generate the schema for the given language
//     let schema_output_path = output_dir.clone().join(format!("{}.schema.json", name));
//     let _schema_res = cmd
//         .arg(input_file.as_path())
//         .arg("-l")
//         .arg("schema")
//         .arg("-o")
//         .arg(schema_output_path.clone())
//         .output()
//         .await?;
//     Ok(schema_output_path)
// }

// #[allow(dead_code)]
// async fn gen_lang(
//     language: &GeneratedLanguage,
//     name: &str,
//     input_file: PathBuf,
//     output_dir: PathBuf,
// ) -> TraefikResult<PathBuf> {
//     // Ensure the output directory exists
//     let output = PathBuf::from(&output_dir);
//     std::fs::create_dir_all(output.clone())?;

//     let language_name = language.name;
//     let language_extension = language.extension;

//     // Generate the code for the given language
//     let code_output_path = output
//         .clone()
//         .join(format!("{}.{}", name, language_extension));
//     let mut cmd = tokio::process::Command::new("typeshare");
//     let code_res = cmd
//         .arg(input_file)
//         .arg("--lang")
//         .arg(language_name)
//         .arg(format!("--output-file={}", code_output_path.display()))
//         .output()
//         .await?;

//     println!("{:?}", code_res);

//     Ok(code_output_path)
// }
