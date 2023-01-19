use crate::{
    backend::riscv::from_ir::{
        function::FunctionCompileContext, register_assign::RegisterAssign, HasSize,
    },
    ir,
    utility::data_type::Type,
};

/// Emit assembly code for a [`ir::statement::SetField`].
pub fn emit_code(statement: &ir::statement::LoadField, ctx: &mut FunctionCompileContext) -> String {
    let ir::statement::LoadField {
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
    {
        let current_offset_bytes = current_offset_bytes;
        match (to_physical_register, from_physical_register) {
            (RegisterAssign::Register(to), RegisterAssign::Register(from)) => {
                format!("    mv {to}, {from}\n")
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
                    result.push_str(&format!("    lw {to_register}, {current_offset}(sp)\n"));
                    current_offset += 4;
                }
                result
            }
            (RegisterAssign::StackValue(to), RegisterAssign::Register(from)) => {
                format!("    sw {from}, {to}(sp)\n")
            }
            (RegisterAssign::StackValue(to), RegisterAssign::StackValue(from)) => {
                let mut result = String::new();
                let mut current_from = from + current_offset_bytes;
                let mut current_to = to + current_offset_bytes;
                for _ in 0..final_result_bytes / 4 {
                    result.push_str(&format!("    lw t0, {current_from}(sp)\n"));
                    result.push_str(&format!("    sw t0, {current_to}(sp)\n"));
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
                for from_item in from.iter().skip(start_at_register).take(final_result_words) {
                    result.push_str(&format!("    sw {from_item}, {current_offset}(sp)\n"));
                    current_offset += 4;
                }
                result
            }
            (RegisterAssign::MultipleRegisters(_), RegisterAssign::Register(_)) => {
                unreachable!(
                    "Unreasonable to copy a single register's content into multiple registers"
                )
            }
            (_, RegisterAssign::StackRef(_)) => {
                unreachable!("Unreasonable to copy a field of an address")
            }
            (RegisterAssign::StackRef(_), _) => {
                unreachable!("Unreasonable to copy a field into an address")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use std::collections::HashMap;

    use crate::{
        backend::riscv::from_ir::Context,
        ir::RegisterName,
        utility::data_type::{self, Type},
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
                fields: vec![data_type::I32.clone(), data_type::I32.clone()],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    data_type::I32.clone(),
                    Type::StructRef("S1".to_string()),
                    data_type::I32.clone(),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        // Simple struct
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        ctx.local_assign.insert(
            RegisterName("b".to_string()),
            RegisterAssign::Register("t4".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 1)],
            leaf_type: data_type::I32.clone(),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t4, t3\n");
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 0)],
            leaf_type: data_type::I32.clone(),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t4, t2\n");
        // struct in struct
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
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
            RegisterName("b".to_string()),
            RegisterAssign::Register("t4".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![
                (Type::StructRef("S2".to_string()), 1),
                (Type::StructRef("S1".to_string()), 1),
            ],
            leaf_type: data_type::I32.clone(),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    mv t4, a2\n");
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 2)],
            leaf_type: data_type::I32.clone(),
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
                fields: vec![data_type::I32.clone(), data_type::I32.clone()],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    data_type::I32.clone(),
                    Type::StructRef("S1".to_string()),
                    data_type::I32.clone(),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
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
            RegisterName("b".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
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
                fields: vec![data_type::I32.clone()],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
            RegisterAssign::Register("a0".to_string()),
        );
        ctx.local_assign.insert(
            RegisterName("b".to_string()),
            RegisterAssign::StackValue(16),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![(Type::StructRef("S0".to_string()), 0)],
            leaf_type: data_type::I32.clone(),
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
                fields: vec![data_type::I32.clone(), data_type::I32.clone()],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    data_type::I32.clone(),
                    Type::StructRef("S1".to_string()),
                    data_type::I32.clone(),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        // Simple struct
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            RegisterName("b".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 1)],
            leaf_type: data_type::I32.clone(),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    lw t2, 20(sp)\n");
        // Struct of structs
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            RegisterName("b".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![
                (Type::StructRef("S2".to_string()), 1),
                (Type::StructRef("S1".to_string()), 1),
            ],
            leaf_type: data_type::I32.clone(),
        };
        let result = emit_code(&ir_code, &mut ctx);
        assert_eq!(result, "    lw t2, 24(sp)\n");
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 2)],
            leaf_type: data_type::I32.clone(),
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
                fields: vec![data_type::I32.clone(), data_type::I32.clone()],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    data_type::I32.clone(),
                    Type::StructRef("S1".to_string()),
                    data_type::I32.clone(),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec![
                "a0".to_string(),
                "a1".to_string(),
                "a2".to_string(),
                "a3".to_string(),
            ]),
        );
        ctx.local_assign.insert(
            RegisterName("b".to_string()),
            RegisterAssign::StackValue(16),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
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
                fields: vec![data_type::I32.clone(), data_type::I32.clone()],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    data_type::I32.clone(),
                    Type::StructRef("S1".to_string()),
                    data_type::I32.clone(),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            RegisterName("b".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
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
                fields: vec![data_type::I32.clone(), data_type::I32.clone()],
            },
        );
        ctx.struct_definitions.insert(
            "S2".to_string(),
            ir::TypeDefinition {
                name: "S2".to_string(),
                fields: vec![
                    data_type::I32.clone(),
                    Type::StructRef("S1".to_string()),
                    data_type::I32.clone(),
                ],
            },
        );
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
            phi_constant_assign: HashMap::new(),
        };
        ctx.local_assign.insert(
            RegisterName("a".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            RegisterName("b".to_string()),
            RegisterAssign::StackValue(32),
        );
        let ir_code = ir::function::statement::LoadField {
            target: RegisterName("b".to_string()),
            source: RegisterName("a".to_string()),
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
