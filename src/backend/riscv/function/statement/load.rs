use crate::{
    backend::riscv::{function::FunctionCompileContext, register_assign::RegisterAssign},
    ir,
};

/// Emit assembly code for a [`ir::statement::Load`].
pub fn emit_code(
    statement: &ir::function::statement::Load,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::function::statement::Load {
        to,
        data_type: _,
        from,
    } = statement;
    let mut result = String::new();
    let from_stack_offset = match from {
        ir::quantity::Quantity::LocalVariableName(local) => {
            let physical_register = ctx.local_assign.get(local).unwrap();
            match physical_register {
                RegisterAssign::Register(_register) => {
                    // unreachable!() ?
                    todo!()
                }
                RegisterAssign::StackRef(stack_offset) => stack_offset,
                RegisterAssign::StackValue(_) => {
                    // unreachable!() ?
                    todo!()
                }
            }
        }
        ir::quantity::Quantity::GlobalVariableName(_global) => todo!(),
        ir::quantity::Quantity::NumberLiteral(_literal) => unreachable!(),
    };
    let to_physical = ctx.local_assign.get(to).unwrap();
    let to_register = match to_physical {
        RegisterAssign::Register(register) => register.to_string(),
        RegisterAssign::StackValue(_) => "t0".to_string(),
        RegisterAssign::StackRef(_) => unreachable!(),
    };
    result.push_str(&format!(
        "    lw {}, -{}(sp)\n",
        to_register, from_stack_offset
    ));
    if let RegisterAssign::StackValue(stack_offset) = to_physical {
        result.push_str(&format!("    sw {}, -{}(sp)\n", to_register, stack_offset));
    }
    result
}
