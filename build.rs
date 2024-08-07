use std::fs::{self, create_dir_all};
use std::io;
use std::path::Path;
use std::process::Command;

const INPUT_ASSETS_DIR: &str = "assets";
const TARGET_ASSETS_DIR: &str = "target/assets";
const INPUT_SHADERS_DIR: &str = "assets/shaders";
const TARGET_SHADERS_DIR: &str = "target/assets/shaders";

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir(dst)?;
    }

    let exclude = Path::new(INPUT_SHADERS_DIR);

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path == exclude {
            continue;
        }

        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn copy_assets() {
    let input_assets = Path::new(INPUT_ASSETS_DIR);
    let target_assets = Path::new(TARGET_ASSETS_DIR);
    if copy_dir_all(input_assets, target_assets).is_err() {
        panic!("Assets copy failed");
    }
}

fn compile_shaders(shaders: Vec<(&str, &str)>) {
    // Define the base output directory
    let out_dir = Path::new(TARGET_SHADERS_DIR);

    for (shader, entry_point) in shaders {
        // Get the output file path
        let shader_path = Path::new(shader);
        let relative_path = shader_path.strip_prefix(INPUT_SHADERS_DIR).unwrap();
        let output_path = out_dir.join(relative_path).with_extension("spv");

        // Ensure the output directory exists
        if let Some(parent) = output_path.parent() {
            create_dir_all(parent).unwrap();
        }

        let status = Command::new("slangc")
            .arg(shader)
            .arg("-emit-spirv-directly")
            .arg("-g2")
            .arg("-profile")
            .arg("glsl_460")
            .arg("-target")
            .arg("spirv")
            .arg("-o")
            .arg(&output_path)
            .arg("-entry")
            .arg(entry_point)
            .status()
            .unwrap();

        if !status.success() {
            panic!("Shader compilation failed for {}", shader);
        }
    }
}

fn main() {
    copy_assets();

    // Specify the list of shaders and their entry points
    let shaders = vec![
        ("assets/shaders/builtin/object.vert.slang", "main"),
        ("assets/shaders/builtin/object.frag.slang", "main"),
    ];
    compile_shaders(shaders);

    // rerun when shaders change
    println!("cargo:rerun-if-changed=src/shaders");
}
