use std::fs;
use std::{collections::HashMap, path::PathBuf};
use walkdir::WalkDir;

use syn::{Attribute, Fields, File, Item, ItemStruct, PathArguments, Type, TypePath};

/*
- If the user keeps all their account structs and instruction context structs (i.e. the ones with #[derive(Accounts)]) in their lib.rs
    (or in files that are mod-declared from lib.rs), then:
    - The parser will successfully walk through everything and extract all the correct struct names automatically.

Note: Still not tested if Account Structs are separated in another folders
// TODO: Inspect the noted issue
*/

fn has_derive_accounts_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("derive") {
            let mut found = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("Accounts") {
                    found = true;
                }
                Ok(())
            });
            if found {
                return true;
            }
        }
    }
    false
}

/// Extracts a map: `field_name -> account_struct_name`
/// Example: `counter_acc_bro -> CounterAccount`
pub fn extract_account_struct_map(
    source_dir: &PathBuf,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut result = HashMap::new();

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        let syntax = syn::parse_file(&content)?;

        extract_from_file(&syntax, &mut result);
    }

    Ok(result)
}

fn extract_from_file(file: &File, map: &mut HashMap<String, String>) {
    for item in &file.items {
        if let Item::Struct(ItemStruct { attrs, fields, .. }) = item {
            if !has_derive_accounts_attr(attrs) {
                continue;
            }

            if let Fields::Named(named_fields) = fields {
                for field in &named_fields.named {
                    let Some(field_name) = field.ident.as_ref() else {
                        continue;
                    };

                    if let Type::Path(TypePath { path, .. }) = &field.ty {
                        let segments = &path.segments;
                        let Some(wrapper_segment) = segments.last() else {
                            continue;
                        };

                        // Look for Account<'info, X> or Signer<'info> or Program<'info, X>
                        if let PathArguments::AngleBracketed(args) = &wrapper_segment.arguments {
                            if wrapper_segment.ident == "Account"
                                || wrapper_segment.ident == "Program"
                            {
                                if args.args.len() == 2 {
                                    if let syn::GenericArgument::Type(Type::Path(inner_ty)) =
                                        &args.args[1]
                                    {
                                        if let Some(struct_name) = inner_ty.path.segments.last() {
                                            map.insert(
                                                field_name.to_string(),
                                                struct_name.ident.to_string(),
                                            );
                                        }
                                    }
                                }
                            } else if wrapper_segment.ident == "Signer" {
                                // Signers have only 1 type param: 'info
                                map.insert(field_name.to_string(), "Signer".to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}
