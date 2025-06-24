use clap::{Parser, Subcommand};

use crate::utils::infer_paths;

mod generate;
mod utils;

// https://www.shuttle.dev/blog/2023/12/08/clap-rust

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Generates a debug wrapper from Anchor IDL
    // Generate {
    //     #[arg(long)]
    //     idl: Option<String>,
    //     #[arg(long)]
    //     program_crate_path: Option<String>,
    //     #[arg(long)]
    //     out: Option<String>,
    // },
    Generate {
        #[arg(long)]
        package: String,
        #[arg(long)]
        idl: Option<String>,
        #[arg(long)]
        program_crate_path: Option<String>,
        out: Option<String>
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::Generate { package, idl, program_crate_path, out } => {

            let (idl_path, program_crate_path) = match (&idl, &program_crate_path) {
                (Some(idl), Some(crate_path)) => (idl.clone(), crate_path.clone()),
                _ => utils::infer_paths(&package)?,
            };

            let out_path = out.unwrap_or_else(|| "debug-wrapper".to_string());
            generate::generate_wrapper(&idl_path, &program_crate_path, &out_path, &package)?;
        }
    }

    Ok(())
}

// #[derive(Parser)]
// #[command(name = "solana-anchor-debuggen", about = "Generate native Rust debug harnesses for Anchor programs")]
// struct Cli {
//     #[command(subcommand)]
//     command: Commands,
// }

// #[derive(Subcommand)]
// enum Commands {
//     /// Generate native Rust wrapper from IDL
//     Generate {
//         #[arg(long)]
//         idl: String,

//         #[arg(long)]
//         program_crate_path: String,

//         #[arg(long, default_value = "debug_wrapper")]
//         out: String,
//     },
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let cli = Cli::parse();

//     match cli.command {
//         Commands::Generate {
//             idl,
//             program_crate_path,
//             out,
//         } => {
//             generate::generate_wrapper(&idl, &program_crate_path, &out)?;
//         }
//     }

//     Ok(())
// }


