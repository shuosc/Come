use crate::{
    backend::riscv::from_ir::{function::FunctionCompileContext, register_assign::RegisterAssign},
    ir,
};

/// Emit assembly code for a [`ir::statement::BinaryCalculate`].
pub fn emit_code(
    statement: &ir::statement::BinaryCalculate,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::statement::BinaryCalculate {
        operation,
        operand1,
        operand2,
        to,
        data_type: _,
    } = statement;
    let mut result = String::new();
    let operand1_register = match operand1 {
        ir::quantity::Quantity::RegisterName(local) => {
            let logical_register_assign = ctx.local_assign.get(local).unwrap();
            match logical_register_assign {
                RegisterAssign::Register(physical_register) => physical_register.clone(),
                RegisterAssign::StackValue(offset) => {
                    result.push_str(&format!("    lw t0, {offset}(sp)\n"));
                    "t0".to_string()
                }
                RegisterAssign::MultipleRegisters(_) => unreachable!(),
                RegisterAssign::StackRef(_) => unreachable!(),
            }
        }
        ir::quantity::Quantity::GlobalVariableName(_global) => todo!(),
        ir::quantity::Quantity::NumberLiteral(literal) => {
            result.push_str(&format!("    li t0, {literal}\n"));
            "t0".to_string()
        }
    };
    let operand2_register = match operand2 {
        ir::quantity::Quantity::RegisterName(local) => {
            let logical_register_assign = ctx.local_assign.get(local).unwrap();
            if let RegisterAssign::Register(physical_register) = logical_register_assign {
                physical_register.clone()
            } else if let RegisterAssign::StackValue(offset) = logical_register_assign {
                result.push_str(&format!("    lw t1, {offset}(sp)\n"));
                "t1".to_string()
            } else {
                unreachable!()
            }
        }
        ir::quantity::Quantity::GlobalVariableName(_global) => todo!(),
        ir::quantity::Quantity::NumberLiteral(literal) => {
            result.push_str(&format!("    li t1, {literal}\n"));
            "t1".to_string()
        }
    };
    let to_register_assign = ctx.local_assign.get(to).unwrap();
    let to_register = match to_register_assign {
        RegisterAssign::Register(register) => register,
        RegisterAssign::StackRef(_stack_offset) => unreachable!(),
        RegisterAssign::StackValue(_stack_offset) => "t0",
        RegisterAssign::MultipleRegisters(_registers) => todo!(),
    };
    match operation {
        ir::statement::calculate::binary::BinaryOperation::Add => {
            result.push_str(&format!(
                "    add {to_register}, {operand1_register}, {operand2_register}\n"
            ));
        }
        ir::statement::calculate::binary::BinaryOperation::LessThan => {
            result.push_str(&format!(
                "    slt {to_register}, {operand1_register}, {operand2_register}\n"
            ));
        }
        ir::statement::calculate::binary::BinaryOperation::LessOrEqualThan => todo!(),
        ir::statement::calculate::binary::BinaryOperation::GreaterThan => todo!(),
        ir::statement::calculate::binary::BinaryOperation::GreaterOrEqualThan => todo!(),
        ir::statement::calculate::binary::BinaryOperation::Equal => {
            result.push_str(&format!(
                "    sub {to_register}, {operand1_register}, {operand2_register}\n"
            ));
            result.push_str(&format!("    seqz {to_register}, {to_register}\n"));
        }
        ir::statement::calculate::binary::BinaryOperation::NotEqual => todo!(),
        ir::statement::calculate::binary::BinaryOperation::Sub => todo!(),
        ir::statement::calculate::binary::BinaryOperation::Or => todo!(),
        ir::statement::calculate::binary::BinaryOperation::Xor => todo!(),
        ir::statement::calculate::binary::BinaryOperation::And => todo!(),
        ir::statement::calculate::binary::BinaryOperation::LogicalShiftLeft => todo!(),
        ir::statement::calculate::binary::BinaryOperation::LogicalShiftRight => todo!(),
        ir::statement::calculate::binary::BinaryOperation::AthematicShiftRight => todo!(),
    }
    if let RegisterAssign::StackValue(stack_offset) = to_register_assign {
        result.push_str(&format!("    sw {to_register}, {stack_offset}(sp)\n"));
    }
    result
}
