use crate::{
    backend::riscv::{function::FunctionCompileContext, register_assign::RegisterAssign},
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
                RegisterAssign::Register(register) => register.clone(),
                RegisterAssign::StackValue(stack_offset) => {
                    result.push_str(&format!("lw t0, -{}(sp)\n", stack_offset));
                    "t0".to_string()
                }
                RegisterAssign::StackRef(_) => unreachable!(),
            }
        }
        ir::quantity::Quantity::GlobalVariableName(_) => todo!(),
        ir::quantity::Quantity::NumberLiteral(n) => {
            result.push_str(&format!("    li t0, {}\n", n));
            "t0".to_string()
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
