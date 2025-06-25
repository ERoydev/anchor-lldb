use crate::{generate::generate_instruction_function, generator::config::GeneratorConfig};
use anchor_idl::Idl;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

/*
This is the main logic responsible to generate all the files and code inside them.
*/

pub struct CodeGenerator<'a> {
    pub idl: &'a Idl,
    pub config: GeneratorConfig<'a>,
    pub crate_name: String,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(
        idl: &'a Idl,
        crate_path: &'a str,
        out_dir: PathBuf,
        package: &'a str,
    ) -> CodeGenerator<'a> {
        let program_name = idl.metadata.name.clone();
        let crate_name = program_name.replace("-", "_");

        let src_dir = out_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create dir");

        let config = GeneratorConfig::new(crate_path, out_dir, src_dir, package);

        CodeGenerator {
            idl,
            config,
            crate_name,
        }
    }

    pub fn generate_mock_rs(&self) -> Result<(), Box<dyn std::error::Error>> {
        // === Write mock.rs stub ===
        let mut mock_rs = File::create(self.config.src_dir.join("mock.rs"))?;

        let template_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/templates/mock_template.rs");
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
            let func = generate_instruction_function(instruction, &self.config.account_map);

            call_functions.push_str(&func);
        }

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
