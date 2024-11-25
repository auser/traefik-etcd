use std::path::PathBuf;

use clap::Args;
use schemars::{schema_for_value, JsonSchema};
use serde::Serialize;
use tempfile::Builder;

use crate::{
    config::{
        deployment::DeploymentConfig,
        headers::HeadersConfig,
        host::{HostConfig, PathConfig},
        middleware::MiddlewareConfig,
        selections::{FromClientIpConfig, SelectionConfig, WithCookieConfig},
    },
    core::client::StoreClient,
    error::TraefikResult,
    features::etcd::Etcd,
    TraefikConfig,
};

#[derive(Args, Debug)]
pub struct CodegenCommand {
    #[arg(short, long, default_value = "frontend/src/lib/types")]
    output_dir: String,
}

struct GeneratedLanguage {
    pub name: &'static str,
    pub extension: &'static str,
}

pub async fn run(
    command: &CodegenCommand,
    _client: &StoreClient<Etcd>,
    _traefik_config: &mut TraefikConfig,
) -> TraefikResult<()> {
    let tmpfile = Builder::new()
        .prefix("traefikctl")
        .suffix(".json")
        .tempfile()?;
    println!("Writing to {}", tmpfile.path().display());

    let output = PathBuf::from(&command.output_dir);

    let language = GeneratedLanguage {
        name: "typescript",
        extension: "ts",
    };

    gen_schemas(language, output).await?;

    Ok(())
}

async fn gen_schemas(_language: GeneratedLanguage, output: PathBuf) -> anyhow::Result<()> {
    let name = "traefik";
    let tmpdir = Builder::new().prefix("traefikctl").tempdir()?;
    let schemas = &[
        (
            "host",
            gen_schema::<HostConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "path",
            gen_schema::<PathConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "headers",
            gen_schema::<HeadersConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "selection",
            gen_schema::<SelectionConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "with_cookie",
            gen_schema::<WithCookieConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "from_client_ip",
            gen_schema::<FromClientIpConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "middleware",
            gen_schema::<MiddlewareConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "deployment",
            gen_schema::<DeploymentConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
        (
            "traefik",
            gen_schema::<TraefikConfig>(tmpdir.path().to_path_buf())
                .expect("Failed to generate schema"),
        ),
    ];

    // let output = PathBuf::from("frontend/src/lib/types");
    let tmpdir = tmpdir.path().to_path_buf();
    for (name, schema) in schemas {
        let generated_schema_file = gen_schema_file(schema.clone(), tmpdir.clone(), name)
            .await
            .expect("Failed to generate schema file");
        println!("Generated {}", generated_schema_file.display());
    }

    let mut cmd = tokio::process::Command::new("quicktype");
    cmd.arg(tmpdir)
        .arg("-l")
        .arg("typescript")
        .arg("-o")
        .arg(output.join(format!("{}.ts", name)));
    cmd.output()
        .await
        .expect("Failed to generate typescript file");
    // let combined = serde_json::json!({
    //     "$schema": "http://json-schema.org/draft-07/schema#",
    //     "title": "TraefikConfig Schema",
    //     "definitions": schemas.iter()
    //         .filter_map(|s| serde_json::from_str::<serde_json::Value>(s).ok())
    //         .collect::<Vec<_>>()
    // });
    Ok(())
}

fn gen_schema<T: JsonSchema + Default + Serialize>(output_dir: PathBuf) -> TraefikResult<PathBuf> {
    let root_schema = schema_for_value!(T::default());

    let schema_with_meta = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": std::any::type_name::<T>(),
        "definitions": root_schema.definitions,
        "properties": root_schema.schema.object.unwrap().properties,
        "type": "object"
    });

    let schema_str = serde_json::to_string_pretty(&schema_with_meta).unwrap();
    let output = output_dir.join(format!("{}.schema.json", std::any::type_name::<T>()));
    std::fs::write(&output, schema_str)?;
    Ok(output)
}

async fn gen_schema_file(
    input_file: PathBuf,
    output_dir: PathBuf,
    name: &str,
) -> TraefikResult<PathBuf> {
    let mut cmd = tokio::process::Command::new("quicktype");
    // Generate the schema for the given language
    let schema_output_path = output_dir.clone().join(format!("{}.schema.json", name));
    let _schema_res = cmd
        .arg(input_file.as_path())
        .arg("-l")
        .arg("schema")
        .arg("-o")
        .arg(schema_output_path.clone())
        .output()
        .await?;
    Ok(schema_output_path)
}

async fn gen_lang(
    language: &GeneratedLanguage,
    name: &str,
    input_file: PathBuf,
    output_dir: PathBuf,
) -> TraefikResult<PathBuf> {
    // Ensure the output directory exists
    let output = PathBuf::from(&output_dir);
    std::fs::create_dir_all(output.clone())?;

    let language_name = language.name;
    let language_extension = language.extension;

    // Generate the code for the given language
    let code_output_path = output
        .clone()
        .join(format!("{}.{}", name, language_extension));
    let mut cmd = tokio::process::Command::new("typeshare");
    let code_res = cmd
        .arg(input_file)
        .arg("--lang")
        .arg(language_name)
        .arg(format!("--output-file={}", code_output_path.display()))
        .output()
        .await?;

    println!("{:?}", code_res);

    Ok(code_output_path)
}
