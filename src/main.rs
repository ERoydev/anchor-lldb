use anchor_idl::Idl;
use clap::{Parser, Subcommand};
use tempfile::tempdir;
use std::{fs, path::PathBuf};

use crate::{temp_crate_builder::{build_and_extract_binary, maybe_inject_workspace, prepare_output_path}, utils::{binary_name_from_package, cli_error, inject_workspace_member}};
mod generate;
pub mod generator;
pub mod scripts;
mod utils;
mod temp_crate_builder;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Generate {
        #[arg(long, help = "Package name of the Anchor program (required)")]
        package: String,

        #[arg(
            long,
            help = "Optional path to the generated IDL .json file. Inferred from --package if not provided."
        )]
        idl: Option<String>,

        #[arg(
            long,
            help = "Optional path to the Anchor program crate root. Inferred from --package if not provided."
        )]
        program_crate_path: Option<String>,

        #[arg(
            help = "Optional output directory for the generated wrapper (default: debug-wrapper)"
        )]
        out: Option<String>,
    },
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::Generate {
            package,
            idl,
            program_crate_path,
            out,
        } => {
            let (idl_path, program_crate_path) = match (&idl, &program_crate_path) {
                (Some(idl), Some(crate_path)) => (idl.clone(), crate_path.clone()),
                _ => utils::infer_paths(&package)?,
            };

            let idl_json = fs::read_to_string(&idl_path)
                .map_err(|e| format!("Failed to read IDL file at {}: {}", idl_path, e))?;

            let idl: Idl = serde_json::from_str(&idl_json)
                .map_err(|e| format!("Failed to parse IDL JSON at {}: {}", idl_path, e))?;

            // determine the output path: either user-specified or a temporary one
            let (out_path, is_ephemeral, _temp_guard) = prepare_output_path(&out)?;

            // Inject debug-wrapper into root cargo workspace if not ephemeral
            maybe_inject_workspace(&out_path, is_ephemeral);

            // Generate the debug wrapper crate files
            if let Err(e) = generate::generate_wrapper(&idl, &program_crate_path, &out_path, &package) {
                cli_error(e);
            }

            // If using temp dir, build the binary, extract it, and optionally run it
            if is_ephemeral {
                build_and_extract_binary(&package, &out_path)?;
            }
        }
    }

    Ok(())
}