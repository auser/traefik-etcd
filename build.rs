use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Only run this in release mode
    if env::var("PROFILE").unwrap() == "release" {
        let out_dir = env::var("OUT_DIR").unwrap();
        let frontend_build = Path::new("frontend/build");
        let dest_path = Path::new(&out_dir).join("frontend/build");

        if frontend_build.exists() {
            println!("cargo:rerun-if-changed=frontend/build");

            // Create the destination directory if it doesn't exist
            fs::create_dir_all(&dest_path).unwrap();

            // Copy the frontend build directory to OUT_DIR
            copy_dir_all(frontend_build, &dest_path).expect("Failed to copy frontend build");
        } else {
            println!("cargo:warning=Frontend build directory not found. Make sure to build frontend first.");
        }
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
