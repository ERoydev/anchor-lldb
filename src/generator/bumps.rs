use anchor_idl::{IdlInstruction, IdlInstructionAccount, IdlSeed};

use crate::utils::visit_account_item;

pub struct InstructionBumpsCode {
    pub bump_fields: Vec<String>,
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
                        }
                        IdlSeed::Arg(seed_arg) => {
                            seed_exprs.push(format!("&{}::to_le_bytes()", seed_arg.path));
                        }
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
