use std::env;
use std::fs::{self, File};
use std::io::{Write};
use std::path::{Path, PathBuf};

use anchor_idl::{Idl, IdlInstruction, IdlInstructionAccount, IdlSeed, IdlType};

use crate::utils::{capitalize_first_letter, to_camel_case, visit_account_item};

/*

Command to generate that project:
    cargo run -- generate \
    --idl /Users/emilemilovroydev/Rust/projects/Solana/shipment-managment/target/idl/shipment_manager.json \
    --program-crate-path /Users/emilemilovroydev/Rust/projects/Solana/shipment-managment --out /Users/emilemilovroydev/Rust/projects/Solana/shipment-managment/debug-wrapper


    Usage using the extension be like 
        - cargo run install --path . => TO install globbally this CLI tool

        - anchor-lldb generate --package={packageName}
*/

struct GeneratorConfig<'a> {
    pub program_path: &'a str,
    pub out_dir: PathBuf,
    pub src_dir: PathBuf,
    pub package_name: &'a str,
}

impl<'a> GeneratorConfig<'a> {
    pub fn new(crate_path: &'a str, out_dir: PathBuf, src_dir: PathBuf, package_name: &'a str) -> Self {
        GeneratorConfig { 
            program_path: crate_path, 
            out_dir,
            src_dir,
            package_name: package_name }
    }
}

struct CodeGenerator<'a> {
    pub idl: &'a Idl,
    pub config: GeneratorConfig<'a>,
    pub crate_name: String,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(idl: &'a Idl, crate_path: &'a str, out_dir: PathBuf, package: &'a str) -> CodeGenerator<'a> {

        let program_name = idl.metadata.name.clone();
        let crate_name = program_name.replace("-", "_");

        let src_dir = out_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create dir");

        let config = GeneratorConfig::new(crate_path, out_dir, src_dir, package);

        CodeGenerator {
            idl,
            config,
            crate_name
        }
    }

    pub fn generate_mock_rs(&self) -> Result<(), Box<dyn std::error::Error>> {
        // === Write mock.rs stub ===
        let mut mock_rs = File::create(self.config.src_dir.join("mock.rs"))?; //TODO: Fix that
        
        let template_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templates/mock_template.rs");
        let mut template = fs::read_to_string(template_path).expect("Failed to read a string");

        template = template.replace("{crate_name}", &self.crate_name);

        let mock_rs_contents = format!(
            r#"use {crate_name}::{{ID as PROGRAM_ID}};
        "#,
        crate_name = &self.crate_name
        );

        let full_contents = format!("{}{}", mock_rs_contents, template);
        mock_rs.write_all(full_contents.as_bytes())?;

        Ok(())
    }

    pub fn generate_cargo_toml(&self) -> Result<(), Box<dyn std::error::Error>> {
        // === Write Cargo.toml ===
        let cargo_toml = format!(
            r#"[package]
    name = "{crate_name}"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    {crate_name} = {{ path = "{program_path}", package = "{package_name}" }}
    anchor-lang = "0.31.1"
    "#,
    crate_name = &self.crate_name,
    program_path = &self.config.program_path,
    package_name = self.config.package_name
        );

        match fs::write(&self.config.out_dir.join("Cargo.toml"), cargo_toml) {
            Ok(_) => println!("File written successfully"),
            Err(e) => eprintln!("Filed to write Cargo.toml: {}", e),
        }

        Ok(())
    }

    pub fn generate_main_rs(&self) -> Result<(), Box<dyn std::error::Error>> {
            // === Write main.rs stub ===
        let mut main_rs = File::create(&self.config.src_dir.join("main.rs"))?;

        let mut call_functions = String::new();
        let mut call_main = String::new();

        for instruction in &self.idl.instructions {
            let func_name = format!("call_{}", instruction.name);
            let call = format!("    {}();\n", func_name);
            call_main.push_str(&call);
            let func = generate_instruction_function(instruction);

            call_functions.push_str(&func);
        }

    // 

        let main_rs_contents = format!(
            r#"use anchor_lang::prelude::*;

    extern crate {crate_name} as cr;
    use cr::ID as PROGRAM_ID;
    use cr::*;
    use cr::{crate_name}::*;

    mod mock;
    use mock::*;

        fn main() {{
            println!("Native debug wrapper for Anchor program: '{crate_name}'");
        {call_main}
        }}

        {call_functions}
        "#,
            crate_name = &self.crate_name,
            call_main = call_main.trim_end(),
            call_functions = call_functions.trim_end(),
        );

        main_rs.write_all(main_rs_contents.as_bytes())?;

        Ok(())
    }
}


