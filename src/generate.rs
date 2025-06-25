use anchor_idl::{Idl, IdlInstruction};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::generator::accounts::InstructionAcountCode;
use crate::generator::args::InstructionArgCode;
use crate::generator::bumps::InstructionBumpsCode;
use crate::generator::codegen::CodeGenerator;
use crate::utils::to_camel_case;

/*

Command to generate that project:
    cargo run -- generate \
    --idl /Users/emilemilovroydev/Rust/projects/Solana/shipment-managment/target/idl/shipment_manager.json \
    --program-crate-path /Users/emilemilovroydev/Rust/projects/Solana/shipment-managment --out /Users/emilemilovroydev/Rust/projects/Solana/shipment-managment/debug-wrapper


    Usage using the extension be like
        - cargo run install --path . => TO install globbally this CLI tool

        - anchor-lldb generate --package={packageName}
*/

pub fn generate_wrapper(
    idl: &Idl,
    crate_path: &str,
    out_path: &PathBuf,
    package: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new(out_path).to_owned();
    let code_generator = CodeGenerator::new(&idl, &crate_path, out_dir, &package);

    code_generator.generate_cargo_toml()?;
    code_generator.generate_mock_rs()?;
    code_generator.generate_main_rs()?;

    Ok(())
}

pub fn generate_instruction_function(
    ix: &IdlInstruction,
    account_map: &HashMap<String, String>,
) -> String {
    let ix_name = &ix.name;
    let function_name = format!("call_{}", ix_name);
    let struct_name = to_camel_case(&ix.name);
    let bump_struct = format!("{}Bumps", struct_name);

    let instruction_account = InstructionAcountCode::generate_account_code(ix, account_map);
    let instuction_args = InstructionArgCode::generate_argument_code(ix);
    let instruction_bumps = InstructionBumpsCode::generate_bumps_code(ix);

    // === Compose final Rust code ===
    format!(
        r#"
fn {function_name}() {{
    {bindings}

    let mut accounts = {struct_name} {{
        {fields}
    }};

    let account_infos = vec![{accounts_info_clones}];
    let bumps = {bump_struct} {{ 
        {bump_fields}
    }};

    let ctx = Context::new(
        &PROGRAM_ID,
        &mut accounts,
        &account_infos,
        bumps
    );
    {args}

    match {ix_name}({call_args}) {{
        Ok(_) => println!("{ix_name} succeeded"),
        Err(e) => eprintln!("{ix_name} failed: {{:?}}", e),
    }}
}}
    "#,
        function_name = function_name,
        ix_name = ix_name,
        bindings = instruction_account.bindings.join("\n    "),
        fields = instruction_account.fields.join(",\n       "),
        accounts_info_clones = instruction_account.account_infos.join(", "),
        args = instuction_args.args.join("\n    "),
        call_args = instuction_args.call_args.join(", "),
        bump_fields = instruction_bumps.bump_fields.join(",\n     "),
    )
}

