use crate::{
    backend::riscv::{register_assign::RegisterAssign, FunctionCompileContext},
    ir,
};

/// Emit assembly code for a [`ir::function::statement::Store`].
pub fn emit_code(
    statement: &ir::function::statement::Store,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::function::statement::Store {
        data_type: _,
        source,
        target,
    } = statement;
    let mut result = String::new();
    let source_register = match source {
        ir::quantity::Quantity::LocalVariableName(local) => {
            let local = ctx.local_assign.get(local).unwrap();
            match local {
                RegisterAssign::Register(register) => register,
                RegisterAssign::StackValue(stack_offset) => {
                    result.push_str(&format!("lw t0, -{}(sp)\n", stack_offset));
                    "t0"
                }
                RegisterAssign::StackRef(_) => unreachable!(),
            }
        }
        ir::quantity::Quantity::GlobalVariableName(_) => todo!(),
        ir::quantity::Quantity::NumberLiteral(n) => {
            result.push_str(&format!("    li t0, {}\n", n));
            "t0"
        }
    };
    let target_stack_offset = if let ir::quantity::Quantity::LocalVariableName(local) = target {
        let local = ctx.local_assign.get(local).unwrap();
        match local {
            RegisterAssign::StackRef(stack_offset) => stack_offset,
            RegisterAssign::Register(_) => todo!(),
            RegisterAssign::StackValue(_) => todo!(),
        }
    } else {
        // unreachable!() ?
        todo!()
    };
    result.push_str(&format!(
        "    sw {}, -{}(sp)\n",
        source_register, target_stack_offset
    ));
    result
}
