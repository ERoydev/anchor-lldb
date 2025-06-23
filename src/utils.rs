
// pub fn generate_toml_file() {
//     const cargo_toml = format!(
//         r#"[package]
// name = "{debug_crate_name}"
// version = "0.1.0"
// edition = "2021"

// [dependencies]
// {crate_name} = {{ path = "{crate_path}" }}
// anchor-lang = "0.31.1"
// "#,
//         debug_crate_name = debug_crate_name,
//         crate_name = crate_name,
//         crate_path = crate_path
//     );

//     cargo_toml
// }

use std::fs;

use anchor_idl::{Idl, IdlInstructionAccount, IdlInstructionAccountItem};


pub fn to_camel_case(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

// let program_name = idl
//     .metadata
//     .as_ref()
//     .and_then(|meta| meta.get("name"))
//     .and_then(|val| val.as_str())
//     .unwrap_or("unnamed_program");

pub fn load_idl(idl_path: &str) -> Result<Idl, Box<dyn std::error::Error>> {
    let idl_json = fs::read_to_string(idl_path).expect("failed to read IDL file");
    let idl: Idl = serde_json::from_str(&idl_json).expect("Failed to parseIDL");
    Ok(idl)
}


pub fn visit_account_item(item: &IdlInstructionAccountItem) -> Option<&IdlInstructionAccount> {
    match item {
        IdlInstructionAccountItem::Single(account) => Some(account),
        IdlInstructionAccountItem::Composite(group) => {
            for nested in &group.accounts {
                // Recursive till reached Single accout of the group
                if let Some(acc) = visit_account_item(nested) {
                    return Some(acc);
                }
            }
            None
        }
    }
}