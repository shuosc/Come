use crate::{
    backend::riscv::{function::FunctionCompileContext, HasSize},
    ir,
    utility::data_type::Type,
};

use super::logical_register_content_copy;

pub fn emit_code(
    statement: &ir::function::statement::LoadField,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::function::statement::LoadField {
        target: to,
        source,
        field_chain,
        leaf_type: final_type,
    } = statement;
    let mut current_offset = 0;
    for (field_parent_type, field_index) in field_chain {
        if let Type::StructRef(struct_name) = field_parent_type {
            let parent_type = ctx
                .parent_context
                .struct_definitions
                .get(struct_name)
                .unwrap();
            for calculating_field_index in 0..*field_index {
                current_offset +=
                    parent_type.fields[calculating_field_index].size(ctx.parent_context);
            }
        }
    }
    let current_offset_bytes = (current_offset + 7) / 8;
    let to_physical_register = ctx.local_assign.get(to).unwrap();
    let from_physical_register = ctx.local_assign.get(source).unwrap();
    let final_result_bytes = (final_type.size(ctx.parent_context) + 7) / 8;
    logical_register_content_copy(
        to_physical_register,
        from_physical_register,
        current_offset_bytes,
        final_result_bytes,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        backend::riscv::{register_assign::RegisterAssign, Context},
        ir::LocalVariableName,
        utility::data_type::{Integer, Type},
    };

    use super::*;

    #[test]
    fn emit_code_multiple_to_single() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S1".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::StructRef("S1".to_string()),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        // Simple struct
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::Register("t4".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 1)],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t4, t3\n");
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 0)],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t4, t2\n");
        // struct in struct
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec![
                // S2.0
                "a0".to_string(),
                // S1
                "a1".to_string(),
                "a2".to_string(),
                // S2.2
                "a3".to_string(),
            ]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::Register("t4".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![
                (Type::StructRef("S2".to_string()), 1),
                (Type::StructRef("S1".to_string()), 1),
            ],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t4, a2\n");
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 2)],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t4, a3\n");
    }

    #[test]
    fn emit_code_multiple_to_multiple() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S1".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::StructRef("S1".to_string()),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec![
                // S2.0
                "a0".to_string(),
                // S1
                "a1".to_string(),
                "a2".to_string(),
                // S2.2
                "a3".to_string(),
            ]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 1)],
            leaf_type: Type::StructRef("S1".to_string()),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t2, a1\n    mv t3, a2\n");
    }

    #[test]
    fn emit_code_single_to_memory() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S0".to_string(),
            ir::TypeDefinition {
                name: "S0".to_string(),
                fields: vec![Type::Integer(Integer {
                    width: 32,
                    signed: true,
                })],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::Register("a0".to_string()),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::StackValue(16),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S0".to_string()), 0)],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    sw a0, 16(sp)\n");
    }

    #[test]
    fn emit_code_memory_to_single() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S1".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::StructRef("S1".to_string()),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        // Simple struct
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 1)],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    lw t2, 20(sp)\n");
        // Struct of structs
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![
                (Type::StructRef("S2".to_string()), 1),
                (Type::StructRef("S1".to_string()), 1),
            ],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    lw t2, 24(sp)\n");
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 2)],
            leaf_type: Type::Integer(Integer {
                signed: true,
                width: 32,
            }),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    lw t2, 28(sp)\n");
    }

    #[test]
    fn emit_code_multiple_to_memory() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S1".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::StructRef("S1".to_string()),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec![
                "a0".to_string(),
                "a1".to_string(),
                "a2".to_string(),
                "a3".to_string(),
            ]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::StackValue(16),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 1)],
            leaf_type: Type::StructRef("S1".to_string()),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    sw a1, 20(sp)\n    sw a2, 24(sp)\n");
    }

    #[test]
    fn emit_code_memory_to_multiple() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S1".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::StructRef("S1".to_string()),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 1)],
            leaf_type: Type::StructRef("S1".to_string()),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    lw t2, 20(sp)\n    lw t3, 24(sp)\n");
    }

    #[test]
    fn emit_code_memory_to_memory() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S1".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                    Type::StructRef("S1".to_string()),
                    Type::Integer(Integer {
                        width: 32,
                        signed: true,
                    }),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::StackValue(32),
        );
        let ir_code = ir::function::statement::LoadField {
            target: LocalVariableName("b".to_string()),
            source: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 1)],
            leaf_type: Type::StructRef("S1".to_string()),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(
            result,
            r#"    lw t0, 20(sp)
    sw t0, 36(sp)
    lw t0, 24(sp)
    sw t0, 40(sp)
"#
        );
    }
}
