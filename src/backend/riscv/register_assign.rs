use std::collections::HashMap;

use crate::ir::{self, function::GenerateRegister, statement::ContentStatement};

use super::{Context, HasSize};

/// How a logical register is mapped to real hardware register or memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterAssign {
    /// The logical register is mapped to a hardware register.
    Register(String),
    /// The logical register is mapped to a set of hardware register.
    MultipleRegisters(Vec<String>),
    /// The logical register is actually alias to some stack space created by alloca and should only be used in `load` and `store`.
    StackRef(usize),
    /// The logical register is spilled to the stack.
    StackValue(usize),
}

/// Assign registers for a [`ir::FunctionDefinition`].
pub fn assign_register(
    ir_code: &ir::FunctionDefinition,
    ctx: &Context,
) -> (HashMap<ir::RegisterName, RegisterAssign>, usize) {
    let mut register_assign = HashMap::new();
    let mut argument_register_used = 0;
    for parameter in ir_code.parameters.iter() {
        // todo: handle parameters which takes more than 8 registers
        let take_registers = parameter.data_type.size(ctx) / 32;
        if take_registers == 1 {
            register_assign.insert(
                parameter.name.clone(),
                RegisterAssign::Register(format!("a{}", argument_register_used)),
            );
        } else {
            register_assign.insert(
                parameter.name.clone(),
                RegisterAssign::MultipleRegisters(
                    (0..take_registers)
                        .map(|it| format!("a{}", argument_register_used + it))
                        .collect(),
                ),
            );
        }
        argument_register_used += take_registers;
    }
    // todo: handle phi
    let statements = ir_code.content.iter().flat_map(|it| &it.content);
    let mut current_used_stack_space = 0;
    // we keep 0 and 1 for storing tempory result
    let mut next_temporary_register_id = 2;
    for statement in statements {
        // alloca statement means we do want a variable on the stack
        if let ContentStatement::Alloca(alloca) = statement {
            register_assign.insert(
                alloca.to.clone(),
                RegisterAssign::StackRef(current_used_stack_space),
            );
            current_used_stack_space += (alloca.alloc_type.size(ctx) + 7) / 8;
        } else {
            let logic_register = statement.generated_register();
            if let Some((logic_register, data_type)) = logic_register {
                let type_bytes = (data_type.size(ctx) + 7) / 8;
                let need_registers = type_bytes / 4;
                if next_temporary_register_id + need_registers - 1 <= 6 {
                    if need_registers == 1 {
                        register_assign.insert(
                            logic_register,
                            RegisterAssign::Register(format!("t{}", next_temporary_register_id)),
                        );
                    } else {
                        register_assign.insert(
                            logic_register,
                            RegisterAssign::MultipleRegisters(
                                (next_temporary_register_id
                                    ..next_temporary_register_id + need_registers)
                                    .map(|it| format!("t{}", it))
                                    .collect(),
                            ),
                        );
                    }
                    next_temporary_register_id += need_registers;
                } else {
                    register_assign.insert(
                        logic_register,
                        RegisterAssign::StackValue(current_used_stack_space),
                    );
                    current_used_stack_space += type_bytes;
                }
            }
        }
    }
    (register_assign, current_used_stack_space)
}
