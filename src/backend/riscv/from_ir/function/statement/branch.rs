use crate::{
    backend::riscv::from_ir::{function::FunctionCompileContext, register_assign::RegisterAssign},
    ir::{
        quantity::Quantity,
        statement::{branch::BranchType, Branch},
    },
};

/// Emit assembly code for a [`Branch`].
pub fn emit_code(branch: &Branch, ctx: &mut FunctionCompileContext) -> String {
    let Branch {
        branch_type,
        operand1,
        operand2,
        success_label,
        failure_label,
    } = branch;
    let mut result = String::new();
    let branch_command = match branch_type {
        BranchType::EQ => "beq",
        BranchType::GE => "bge",
        BranchType::LT => "blt",
        BranchType::NE => "bne",
    };
    let operand1_register = match operand1 {
        Quantity::RegisterName(local) => {
            let logical_register_assign = ctx.local_assign.get(local).unwrap();
            match logical_register_assign {
                RegisterAssign::Register(register) => register.clone(),
                RegisterAssign::StackValue(stack_offset) => {
                    result.push_str(&format!("    lw t0, {stack_offset}(sp)\n"));
                    "t0".to_string()
                }
                RegisterAssign::StackRef(_) => unreachable!(),
                RegisterAssign::MultipleRegisters(_) => unreachable!(),
            }
        }
        Quantity::GlobalVariableName(_) => todo!(),
        Quantity::NumberLiteral(n) => {
            result.push_str(&format!("    li t0, {n}\n"));
            "t0".to_string()
        }
    };
    let operand2_register = match operand2 {
        Quantity::RegisterName(local) => {
            let logical_register_assign = ctx.local_assign.get(local).unwrap();
            match logical_register_assign {
                RegisterAssign::Register(register) => register.clone(),
                RegisterAssign::StackValue(stack_offset) => {
                    result.push_str(&format!("    lw t1, {stack_offset}(sp)\n"));
                    "t1".to_string()
                }
                RegisterAssign::StackRef(_) => unreachable!(),
                RegisterAssign::MultipleRegisters(_) => unreachable!(),
            }
        }
        Quantity::GlobalVariableName(_) => todo!(),
        Quantity::NumberLiteral(n) => {
            result.push_str(&format!("    li t1, {n}\n"));
            "t1".to_string()
        }
    };
    result.push_str(&format!(
        "    {branch_command} {operand1_register}, {operand2_register}, {success_label}\n"
    ));
    result.push_str(&format!("    j {failure_label}\n"));
    result
}
