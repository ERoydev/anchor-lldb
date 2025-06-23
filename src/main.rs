use clap::{Parser, Subcommand};

mod generate;
mod utils;

// https://www.shuttle.dev/blog/2023/12/08/clap-rust

#[derive(Parser)]
#[command(name = "solana-anchor-debuggen", about = "Generate native Rust debug harnesses for Anchor programs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate native Rust wrapper from IDL
    Generate {
        #[arg(long)]
        idl: String,

        #[arg(long)]
        program_crate_path: String,

        #[arg(long, default_value = "debug_wrapper")]
        out: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            idl,
            program_crate_path,
            out,
        } => {
            generate::generate_wrapper(&idl, &program_crate_path, &out)?;
        }
    }

    Ok(())
}


// Bellow is the mocked version that i've got working !

// use std::collections::BTreeMap;

// use anchor_lang::prelude::*;
// use anchor_lang::Bump;
// use shipment_manager::shipment_manager::initialize_counter;
// use shipment_manager::InitializeCounter;
// use shipment_manager::InitializeCounterBumps;
// use shipment_manager::ID as PROGRAM_ID;

// mod mocks;

// use mocks::*;

// fn main() {
//     println!("Native debug wrapper for Anchor program: 'shipment_manager'");
//     call_accept_shipment();
//     call_create_shipment();
//     call_initialize_counter();
//     call_refuse_shipment();
//     call_validate_shipment();
// }


// fn call_accept_shipment() {
//     println!("Calling accept_shipment...");
//     // TODO: Mock accounts and context
//     // let ctx = ...;

//     // shipment_manager::program::shipment_manager::accept_shipment(ctx).unwrap();
// }

// fn call_create_shipment() {
//     println!("Calling create_shipment...");
//     // TODO: Mock accounts and context
//     // let ctx = ...;
//     // shipment_manager::program::shipment_manager::create_shipment(ctx).unwrap();
// }


// fn call_initialize_counter() {
//     println!("Calling initialize_counter...");

//     let signer = Box::leak(Box::new(mock_signer_account("signer")));

//     // PDA: [b"counter"]
//     let (pda, bump) = Pubkey::find_program_address(&[b"counter"], &PROGRAM_ID);
//     let counter = Box::leak(Box::new(mock_pda_account(&[b"counter"], &PROGRAM_ID, 64)));

//     let system_program = Box::leak(Box::new(mock_system_program()));

//     // Build account struct
//     let mut accounts = InitializeCounter {
//         counter: Account::try_from(counter).unwrap(),
//         signer: Signer::try_from(signer).unwrap(),
//         system_program: Program::try_from(&*system_program).expect("REQUIRED")
//     };

//     let account_infos = vec![counter.clone(), signer.clone(), system_program.clone()];

//     let bumps = InitializeCounterBumps { counter: 2 };

//     println!("{:?}", bumps);

//     // // Construct context with bumps
//     let ctx = Context::new(
//         &PROGRAM_ID, 
//         &mut accounts, 
//         &account_infos, 
//         bumps
//     );

//     // Run the instruction
//     match initialize_counter(ctx) {
//         Ok(_) => println!("✅ initialize_counter succeeded"),
//         Err(e) => eprintln!("❌ initialize_counter failed: {:?}", e),
//     }
// }

// fn call_refuse_shipment() {
//     println!("Calling refuse_shipment...");
//     // TODO: Mock accounts and context
//     // let ctx = ...;
//     // shipment_manager::program::shipment_manager::refuse_shipment(ctx).unwrap();
// }

// fn call_validate_shipment() {
//     println!("Calling validate_shipment...");
//     // TODO: Mock accounts and context
//     // let ctx = ...;
//     // shipment_manager::program::shipment_manager::validate_shipment(ctx).unwrap();
// }


/*

Example usage when i debug that shit

(lldb) fr v ctx
    (anchor_lang::context::Context<shipment_manager::InitializeCounter>) ctx = {
    program_id = 0x0000000100054fd4
    accounts = 0x000000016fdfe2e0
    remaining_accounts = {
        data_ptr = 0x0000000126e055f0
        length = 3
    }
    bumps = (counter = '\x02')
    }

(lldb) p ctx.accounts
    (shipment_manager::InitializeCounter *) 0x000000016fdfe2e0

(lldb) p *ctx.accounts
    (shipment_manager::InitializeCounter) {
        counter = {
            account = (count = 0)
            info = 0x0000000126e06150
        }

        signer = {
            info = 0x0000000126e05ea0
        }

        system_program = {
            info = 0x0000000126e05f70
            _phantom = {}
        }
    }

(lldb) p (*ctx.accounts).counter
    (anchor_lang::accounts::account::Account<shipment_manager::ShipmentIdCounter>) {
        account = (count = 0)
        info = 0x0000000126e06150
    }
*/