struct InstructionAcountCode {
    bindings: Vec<String>,
    fields: Vec<String>,
    account_infos: Vec<String>
}

impl InstructionAcountCode {
    pub fn generate_account_code(ix: &IdlInstruction) -> InstructionAcountCode {
        let mut bindings = vec![];
        let mut fields = vec![];
        let mut accounts_info_clones = vec![];

        for acc in &ix.accounts {
            let account: &IdlInstructionAccount = visit_account_item(acc).expect("Failed te retreive accounts");
            let acc_name: &String = &account.name;
            let acc_struct_name = capitalize_first_letter(&acc_name);

            let mock_call: String = if acc_name.to_lowercase() == "system_program" {
                format!(r#"Box::leak(Box::new(mock_system_program()))"#)
            } else if account.signer {
                format!(r#"Box::leak(Box::new(mock_signer_account("{acc_name}")))"#)
            } else {
                // TODO: Generation of PDA should be handled better than this !
                format!(
                    r#"Box::leak(Box::new(mock_pda_account::<{acc_struct_name}>(&[b"{acc_name}"], &PROGRAM_ID, 64)))"#
                )
            };

            bindings.push(format!("let {acc_name} = {mock_call};"));

            let account_type =  if account.signer {
                "Signer"
            } else if acc_name.to_lowercase() == "system_program" {
                "Program"
            } else {
                "Account"
            };

            let prefix = if acc_name.to_lowercase() == "system_program" {
                "&*"
            } else {
                ""
            };

            fields.push(format!(
                r#"{acc_name}: {account_type}::try_from({prefix}{acc_name}).unwrap()"#
            ));

            accounts_info_clones.push(format!("{acc_name}.clone()"));
        }

        InstructionAcountCode { bindings, fields, account_infos: accounts_info_clones }

    }
}

struct InstructionArgCode {
    args: Vec<String>,
    call_args: Vec<String>,
} 

impl InstructionArgCode {
    pub fn generate_argument_code(ix: &IdlInstruction) -> InstructionArgCode {
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

        InstructionArgCode { args, call_args }
    }
}

struct InstructionBumpsCode {
    bump_fields: Vec<String>,
}

impl InstructionBumpsCode {
    pub fn generate_bumps_code(ix: &IdlInstruction) -> InstructionBumpsCode {
        let mut bump_fields = vec![];

        // The Pda derive logic
        for acc in &ix.accounts {
            let account: &IdlInstructionAccount = visit_account_item(acc).unwrap();
            let acc_name = &account.name;

            if let Some(pda) = &account.pda {
                // Create a list of bytes for find_program_address
                let mut seed_exprs = vec![];

                for seed in &pda.seeds {
                    match seed {
                        IdlSeed::Const(seed_const) => {
                            seed_exprs.push(format!("&{:?}", seed_const.value));
                        },
                        IdlSeed::Arg(seed_arg) => {
                            seed_exprs.push(format!("&{}::to_le_bytes()", seed_arg.path));
                        },
                        IdlSeed::Account(seed_account) => {
                            let account_var = seed_account.path.split('.').next().unwrap();
                            seed_exprs.push(format!("{}.key().as_ref()", account_var));
                        }
                    }
                }

                let seed_refs = seed_exprs.join(", ");
                bump_fields.push(format!(
                    "{acc_name}: Pubkey::find_program_address(&[{seed_refs}], &PROGRAM_ID).1"
                ));
            }
        }

        InstructionBumpsCode { bump_fields }

    }
}

pub fn generate_wrapper(
    idl: &Idl,
    crate_path: &str,
    out_path: &str,
    package: &str
) -> Result<(), Box<dyn std::error::Error>> {
    
    let out_dir = Path::new(out_path).to_owned();
    let code_generator = CodeGenerator::new(&idl, &crate_path, out_dir, &package);

    code_generator.generate_cargo_toml()?;
    code_generator.generate_mock_rs()?;
    code_generator.generate_main_rs()?;

    Ok(())
}


pub fn generate_instruction_function(ix: &IdlInstruction) -> String {
    let ix_name = &ix.name;
    let function_name = format!("call_{}", ix_name);
    let struct_name = to_camel_case(&ix.name);
    let bump_struct = format!("{}Bumps", struct_name);

    let instruction_account = InstructionAcountCode::generate_account_code(ix);
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