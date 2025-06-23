use std::{fs, path::Path};

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

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
}

pub fn read_package_name(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let toml_content = fs::read_to_string(path)?;
    let cargo_toml: CargoToml = toml::from_str(&toml_content)?;
    Ok(cargo_toml.package.name)
}

// TODO: This can lead to bugs because Account name can be ShipmentIdCounter and this will result in Shipmentidcounter
pub fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}