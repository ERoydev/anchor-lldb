use crate::scripts::extract_account_struct_map::extract_account_struct_map;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct GeneratorConfig<'a> {
    pub program_path: &'a str,
    pub out_dir: PathBuf,
    pub src_dir: PathBuf,
    pub package_name: &'a str,
    pub account_map: HashMap<String, String>, // account name -> Account struct name -> Used to derive `DISCRIMINATOR` later when constructing `mock_pda`
}

impl<'a> GeneratorConfig<'a> {
    pub fn new(
        crate_path: &'a str,
        out_dir: PathBuf,
        src_dir: PathBuf,
        package_name: &'a str,
    ) -> Self {
        let crate_src_dir = Path::new(crate_path).join("src");
        let map = extract_account_struct_map(&crate_src_dir)
            .expect("Failed to extract account structs from source directory. Maybe you have defined your account structs somewhere else ?");

        GeneratorConfig {
            program_path: crate_path,
            out_dir,
            src_dir,
            package_name: package_name,
            account_map: map,
        }
    }
}
