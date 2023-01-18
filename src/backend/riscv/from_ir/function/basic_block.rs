use super::{statement, FunctionCompileContext};
use crate::{asm::riscv::register_assign::RegisterAssign, ir};

/// Emit assembly code for a [`ir::function::basic_block::BasicBlock`].
pub fn emit_code(
    basic_block: &ir::function::basic_block::BasicBlock,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::function::basic_block::BasicBlock { name, content } = basic_block;
    let mut result = String::new();
    if let Some(name) = name {
        result.push_str(format!("{}:\n", name).as_str());
    }
    if let Some((terminator, content)) = content.split_last() {
        for statement in content {
            let statement_code = statement::emit_code(statement, ctx);
            result.push_str(&statement_code);
        }
        result.push_str(&append_phi_insert(ctx, basic_block));
        let terminator_code = statement::emit_code(terminator, ctx);
        result.push_str(&terminator_code);
        result
    } else {
        String::new()
    }
}

fn append_phi_insert(
    ctx: &mut FunctionCompileContext,
    basic_block: &ir::function::basic_block::BasicBlock,
) -> String {
    let mut result = String::new();
    if let Some(phi_insert) = ctx
        .phi_constant_assign
        .get(basic_block.name.as_ref().unwrap())
    {
        for (register_assign, constant) in phi_insert {
            match register_assign {
                RegisterAssign::Register(register) => {
                    result.push_str(format!("    li {}, {}\n", register, constant).as_str());
                }
                RegisterAssign::StackRef(offset) => {
                    result.push_str(format!("    li t0, {}\n", constant).as_str());
                    result.push_str(format!("    sw t0, {}(sp)\n", offset).as_str());
                }
                RegisterAssign::StackValue(offset) => {
                    result.push_str(format!("    li t0, {}\n", constant).as_str());
                    result.push_str(format!("    sw t0, {}(sp)\n", offset).as_str());
                }
                RegisterAssign::MultipleRegisters(_) => {
                    unreachable!()
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;
    use std::collections::HashMap;

    use crate::{
        asm::riscv::{function::FunctionCompileContext, register_assign::RegisterAssign, Context},
        ir::{
            self,
            function::{basic_block::BasicBlock, test_util::*},
            statement::{phi::PhiSource, Phi},
            RegisterName,
        },
        utility::data_type::{self, Type},
    };

    #[test]
    fn phi_insert() {
        let function = ir::FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("f_entry".to_string()),
                    content: vec![binop_constant("reg1"), branch("bb1", "bb2")],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![binop_constant("reg2"), jump("bb3")],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![binop_constant("reg3"), jump("bb3")],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![Phi {
                        to: RegisterName("reg0".to_string()),
                        data_type: data_type::I32.clone(),
                        from: vec![
                            PhiSource {
                                name: 1.into(),
                                block: "bb1".to_string(),
                            },
                            PhiSource {
                                name: 2.into(),
                                block: "bb2".to_string(),
                            },
                        ],
                    }
                    .into()],
                },
            ],
        };
        let mut register_assign = HashMap::new();
        register_assign.insert(
            RegisterName("reg0".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        register_assign.insert(
            RegisterName("reg1".to_string()),
            RegisterAssign::Register("t3".to_string()),
        );
        register_assign.insert(
            RegisterName("reg2".to_string()),
            RegisterAssign::Register("t4".to_string()),
        );
        register_assign.insert(
            RegisterName("reg3".to_string()),
            RegisterAssign::Register("t5".to_string()),
        );
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        let mut context = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: register_assign,
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        context.phi_constant_assign.insert(
            "bb1".to_string(),
            vec![(RegisterAssign::Register("t2".to_string()), 1.into())],
        );
        context.phi_constant_assign.insert(
            "bb2".to_string(),
            vec![(RegisterAssign::Register("t2".to_string()), 2.into())],
        );
        let code = emit_code(&function.content[1], &mut context);
        assert_eq!(
            code,
            r#"bb1:
    li t0, 1
    li t1, 2
    add t4, t0, t1
    li t2, 1
    j bb3
"#
        );
        let code = emit_code(&function.content[2], &mut context);
        assert_eq!(
            code,
            r#"bb2:
    li t0, 1
    li t1, 2
    add t5, t0, t1
    li t2, 2
    j bb3
"#
        )
    }
}
