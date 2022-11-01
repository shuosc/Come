use std::collections::HashMap;

use crate::ir::{self, function::GenerateRegister, statement::IRStatement};

/// How a logical register is mapped to real hardware register or memory.
pub enum RegisterAssign {
    /// The logical register is mapped to a hardware register.
    Register(String),
    /// The logical register is actually alias to some stack space created by alloca and should only be used in `load` and `store`.
    StackRef(usize),
    /// The logical register is spilled to the stack.
    StackValue(usize),
}

/// Assign registers for a [`ir::FunctionDefinition`].
pub fn assign_register(
    ir_code: &ir::FunctionDefinition,
) -> (HashMap<ir::LocalVariableName, RegisterAssign>, usize) {
    let mut register_assign = HashMap::new();
    for (index, parameter) in ir_code.parameters.iter().enumerate() {
        register_assign.insert(
            parameter.name.clone(),
            RegisterAssign::Register(format!("a{}", index)),
        );
    }
    // todo: handle phi
    let statements = ir_code.content.iter().flat_map(|it| &it.content);
    let mut current_used_stack_space = 0;
    // we keep 0 and 1 for storing tempory result
    let mut next_temporary_register_id = 2;
    for statement in statements {
        // alloca statement means we do want a variable on the stack
        if let IRStatement::Alloca(alloca) = statement {
            register_assign.insert(
                alloca.to.clone(),
                RegisterAssign::StackRef(current_used_stack_space),
            );
            // todo: alloca.alloc_type.size() instead of 4
            current_used_stack_space += 4;
        } else {
            let logic_register = statement.register();
            if let Some(logic_register) = logic_register {
                if next_temporary_register_id <= 6 {
                    register_assign.insert(
                        logic_register,
                        RegisterAssign::Register(format!("t{}", next_temporary_register_id)),
                    );
                    next_temporary_register_id += 1;
                } else {
                    register_assign.insert(
                        logic_register,
                        RegisterAssign::StackValue(current_used_stack_space),
                    );
                    current_used_stack_space += 4;
                }
            }
        }
    }
    (register_assign, current_used_stack_space)
}
