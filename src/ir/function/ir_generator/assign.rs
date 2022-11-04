use super::{rvalue_from_ast, IRGeneratingContext};
use crate::{
    ast::{self, expression::LValue},
    ir::{function::statement, statement::set_field::Field, LocalVariableName},
    utility::data_type::Type,
};

/// Generate IR from an [`ast::statement::Assign`] AST node.
pub fn from_ast(ast: &ast::statement::Assign, ctx: &mut IRGeneratingContext) {
    // generate rhs code and get the result register
    let rvalue_register = rvalue_from_ast(&ast.rhs, ctx);
    match &ast.lhs {
        ast::expression::LValue::VariableRef(variable_ref) => {
            to_variable(ctx, variable_ref, &rvalue_register);
        }
        ast::expression::LValue::FieldAccess(field_access) => {
            to_field_access(field_access, ctx, rvalue_register);
        }
    }
}

fn field_indexes_and_leaf_type(
    mut root_type: Type,
    mut field_name_chain: impl Iterator<Item = String>,
    ctx: &mut IRGeneratingContext,
) -> (Vec<usize>, Type) {
    let mut result = Vec::new();
    while let Type::StructRef(struct_ref) = root_type {
        let current_mapping = ctx
            .parent_context
            .type_definitions
            .get(&struct_ref)
            .unwrap();
        let field_name = field_name_chain.next().unwrap();
        let index = current_mapping.field_names.get(&field_name).unwrap();
        result.push(*index);
        root_type = current_mapping.field_types.get(*index).unwrap().clone();
    }
    (result, root_type.clone())
}

fn to_field_access(
    field_access: &ast::expression::FieldAccess,
    ctx: &mut IRGeneratingContext,
    rvalue_register: crate::ir::quantity::Quantity,
) {
    // for field access, we will first load the "root" object
    let mut field_names = vec![field_access.name.clone()];
    let mut root = field_access.from.as_ref();
    while let LValue::FieldAccess(field_access) = &root {
        field_names.push(field_access.name.clone());
        root = field_access.from.as_ref();
    }
    let root_variable = if let LValue::VariableRef(x) = root {
        x
    } else {
        unreachable!()
    };
    let root_variable_addr = LocalVariableName(format!("{}_addr", root_variable.0));
    ctx.local_variable_types
        .insert(root_variable_addr.clone(), Type::Address);
    let root_variable_type = ctx.type_of_variable(root_variable);
    let root_variable_register = ctx.next_register_with_type(&root_variable_type);
    ctx.current_basic_block.append_statement(statement::Load {
        to: root_variable_register.clone(),
        data_type: root_variable_type.clone(),
        from: root_variable_addr.clone().into(),
    });
    let (field_indexes, leaf_type) = field_indexes_and_leaf_type(
        root_variable_type.clone(),
        field_names.into_iter().rev(),
        ctx,
    );
    // then we will generate code for field set
    let field = Field {
        name: root_variable_register.into(),
        index: field_indexes,
    };
    let result_register = ctx.next_register_with_type(&root_variable_type);
    ctx.current_basic_block
        .append_statement(statement::SetField {
            data_type: leaf_type,
            value: rvalue_register,
            field,
            result: result_register.clone(),
        });
    // and store the register back
    ctx.current_basic_block.append_statement(statement::Store {
        data_type: root_variable_type,
        source: result_register.into(),
        target: root_variable_addr.into(),
    });
}

