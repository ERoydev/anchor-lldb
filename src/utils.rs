use anchor_idl::{IdlInstructionAccount, IdlInstructionAccountItem};
use regex::Regex;
use toml_edit::{DocumentMut, Item, Value};
use std::{fs, path::Path};

// Just format util fn
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

// Used to extract the Single account recursively 
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

// Used to derive tha paths when using `anchor-lldb generate` instead of user passing the paths manually
pub fn infer_paths(package: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let dir = std::env::current_dir()?;

    let program_path = dir.join(format!("programs/{}", package));
    let cargo_path = program_path.join(format!("Cargo.toml"));
    let lib_rs_path = program_path.join("src/lib.rs");

    if !cargo_path.exists() {
        return Err(format!("Could not find Cargo.toml at {}", cargo_path.display()).into());
    }

    if !lib_rs_path.exists() {
        return Err(format!("Could not find src/lib.rs at {}", lib_rs_path.display()).into());
    }

    let mod_name = extract_program_mod_name(&lib_rs_path)?;
    let idl_path = dir.join(format!("target/idl/{}.json", mod_name));

    Ok((
        idl_path.to_string_lossy().to_string(),
        program_path.to_string_lossy().to_string(),
    ))
}

pub fn extract_program_mod_name(lib_rs_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(lib_rs_path)?;

    // Match #[program] followed by pub mod <name>
    let re = Regex::new(r#"(?m)#\s*\[program\]\s*pub\s+mod\s+([a-zA-Z_][a-zA-Z0-9_]*)"#)?;

    let captures = re
        .captures(&contents)
        .ok_or("Could not find #[program] pub mod <name> in lib.rs")?;

    Ok(captures[1].to_string())
}

// Add the crate inside the root cargo Toml 
pub fn inject_workspace_member(cargo_toml: &Path, member: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(cargo_toml)?;
    let mut doc = content.parse::<DocumentMut>()?;

    let members = doc["workspace"]["members"]
        .or_insert(Item::Value(Value::Array(Default::default())))
        .as_array_mut()
        .ok_or("`workspace.members` is not an array")?;

    let already_present = members.iter().any(|v| v.as_str() == Some(member));

    if !already_present {
        members.push(member);
        fs::write(cargo_toml, doc.to_string())?;
    } 
    Ok(())
}

pub fn cli_error<E: std::fmt::Display>(e: E) -> ! {
    eprintln!("anchor-lldb Error: {}", e);
    std::process::exit(1);
}