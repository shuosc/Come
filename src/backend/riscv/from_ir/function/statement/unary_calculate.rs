use crate::{
    backend::riscv::from_ir::{function::FunctionCompileContext, register_assign::RegisterAssign},
    ir::{self, statement::calculate::unary::UnaryOperation},
};

/// Emit assembly code for a [`ir::statement::UnaryCalculate`].
pub fn emit_code(
    statement: &ir::statement::UnaryCalculate,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::statement::UnaryCalculate {
        operation,
        operand,
        to,
        data_type: _,
    } = statement;
    let mut result = String::new();
    let operand_register = match operand {
        ir::quantity::Quantity::RegisterName(local) => {
            let logical_register_assign = ctx.local_assign.get(local).unwrap();
            if let RegisterAssign::Register(physical_register) = logical_register_assign {
                physical_register.clone()
            } else if let RegisterAssign::StackValue(offset) = logical_register_assign {
                result.push_str(&format!("    lw t0, {offset}(sp)\n"));
                "t0".to_string()
            } else {
                unreachable!()
            }
        }
        ir::quantity::Quantity::GlobalVariableName(_global) => todo!(),
        ir::quantity::Quantity::NumberLiteral(literal) => {
            result.push_str(&format!("    li t0, {literal}\n"));
            "t0".to_string()
        }
    };
    let to_register_assign = ctx.local_assign.get(to).unwrap();
    let to_register = match to_register_assign {
        RegisterAssign::Register(register) => register,
        RegisterAssign::StackRef(_stack_offset) => unreachable!(),
        RegisterAssign::StackValue(_stack_offset) => "t0",
        RegisterAssign::MultipleRegisters(_) => unreachable!(),
    };
    match operation {
        UnaryOperation::Neg => {
            result.push_str(&format!("    neg {to_register}, {operand_register}\n"));
        }
        UnaryOperation::Not => {
            result.push_str(&format!("    not {to_register}, {operand_register}\n"));
        }
    }
    if let RegisterAssign::StackValue(stack_offset) = to_register_assign {
        result.push_str(&format!("    sw {to_register}, {stack_offset}(sp)\n"));
    }
    result
}
