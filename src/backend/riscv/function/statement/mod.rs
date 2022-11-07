use crate::{backend::riscv::register_assign::RegisterAssign, ir};

use super::FunctionCompileContext;

/// Compile a binary operator.
mod binary_calculate;
/// Compile a load command.
mod load;
/// Compile a store command.
mod store;
/// Compile a unary operator.
mod unary_calculate;

mod load_field;

mod set_field;

pub fn logical_register_content_copy(
    to_physical_register: &RegisterAssign,
    from_physical_register: &RegisterAssign,
    current_offset_bytes: usize,
    final_result_bytes: usize,
) -> String {
    match (to_physical_register, from_physical_register) {
        (RegisterAssign::Register(to), RegisterAssign::Register(from)) => {
            format!("    mv {}, {}\n", to, from)
        }
        (RegisterAssign::Register(to), RegisterAssign::MultipleRegisters(from)) => {
            // todo: handle unaligned access
            let field_at_register = current_offset_bytes / 4;
            format!("    mv {}, {}\n", to, from[field_at_register])
        }
        (RegisterAssign::Register(to), RegisterAssign::StackValue(stack_offset)) => {
            format!(
                "    lw {}, {}(sp)\n",
                to,
                stack_offset + current_offset_bytes
            )
        }
        (RegisterAssign::MultipleRegisters(to), RegisterAssign::MultipleRegisters(from)) => {
            let mut field_at_register = current_offset_bytes / 4;
            let mut result = String::new();
            for to_register in to {
                result.push_str(&format!(
                    "    mv {}, {}\n",
                    to_register, from[field_at_register]
                ));
                field_at_register += 1;
            }
            result
        }
        (RegisterAssign::MultipleRegisters(to), RegisterAssign::StackValue(from)) => {
            let mut result = String::new();
            let mut current_offset = from + current_offset_bytes;
            for to_register in to {
                result.push_str(&format!("    lw {}, {}(sp)\n", to_register, current_offset));
                current_offset += 4;
            }
            result
        }
        (RegisterAssign::StackValue(to), RegisterAssign::Register(from)) => {
            format!("    sw {}, {}(sp)\n", from, to)
        }
        (RegisterAssign::StackValue(to), RegisterAssign::StackValue(from)) => {
            let mut result = String::new();
            let mut current_from = from + current_offset_bytes;
            let mut current_to = to + current_offset_bytes;
            for _ in 0..final_result_bytes / 4 {
                result.push_str(&format!("    lw t0, {}(sp)\n", current_from));
                result.push_str(&format!("    sw t0, {}(sp)\n", current_to));
                current_from += 4;
                current_to += 4;
            }
            result
        }
        (RegisterAssign::StackValue(to), RegisterAssign::MultipleRegisters(from)) => {
            let mut result = String::new();
            let mut current_offset = to + current_offset_bytes;
            let start_at_register = current_offset_bytes / 4;
            let final_result_words = final_result_bytes / 4;
            for idx in start_at_register..start_at_register + final_result_words {
                result.push_str(&format!("    sw {}, {}(sp)\n", from[idx], current_offset));
                current_offset += 4;
            }
            result
        }
        (RegisterAssign::MultipleRegisters(_), RegisterAssign::Register(_)) => {
            unreachable!("Unreasonable to copy a single register's content into multiple registers")
        }
        (_, RegisterAssign::StackRef(_)) => {
            unreachable!("Unreasonable to copy a field of an address")
        }
        (RegisterAssign::StackRef(_), _) => {
            unreachable!("Unreasonable to copy a field into an address")
        }
    }
}

/// Emit assembly code for a [`ir::function::statement::IRStatement`].
pub fn emit_code(
    statement: &ir::function::statement::IRStatement,
    ctx: &mut FunctionCompileContext,
) -> String {
    match statement {
        ir::statement::IRStatement::Alloca(_) => String::new(),
        ir::statement::IRStatement::UnaryCalculate(unary_calculate) => {
            unary_calculate::emit_code(unary_calculate, ctx)
        }
        ir::statement::IRStatement::BinaryCalculate(binary_calculate) => {
            binary_calculate::emit_code(binary_calculate, ctx)
        }
        ir::statement::IRStatement::Load(load) => load::emit_code(load, ctx),
        ir::statement::IRStatement::Store(store) => store::emit_code(store, ctx),
        ir::statement::IRStatement::LoadField(load_field) => load_field::emit_code(load_field, ctx),
        ir::statement::IRStatement::SetField(set_field) => set_field::emit_code(set_field, ctx),
    }
}
