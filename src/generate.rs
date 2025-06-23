use std::env;
use std::fmt::format;
use std::fs::{self, File};
use std::io::{Write};
use std::path::Path;

use anchor_idl::{Idl, IdlInstruction, IdlInstructionAccount, IdlInstructionAccountItem, IdlType};

use crate::utils::{to_camel_case, visit_account_item};

/*

Command to generate that project:
    cargo run -- generate \
    --idl /Users/emilemilovroydev/Rust/projects/Solana/shipment-manager/target/idl/shipment_manager.json \
    --program-crate-path /Users/emilemilovroydev/Rust/projects/Solana/shipment-manager --out /Users/emilemilovroydev/Rust/projects/Solana/shipment-manager/debug-wrapper
*/

pub fn generate_wrapper(
    idl_path: &str,
    crate_path: &str,
    out_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // let idl = load_idl(idl_path).unwrap();

    let idl_json = fs::read_to_string(idl_path).expect("failed to read IDL file");
    let idl: Idl = serde_json::from_str(&idl_json).expect("Failed to parseIDL");

    let program_name = idl.metadata.name;

    let crate_name = program_name.replace("-", "_");
    
    let mut call_functions = String::new();
    let mut call_main = String::new();

    for instruction in &idl.instructions {
        let func_name = format!("call_{}", instruction.name);
        let call = format!("    {}();\n", func_name);
        call_main.push_str(&call);
        let func = generate_instruction_function(instruction, &program_name);

    //     let func = format!(
    //         r#"
    // fn {func_name}() {{
    //     println!("Calling {original}...");
    //     // TODO: Mock accounts and context
    //     // let ctx = ...;
    //     // {crate}::program::{crate}::{original}(ctx).unwrap();
    // }}
    // "#,
    //         func_name = func_name,
    //         original = instruction.name,
    //         crate = crate_name
    //     );

        call_functions.push_str(&func);
    }

    let out_dir = Path::new(out_path);
    let src_dir = out_dir.join("src");

    fs::create_dir_all(&src_dir)?;

    // === Write Cargo.toml ===
    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
{crate_name} = {{ path = "{crate_path}" }}
anchor-lang = "0.31.1"
"#,
crate_name = crate_name,
crate_path = crate_path
    );

    fs::write(out_dir.join("Cargo.toml"), cargo_toml)?;
    

    // === Write mock.rs stub ===
    let mut mock_rs = File::create(src_dir.join("mock.rs"))?;
    
    let template_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templates/mock_template.rs");
    let mut template = fs::read_to_string(template_path).expect("Failed to read a string");
    template = template.replace("{crate_name}", &crate_name);

    let mock_rs_contents = format!(
        r#"use anchor_lang::prelude::*;
use {crate_name}::{{ID as PROGRAM_ID}};


    "#
    );

    mock_rs.write_all(template.as_bytes())?;

    // === Write main.rs stub ===
    let mut main_rs = File::create(src_dir.join("main.rs"))?;

    let main_rs_contents = format!(
        r#"use anchor_lang::prelude::*;
use {crate_name}::program::{crate_name}::*;
use {crate_name}::{{ID as PROGRAM_ID}};

    fn main() {{
        println!("Native debug wrapper for Anchor program: '{crate_name}'");
    {call_main}
    }}

    {call_functions}
    "#,
        crate_name = crate_name,
        call_main = call_main.trim_end(),
        call_functions = call_functions.trim_end(),
    );

    main_rs.write_all(main_rs_contents.as_bytes())?;

    for instruction in idl.instructions {
        generate_instruction_function(&instruction, &program_name);
    }
    Ok(())
}


pub fn generate_instruction_function(ix: &IdlInstruction, program_name: &str) -> String {
    let ix_name = &ix.name;
    let function_name = format!("call_{}", ix_name);
    let struct_name = to_camel_case(&ix.name);
    let bump_struct = format!("{}Bumps", struct_name);

    let mut bindings = vec![];
    let mut fields = vec![];
    let mut accounts_info_clones = vec![];

    for acc in &ix.accounts {
        let account: &IdlInstructionAccount = visit_account_item(acc).expect("Failed te retreive accounts");
        let acc_name: &String = &account.name;

        let mock_call: String = if acc_name.to_lowercase() == "systemprogram" {
            format!(r#"Box::leak(Box::new(mock_system_program()));"#)
        } else if account.signer {
            format!(r#"Box::leak(Box::new(mock_signer_account("{acc_name}")));"#)
        } else {
            // TODO: Generation of PDA should be handled better than this !
            format!(
                r#"Box::leak(Box::new(mock_pda_account(&[b"{acc_name}"], &PROGRAM_ID, 64)));"#
            )
        };

        bindings.push(format!("let {acc_name} = {mock_call};"));
        fields.push(format!(
            r#"{acc_name}: {}::try_from({acc_name}).unwrap()"#,
            if account.signer {
                "Signer"
            } else if acc_name.to_lowercase() == "systemprogram" {
                "Program"
            } else {
                "Account"
            }
        ));
        accounts_info_clones.push(format!("{acc_name}.clone()"));
    }

     // === Generate dummy args ===
    let mut args = vec![];
    let mut call_args = vec!["ctx".to_string()];

    for arg in &ix.args {
        let arg_name = &arg.name;
        let dummy = match &arg.ty {
            IdlType::U8 => "0u8".to_string(),
            IdlType::U64 => "0u64".to_string(),
            IdlType::Bool => "false".to_string(),
            IdlType::String => r#""test".to_string()"#.to_string(),
            IdlType::Pubkey => "Pubkey::new_unique()".to_string(),
            IdlType::Array(_, n) => format!("[0u8; {:?}]", n),
            _ => format!("/* unsupported arg type: {:?} */ Default::default()", arg.ty),
        };

        args.push(format!("let {arg_name} = {dummy};"));
        call_args.push(arg_name.to_string());
    }

    // === Compose final Rust code ===
    format!(
        r#"
fn {function_name}() {{
    println!("Calling {ix_name}...");

    {bindings}

    let mut accounts = {struct_name} {{
        {fields}
    }};

    let account_infos = vec![{accounts_info_clones}];
    let bumps = {bump_struct} {{ /* TODO: fill bump values */ }};

    let ctx = Context::new(
        &PROGRAM_ID,
        &mut accounts,
        &account_infos,
        bumps
    );

    {args}

    match {ix_name}({call_args}) {{
        Ok(_) => println!("✅ {ix_name} succeeded"),
        Err(e) => eprintln!("❌ {ix_name} failed: {{:?}}", e),
    }}
}}
    "#,
        function_name = function_name,
        ix_name = ix_name,
        bindings = bindings.join("\n    "),
        fields = fields.join("\n    "),
        accounts_info_clones = accounts_info_clones.join(", "),
        args = args.join("\n    "),
        call_args = call_args.join(", ")
    )

}