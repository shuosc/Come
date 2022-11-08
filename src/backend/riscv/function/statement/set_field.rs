use crate::{
    backend::riscv::{function::FunctionCompileContext, register_assign::RegisterAssign, HasSize},
    ir::{self, quantity::Quantity},
    utility::data_type::Type,
};

/// Emit assembly code for a [`ir::statement::SetField`].
pub fn emit_code(set_field: &ir::statement::SetField, ctx: &mut FunctionCompileContext) -> String {
    let ir::statement::SetField {
        target,
        source,
        origin_root,
        field_chain,
        final_type,
    } = set_field;
    let mut code = String::new();
    let value_to_set = match source {
        Quantity::LocalVariableName(local) => ctx.local_assign.get(local).unwrap().clone(),
        Quantity::NumberLiteral(n) => {
            code.push_str(&format!("    li t1, {}\n", n));
            RegisterAssign::Register("t1".to_string())
        }
        Quantity::GlobalVariableName(_) => todo!(),
    };
    let value_to_be_setted = ctx.local_assign.get(origin_root).unwrap();
    let result_register = ctx.local_assign.get(target).unwrap();
    let mut current_offset = 0;
    let root_type_bytes = (field_chain[0].0.size(ctx.parent_context) + 7) / 8;
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
    let final_type_bytes = (final_type.size(ctx.parent_context) + 7) / 8;
    code.push_str(
        match (result_register, value_to_be_setted, value_to_set) {
            (RegisterAssign::Register(result), _, RegisterAssign::Register(to_set)) => {
                format!("    mv {}, {}\n", result, to_set)
            }
            (RegisterAssign::Register(result), _, RegisterAssign::StackValue(to_set)) => {
                format!("    lw {}, {}(sp)\n", result, to_set)
            }
            (
                RegisterAssign::MultipleRegisters(result),
                RegisterAssign::MultipleRegisters(to_be_setted),
                RegisterAssign::Register(value_to_set),
            ) => {
                let mut result_code = String::new();
                for i in 0..result.len() {
                    if i == current_offset_bytes / 4 {
                        result_code.push_str(&format!("    mv {}, {}\n", result[i], value_to_set));
                    } else {
                        result_code
                            .push_str(&format!("    mv {}, {}\n", result[i], to_be_setted[i]));
                    }
                }
                result_code
            }
            (
                RegisterAssign::MultipleRegisters(result),
                RegisterAssign::MultipleRegisters(to_be_setted),
                RegisterAssign::MultipleRegisters(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    result_code.push_str(&format!("    mv {}, {}\n", result[i], to_be_setted[i]));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    result_code.push_str(&format!(
                        "    mv {}, {}\n",
                        result[i],
                        value_to_set[i - current_offset_bytes / 4]
                    ));
                    i += 1;
                }
                while i < result.len() {
                    result_code.push_str(&format!("    mv {}, {}\n", result[i], to_be_setted[i]));
                    i += 1;
                }
                result_code
            }
            (
                RegisterAssign::MultipleRegisters(result),
                RegisterAssign::MultipleRegisters(to_be_setted),
                RegisterAssign::StackValue(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    result_code.push_str(&format!("    mv {}, {}\n", result[i], to_be_setted[i]));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    let offset = value_to_set + (i - current_offset_bytes / 4) * 4;
                    result_code.push_str(&format!("    lw {}, {}(sp)\n", result[i], offset));
                    i += 1;
                }
                while i < result.len() {
                    result_code.push_str(&format!("    mv {}, {}\n", result[i], to_be_setted[i]));
                    i += 1;
                }
                result_code
            }
            (
                RegisterAssign::MultipleRegisters(result),
                RegisterAssign::StackValue(to_be_setted),
                RegisterAssign::Register(value_to_set),
            ) => {
                let mut result_code = String::new();
                for (i, result_register) in result.iter().enumerate() {
                    if i == current_offset_bytes / 4 {
                        result_code
                            .push_str(&format!("    mv {}, {}\n", result_register, value_to_set));
                    } else {
                        let offset = to_be_setted + i * 4;
                        result_code
                            .push_str(&format!("    lw {}, {}(sp)\n", result_register, offset));
                    }
                }
                result_code
            }
            (
                RegisterAssign::MultipleRegisters(result),
                RegisterAssign::StackValue(to_be_setted),
                RegisterAssign::MultipleRegisters(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    let offset = to_be_setted + i * 4;
                    result_code.push_str(&format!("    lw {}, {}(sp)\n", result[i], offset));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    let index = i - current_offset_bytes / 4;
                    result_code
                        .push_str(&format!("    mv {}, {}\n", result[i], value_to_set[index]));
                    i += 1;
                }
                while i < result.len() {
                    let offset = to_be_setted + i * 4;
                    result_code.push_str(&format!("    lw {}, {}(sp)\n", result[i], offset));
                    i += 1;
                }
                result_code
            }
            (
                RegisterAssign::MultipleRegisters(result),
                RegisterAssign::StackValue(to_be_setted),
                RegisterAssign::StackValue(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    let offset = to_be_setted + i * 4;
                    result_code.push_str(&format!("    lw {}, {}(sp)\n", result[i], offset));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    let offset = value_to_set + (i - current_offset_bytes / 4) * 4;
                    result_code.push_str(&format!("    lw {}, {}(sp)\n", result[i], offset));
                    i += 1;
                }
                while i < result.len() {
                    let offset = to_be_setted + i * 4;
                    result_code.push_str(&format!("    lw {}, {}(sp)\n", result[i], offset));
                    i += 1;
                }
                result_code
            }
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::Register(_to_be_setted),
                RegisterAssign::Register(value_to_set),
            ) => format!("    sw {}, {}(sp)\n", value_to_set, result),
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::Register(_to_be_setted),
                RegisterAssign::StackValue(value_to_set),
            ) => format!(
                "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                value_to_set, result
            ),
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::MultipleRegisters(to_be_setted),
                RegisterAssign::Register(value_to_set),
            ) => {
                let mut result_code = String::new();
                for (i, to_be_setted_register) in to_be_setted.iter().enumerate() {
                    let offset = result + i * 4;
                    if i == current_offset_bytes / 4 {
                        result_code.push_str(&format!("    sw {}, {}(sp)\n", value_to_set, offset));
                    } else {
                        result_code.push_str(&format!(
                            "    sw {}, {}(sp)\n",
                            to_be_setted_register, offset
                        ));
                    }
                }
                result_code
            }
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::MultipleRegisters(to_be_setted),
                RegisterAssign::MultipleRegisters(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    let offset = result + i * 4;
                    result_code.push_str(&format!("    sw {}, {}(sp)\n", to_be_setted[i], offset));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    let offset = result + i * 4;
                    let index = i - current_offset_bytes / 4;
                    result_code
                        .push_str(&format!("    sw {}, {}(sp)\n", value_to_set[index], offset));
                    i += 1;
                }
                while i < to_be_setted.len() {
                    let offset = result + i * 4;
                    result_code.push_str(&format!("    sw {}, {}(sp)\n", to_be_setted[i], offset));
                    i += 1;
                }
                result_code
            }
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::MultipleRegisters(to_be_setted),
                RegisterAssign::StackValue(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    let offset = i * 4;
                    result_code.push_str(&format!(
                        "    sw {}, {}(sp)\n",
                        to_be_setted[i],
                        result + offset
                    ));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    let offset = i * 4;
                    let value_to_set_offset = (i - current_offset_bytes / 4) * 4;
                    result_code.push_str(&format!(
                        "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                        value_to_set + value_to_set_offset,
                        result + offset
                    ));
                    i += 1;
                }
                while i < to_be_setted.len() {
                    let offset = i * 4;
                    result_code.push_str(&format!(
                        "    sw {}, {}(sp)\n",
                        to_be_setted[i],
                        result + offset
                    ));
                    i += 1;
                }
                result_code
            }
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::StackValue(to_be_setted),
                RegisterAssign::Register(value_to_set),
            ) => {
                let mut result_code = String::new();
                for i in 0..current_offset_bytes / 4 {
                    let offset = i * 4;
                    if i == current_offset_bytes / 4 {
                        result_code.push_str(&format!(
                            "    sw {}, {}(sp)\n",
                            value_to_set,
                            result + offset
                        ));
                    } else {
                        result_code.push_str(&format!(
                            "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                            to_be_setted + offset,
                            result + offset
                        ));
                    }
                }
                result_code
            }
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::StackValue(to_be_setted),
                RegisterAssign::MultipleRegisters(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    let offset = i * 4;
                    result_code.push_str(&format!(
                        "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                        to_be_setted + offset,
                        result + offset
                    ));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    let offset = i * 4;
                    let index = i - current_offset_bytes / 4;
                    result_code.push_str(&format!(
                        "    sw {}, {}(sp)\n",
                        value_to_set[index],
                        result + offset
                    ));
                    i += 1;
                }
                while i < root_type_bytes / 4 {
                    let offset = i * 4;
                    result_code.push_str(&format!(
                        "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                        to_be_setted + offset,
                        result + offset
                    ));
                    i += 1;
                }
                result_code
            }
            (
                RegisterAssign::StackValue(result),
                RegisterAssign::StackValue(to_be_setted),
                RegisterAssign::StackValue(value_to_set),
            ) => {
                let mut result_code = String::new();
                let mut i = 0;
                while i < current_offset_bytes / 4 {
                    let offset = i * 4;
                    result_code.push_str(&format!(
                        "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                        to_be_setted + offset,
                        result + offset
                    ));
                    i += 1;
                }
                while i < current_offset_bytes / 4 + final_type_bytes / 4 {
                    let offset = i * 4;
                    let value_to_set_offset = (i - current_offset_bytes / 4) * 4;
                    result_code.push_str(&format!(
                        "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                        value_to_set + value_to_set_offset,
                        result + offset
                    ));
                    i += 1;
                }
                while i < root_type_bytes / 4 {
                    let offset = i * 4;
                    result_code.push_str(&format!(
                        "    lw t0, {}(sp)\n    sw t0, {}(sp)\n",
                        to_be_setted + offset,
                        result + offset
                    ));
                    i += 1;
                }
                result_code
            }
            (_, RegisterAssign::StackRef(_), _) => todo!(),
            (_, _, RegisterAssign::StackRef(_)) => todo!(),
            (_, RegisterAssign::Register(_), RegisterAssign::MultipleRegisters(_)) => {
                unreachable!()
            }
            (RegisterAssign::MultipleRegisters(_), RegisterAssign::Register(_), _) => {
                unreachable!()
            }
            (RegisterAssign::Register(_), RegisterAssign::MultipleRegisters(_), _) => {
                unreachable!()
            }
            (RegisterAssign::Register(_), _, RegisterAssign::MultipleRegisters(_)) => {
                unreachable!()
            }
            (RegisterAssign::StackRef(_), _, _) => unreachable!(),
        }
        .as_str(),
    );
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::{
        backend::riscv::{
            function::FunctionCompileContext, register_assign::RegisterAssign, Context,
        },
        ir::{self, LocalVariableName},
        utility::data_type::{Integer, Type},
    };

    #[test]
    fn emit_code_reg_whatever_reg() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S0".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
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
        // Simple struct
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::Register("t3".to_string()),
        );
        let set_field = ir::statement::SetField {
            target: LocalVariableName("a".to_string()),
            source: 42.into(),
            origin_root: LocalVariableName("b".to_string()),
            field_chain: vec![(Type::StructRef("S0".to_string()), 0)],
            final_type: Type::Integer(Integer {
                width: 32,
                signed: true,
            }),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(code, "    li t1, 42\n    mv t2, t1\n");
    }

    #[test]
    fn emit_code_reg_whatever_mem() {
        let mut ctx = Context {
            struct_definitions: HashMap::new(),
        };
        ctx.struct_definitions.insert(
            "S0".to_string(),
            ir::TypeDefinition {
                name: "S1".to_string(),
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
        // Simple struct
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::Register("t3".to_string()),
        );
        ctx.local_assign.insert(
            LocalVariableName("c".to_string()),
            RegisterAssign::StackValue(16),
        );
        let set_field = ir::statement::SetField {
            target: LocalVariableName("a".to_string()),
            source: LocalVariableName("c".to_string()).into(),
            origin_root: LocalVariableName("b".to_string()),
            field_chain: vec![(Type::StructRef("S0".to_string()), 0)],
            final_type: Type::Integer(Integer {
                width: 32,
                signed: true,
            }),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(code, "    lw t2, 16(sp)\n");
    }

    #[test]
    fn emit_code_multi_multi_reg() {
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
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t4".to_string(), "t5".to_string()]),
        );
        let set_field = ir::statement::SetField {
            target: LocalVariableName("a".to_string()),
            source: 42.into(),
            origin_root: LocalVariableName("b".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 0)],
            final_type: Type::Integer(Integer {
                width: 32,
                signed: true,
            }),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(code, "    li t1, 42\n    mv t2, t1\n    mv t3, t5\n");
        let set_field = ir::statement::SetField {
            target: LocalVariableName("a".to_string()),
            source: 42.into(),
            origin_root: LocalVariableName("b".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 1)],
            final_type: Type::Integer(Integer {
                width: 32,
                signed: true,
            }),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(code, "    li t1, 42\n    mv t2, t4\n    mv t3, t1\n");
    }

    #[test]
    fn emit_code_multi_multi_multi() {
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
            RegisterAssign::MultipleRegisters(vec![
                // S2.0
                "a0".to_string(),
                // S1.0
                "a1".to_string(),
                // S1.1
                "a2".to_string(),
                // S2.1
                "a3".to_string(),
            ]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        ctx.local_assign.insert(
            LocalVariableName("c".to_string()),
            RegisterAssign::MultipleRegisters(vec![
                // S2.0
                "a4".to_string(),
                // S1.0
                "a5".to_string(),
                // S1.1
                "a6".to_string(),
                // S2.1
                "a7".to_string(),
            ]),
        );
        let set_field = ir::statement::SetField {
            target: LocalVariableName("c".to_string()),
            source: LocalVariableName("b".to_string()).into(),
            origin_root: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 1)],
            final_type: Type::StructRef("S1".to_string()),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(
            code,
            "    mv a4, a0\n    mv a5, t2\n    mv a6, t3\n    mv a7, a3\n"
        );
    }

    #[test]
    fn emit_code_multi_multi_mem() {
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
            RegisterAssign::MultipleRegisters(vec![
                // S2.0
                "a0".to_string(),
                // S1.0
                "a1".to_string(),
                // S1.1
                "a2".to_string(),
                // S2.1
                "a3".to_string(),
            ]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            LocalVariableName("c".to_string()),
            RegisterAssign::MultipleRegisters(vec![
                // S2.0
                "a4".to_string(),
                // S1.0
                "a5".to_string(),
                // S1.1
                "a6".to_string(),
                // S2.1
                "a7".to_string(),
            ]),
        );
        let set_field = ir::statement::SetField {
            target: LocalVariableName("c".to_string()),
            source: LocalVariableName("b".to_string()).into(),
            origin_root: LocalVariableName("a".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 1)],
            final_type: Type::StructRef("S1".to_string()),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(
            code,
            "    mv a4, a0\n    lw a5, 16(sp)\n    lw a6, 20(sp)\n    mv a7, a3\n"
        );
    }

    #[test]
    fn emit_code_multi_mem_reg() {
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
        let mut ctx = FunctionCompileContext {
            parent_context: &mut ctx,
            local_assign: HashMap::new(),
            cleanup_label: None,
        };
        // Simple struct
        ctx.local_assign.insert(
            LocalVariableName("a".to_string()),
            RegisterAssign::MultipleRegisters(vec!["a0".to_string(), "a1".to_string()]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            LocalVariableName("c".to_string()),
            RegisterAssign::Register("t2".to_string()),
        );
        let set_field = ir::statement::SetField {
            target: LocalVariableName("a".to_string()),
            source: LocalVariableName("c".to_string()).into(),
            origin_root: LocalVariableName("b".to_string()),
            field_chain: vec![(Type::StructRef("S1".to_string()), 1)],
            final_type: Type::Integer(Integer {
                width: 32,
                signed: true,
            }),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(code, "    lw a0, 16(sp)\n    mv a1, t2\n");
    }

    #[test]
    fn emit_code_multi_mem_multi() {
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
                // S1.0
                "a1".to_string(),
                // S1.1
                "a2".to_string(),
                // S2.1
                "a3".to_string(),
            ]),
        );
        ctx.local_assign.insert(
            LocalVariableName("b".to_string()),
            RegisterAssign::StackValue(16),
        );
        ctx.local_assign.insert(
            LocalVariableName("c".to_string()),
            RegisterAssign::MultipleRegisters(vec!["t2".to_string(), "t3".to_string()]),
        );
        let set_field = ir::statement::SetField {
            target: LocalVariableName("a".to_string()),
            source: LocalVariableName("c".to_string()).into(),
            origin_root: LocalVariableName("b".to_string()),
            field_chain: vec![(Type::StructRef("S2".to_string()), 1)],
            final_type: Type::StructRef("S1".to_string()),
        };
        let code = emit_code(&set_field, &mut ctx);
        assert_eq!(
            code,
            "    lw a0, 16(sp)\n    mv a1, t2\n    mv a2, t3\n    lw a3, 28(sp)\n"
        );
        // todo: test other cases
    }
}
