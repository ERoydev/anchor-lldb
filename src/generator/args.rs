use anchor_idl::{IdlInstruction, IdlType};

pub struct InstructionArgCode {
    pub args: Vec<String>,
    pub call_args: Vec<String>,
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
                _ => format!(
                    "/* unsupported arg type: {:?} */ Default::default()",
                    arg.ty
                ),
            };

            args.push(format!("let {arg_name} = {dummy};"));
            call_args.push(arg_name.to_string());
        }

        InstructionArgCode { args, call_args }
    }
}
