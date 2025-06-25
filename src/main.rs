use anchor_idl::Idl;
use clap::{Parser, Subcommand};
use std::{fs};

use crate::utils::{cli_error, inject_workspace_member};
mod generate;
pub mod generator;
pub mod scripts;
mod utils;

const DEFAULT_OUT_PATH: &str = "debug-wrapper";

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

            let out_path = out.unwrap_or_else(|| DEFAULT_OUT_PATH.to_string());

            let root_dir = std::env::current_dir()?;
            let root_cargo = root_dir.join("Cargo.toml");

            if let Err(e) = inject_workspace_member(&root_cargo, &out_path) {
                cli_error(e);
            }

            if let Err(e) = generate::generate_wrapper(&idl, &program_crate_path, &out_path, &package) {
                cli_error(e);
            }
        }
    }

    Ok(())
}
