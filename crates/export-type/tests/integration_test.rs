#![allow(unused)]
use export_type::ExportType;

#[derive(ExportType)]
#[export_type(path = "target/test_exports", lang = "typescript")]
struct TestUser {
    id: i32,
    name: String,
    #[export_type(rename = "emailAddress")]
    email: Option<String>,
    roles: Vec<String>,
    #[export_type(rename = "custom_headers")]
    custom_headers: std::collections::HashMap<String, String>,
}

#[derive(ExportType)]
#[export_type(path = "target/test_exports", lang = "typescript")]
enum TestStatus {
    Active,
    Inactive,
    Pending { reason: String },
}

#[test]
fn test_generated_files_exist() {
    // These files should be generated during compilation
    assert!(std::path::Path::new("target/test_exports/index.ts").exists());
}

#[test]
fn test_generated_content() {
    let user_content = std::fs::read_to_string("target/test_exports/index.ts")
        .expect("Should read user typescript file");

    assert!(user_content.contains("export interface TestUser"));
    assert!(user_content.contains("id: number"));
    assert!(user_content.contains("emailAddress?: string"));
    assert!(user_content.contains("roles: string[]"));

    let status_content = std::fs::read_to_string("target/test_exports/index.ts")
        .expect("Should read status typescript file");

    assert!(status_content.contains("export type TestStatus"));
    assert!(status_content.contains("| \"Active\""));
    assert!(status_content.contains("| { type: \"Pending\";     reason: string; }"));
}
