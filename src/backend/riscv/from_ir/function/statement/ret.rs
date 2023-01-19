use crate::{
    backend::riscv::from_ir::{function::FunctionCompileContext, register_assign::RegisterAssign},
    ir::{quantity::Quantity, statement::Ret},
};

/// Emit assembly code for a [`Ret`].
pub fn emit_code(ret: &Ret, ctx: &mut FunctionCompileContext) -> String {
    let mut result = if let Some(operand) = &ret.value {
        match operand {
            Quantity::RegisterName(local) => {
                let logical_register_assign = ctx.local_assign.get(local).unwrap();
                match logical_register_assign {
                    RegisterAssign::Register(register) => format!("    mv a0, {register}\n"),
                    RegisterAssign::StackValue(stack_offset) => {
                        format!("    lw a0, {stack_offset}(sp)\n")
                    }
                    RegisterAssign::StackRef(_) => unreachable!(),
                    RegisterAssign::MultipleRegisters(_) => todo!(),
                }
            }
            Quantity::NumberLiteral(n) => format!("    li a0, {n}\n"),
            Quantity::GlobalVariableName(_) => todo!(),
        }
    } else {
        String::new()
    };
    if let Some(cleanup_label) = &ctx.cleanup_label {
        result.push_str(&format!("    j {cleanup_label}\n"));
    } else {
        result.push_str("    ret\n");
    }
    result
}
