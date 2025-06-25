use crate::utils::visit_account_item;
use anchor_idl::{IdlInstruction, IdlInstructionAccount};
use std::collections::HashMap;

pub struct InstructionAcountCode {
    pub bindings: Vec<String>,
    pub fields: Vec<String>,
    pub account_infos: Vec<String>,
}

impl InstructionAcountCode {
    pub fn generate_account_code(
        ix: &IdlInstruction,
        account_map: &HashMap<String, String>,
    ) -> InstructionAcountCode {
        let mut bindings = vec![];
        let mut fields = vec![];
        let mut accounts_info_clones = vec![];

        for acc in &ix.accounts {
            let account: &IdlInstructionAccount =
                visit_account_item(acc).expect("Failed te retrieve accounts");
            let acc_name: &String = &account.name;

            let mock_call: String = if acc_name.to_lowercase() == "system_program" {
                format!(r#"Box::leak(Box::new(mock_system_program()))"#)
            } else if account.signer {
                format!(r#"Box::leak(Box::new(mock_signer_account("{acc_name}")))"#)
            } else {
                let struct_name = account_map.get(acc_name)
                    .expect("Account struct name not found, maybe you don't have it in lib.rs and anchor-lldb cannot use it to derive account discriminator.");
                format!(
                    r#"Box::leak(Box::new(mock_pda_account::<{}>(&[b"{acc_name}"], &PROGRAM_ID, 64)))"#,
                    struct_name
                )
            };

            bindings.push(format!("let {acc_name} = {mock_call};"));

            let account_type = if account.signer {
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

        InstructionAcountCode {
            bindings,
            fields,
            account_infos: accounts_info_clones,
        }
    }
}
