use crate::{
    backend::riscv::{register_assign::RegisterAssign, FunctionCompileContext},
    ir::{quantity::Quantity, statement::Ret},
};

/// Emit assembly code for a [`Ret`].
pub fn emit_code(ret: &Ret, ctx: &mut FunctionCompileContext) -> String {
    let mut result = if let Some(operand) = &ret.value {
        match operand {
            Quantity::LocalVariableName(local) => {
                let logical_register_assign = ctx.local_assign.get(local).unwrap();
                match logical_register_assign {
                    RegisterAssign::Register(register) => format!("    mv a0, {}\n", register),
                    RegisterAssign::StackValue(stack_offset) => {
                        format!("    lw a0, -{}(sp)\n", stack_offset)
                    }
                    RegisterAssign::StackRef(_) => unreachable!(),
                }
            }
            Quantity::NumberLiteral(n) => format!("    li a0, {}\n", n),
            Quantity::GlobalVariableName(_) => todo!(),
        }
    } else {
        String::new()
    };
    if let Some(cleanup_label) = &ctx.cleanup_label {
        result.push_str(&format!("    j {}\n", cleanup_label));
    } else {
        result.push_str("    ret\n");
    }
    result
}