fn to_variable(
    ctx: &mut IRGeneratingContext,
    variable_ref: &ast::expression::VariableRef,
    rvalue_register: &crate::ir::quantity::Quantity,
) {
    let data_type = ctx.type_of_variable(variable_ref);
    let lhs_address_register = LocalVariableName(format!("{}_addr", variable_ref.0));
    ctx.local_variable_types
        .insert(lhs_address_register.clone(), Type::Address);
    // generate store code
    ctx.current_basic_block.append_statement(statement::Store {
        source: rvalue_register.clone(),
        target: lhs_address_register.into(),
        data_type,
    });
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        ast::expression::{FieldAccess, IntegerLiteral, VariableRef},
        ir::{type_definition::TypeDefinitionMapping, LocalVariableName},
        utility::data_type::Integer,
    };

    #[test]
    fn test_to_variable() {
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        ctx.variable_types_stack.last_mut().unwrap().insert(
            VariableRef("a".to_string()),
            Type::Integer(Integer {
                signed: true,
                width: 64,
            }),
        );
        let ast = ast::statement::Assign {
            lhs: VariableRef("a".to_string()).into(),
            rhs: IntegerLiteral(42).into(),
        };
        from_ast(&ast, &mut ctx);
        let basic_blocks = ctx.done();
        assert_eq!(basic_blocks.len(), 1);
        assert_eq!(
            basic_blocks[0].content[0],
            statement::Store {
                source: 42.into(),
                target: LocalVariableName("a_addr".to_string()).into(),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 64,
                }),
            }
            .into()
        );
    }

    #[test]
    fn test_to_field_access() {
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut field_names = HashMap::new();
        field_names.insert("e0".to_string(), 0);
        field_names.insert("e1".to_string(), 1);
        let field_types = vec![
            Type::Integer(Integer {
                signed: true,
                width: 64,
            }),
            Type::Integer(Integer {
                signed: false,
                width: 32,
            }),
        ];
        parent_ctx.type_definitions.insert(
            "S".to_string(),
            TypeDefinitionMapping {
                field_names,
                field_types,
            },
        );
        let mut field_names = HashMap::new();
        field_names.insert("e2".to_string(), 0);
        field_names.insert("e3".to_string(), 1);
        let field_types = vec![
            Type::StructRef("S".to_string()),
            Type::Integer(Integer {
                signed: false,
                width: 64,
            }),
        ];
        parent_ctx.type_definitions.insert(
            "SS".to_string(),
            TypeDefinitionMapping {
                field_names,
                field_types,
            },
        );
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        ctx.variable_types_stack.last_mut().unwrap().insert(
            VariableRef("s".to_string()),
            Type::StructRef("S".to_string()),
        );
        let ast = ast::statement::Assign {
            lhs: LValue::FieldAccess(FieldAccess {
                from: Box::new(LValue::VariableRef(VariableRef("s".to_string()))),
                name: "e1".to_string(),
            }),
            rhs: IntegerLiteral(42).into(),
        };
        from_ast(&ast, &mut ctx);
        let basic_blocks = ctx.done();
        assert_eq!(basic_blocks.len(), 1);
        assert_eq!(
            basic_blocks[0].content[0],
            statement::Load {
                to: LocalVariableName("0".to_string()),
                data_type: Type::StructRef("S".to_string()),
                from: LocalVariableName("s_addr".to_string()).into()
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[1],
            statement::SetField {
                data_type: Type::Integer(Integer {
                    signed: false,
                    width: 32
                }),
                value: 42.into(),
                field: Field {
                    name: LocalVariableName("0".to_string()).into(),
                    index: vec![1],
                },
                result: LocalVariableName("1".to_string()),
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[2],
            statement::Store {
                data_type: Type::StructRef("S".to_string()),
                source: LocalVariableName("1".to_string()).into(),
                target: LocalVariableName("s_addr".to_string()).into(),
            }
            .into()
        );
        parent_ctx.next_register_id = 0;
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        ctx.variable_types_stack.last_mut().unwrap().insert(
            VariableRef("ss".to_string()),
            Type::StructRef("SS".to_string()),
        );
        let ast = ast::statement::Assign {
            lhs: LValue::FieldAccess(FieldAccess {
                from: Box::new(LValue::FieldAccess(FieldAccess {
                    from: Box::new(LValue::VariableRef(VariableRef("ss".to_string()))),
                    name: "e2".to_string(),
                })),
                name: "e0".to_string(),
            }),
            rhs: IntegerLiteral(42).into(),
        };
        from_ast(&ast, &mut ctx);
        let basic_blocks = ctx.done();
        assert_eq!(basic_blocks.len(), 1);
        assert_eq!(
            basic_blocks[0].content[0],
            statement::Load {
                to: LocalVariableName("0".to_string()),
                data_type: Type::StructRef("SS".to_string()),
                from: LocalVariableName("ss_addr".to_string()).into()
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[1],
            statement::SetField {
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 64,
                }),
                value: 42.into(),
                field: Field {
                    name: LocalVariableName("0".to_string()).into(),
                    index: vec![0, 0],
                },
                result: LocalVariableName("1".to_string())
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[2],
            statement::Store {
                data_type: Type::StructRef("SS".to_string()),
                source: LocalVariableName("1".to_string()).into(),
                target: LocalVariableName("ss_addr".to_string()).into(),
            }
            .into()
        );
    }
}
