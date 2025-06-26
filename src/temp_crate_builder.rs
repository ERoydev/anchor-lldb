use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};

use crate::utils::{binary_name_from_package, cli_error, inject_workspace_member};


pub fn prepare_output_path(user_out: &Option<String>) -> Result<(PathBuf, bool, Option<TempDir>), Box<dyn std::error::Error>> {
    if let Some(user_path) = user_out {
        Ok((PathBuf::from(user_path), false, None))
    } else {
        let temp = tempdir()?;
        Ok((temp.path().to_path_buf(), true, Some(temp)))
    }
}

// If the output path is not tempDir that means i need to add inside root cargo the name of this crate which is going to be created by this CLI tool
pub fn maybe_inject_workspace(out_path: &Path, is_ephemeral: bool) {
    if !is_ephemeral {
        if let Ok(root_dir) = std::env::current_dir() {
            let root_cargo = root_dir.join("Cargo.toml");
            if let Err(e) = inject_workspace_member(&root_cargo, out_path.to_str().unwrap()) {
                cli_error(e);
            }
        }
    }
}

// This should build the tempDir into a specified path to maintain the compiled data and then just move the exe binary used for debbugging to provide easy access
pub fn build_and_extract_binary(
    package: &str,
    out_path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = out_path.join("Cargo.toml");

    let build_status = std::process::Command::new("cargo")
        .args([
            "build",
            "--manifest-path",
            manifest_path.to_str().unwrap(),
            "--target-dir",
            "target/debuggen",
        ])
        .status()?;

    if !build_status.success() {
        panic!("Failed to uild the debug wrapper in order to create the exe binary file.");
    }

    let binary_name = binary_name_from_package(package);
    let built_bin_path = PathBuf::from("target/debuggen/debug").join(&binary_name);
    let bin_out_path = std::env::current_dir()?.join("target/debug").join(package);

    std::fs::copy(&built_bin_path, &bin_out_path)?;
    println!("\n[INFO] Debug binary successfully written to:\n -> {}\n::BIN_OUT::{}\n", bin_out_path.display(), bin_out_path.display());

    let run_status = std::process::Command::new(&bin_out_path).status()?;

    if !run_status.success() {
        panic!("Execution of the deug wrapper binary failed.");
    }

    Ok(())
}
