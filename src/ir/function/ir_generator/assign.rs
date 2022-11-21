use super::{rvalue_from_ast, IRGeneratingContext};
use crate::{
    ast::{self, expression::LValue},
    ir::function::statement,
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

fn field_chain_from_ast(
    mut root_type: Type,
    mut field_name_chain: impl Iterator<Item = String>,
    ctx: &mut IRGeneratingContext,
) -> (Vec<(Type, usize)>, Type) {
    let mut result = Vec::new();
    while let Type::StructRef(struct_ref) = &root_type {
        let current_mapping = ctx.parent_context.type_definitions.get(struct_ref).unwrap();
        let field_name = field_name_chain.next().unwrap();
        let index = current_mapping.field_names.get(&field_name).unwrap();
        result.push((root_type.clone(), *index));
        root_type = current_mapping.field_types.get(*index).unwrap().clone();
    }
    (result, root_type)
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
    let root_variable_addr = ctx
        .symbol_table
        .current_variable_address_register(root_variable);
    let root_variable_type = ctx.type_of_variable(root_variable);
    let root_variable_register = ctx.next_register_with_type(&root_variable_type);
    ctx.current_basic_block.append_statement(statement::Load {
        to: root_variable_register.clone(),
        data_type: root_variable_type.clone(),
        from: root_variable_addr.clone().into(),
    });
    let (field_chain, leaf_type) = field_chain_from_ast(
        root_variable_type.clone(),
        field_names.into_iter().rev(),
        ctx,
    );
    // then we will generate code for field set
    let set_field_result = ctx.next_register_with_type(&field_chain.first().unwrap().0);
    ctx.current_basic_block
        .append_statement(statement::SetField {
            source: rvalue_register,
            origin_root: root_variable_register,
            field_chain,
            final_type: leaf_type,
            target: set_field_result.clone(),
        });
    // and store the register back
    ctx.current_basic_block.append_statement(statement::Store {
        data_type: root_variable_type,
        source: set_field_result.into(),
        target: root_variable_addr.into(),
    });
}

fn to_variable(
    ctx: &mut IRGeneratingContext,
    variable_ref: &ast::expression::VariableRef,
    rvalue_register: &crate::ir::quantity::Quantity,
) {
    let data_type = ctx.symbol_table.type_of_variable(variable_ref);
    let lhs_address_register = ctx
        .symbol_table
        .current_variable_address_register(variable_ref);
    // generate store code
    ctx.current_basic_block.append_statement(statement::Store {
        source: rvalue_register.clone(),
        target: lhs_address_register.into(),
        data_type,
    });
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use std::collections::HashMap;

    use super::*;
    use crate::{
        ast::expression::{FieldAccess, IntegerLiteral, VariableRef},
        ir::{type_definition::TypeDefinitionMapping, RegisterName},
        utility::data_type,
    };

    #[test]
    fn test_to_variable() {
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        ctx.symbol_table
            .variable_types_stack
            .last_mut()
            .unwrap()
            .insert(VariableRef("a".to_string()), (data_type::I64.clone(), 0));
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
                target: RegisterName("a_0_addr".to_string()).into(),
                data_type: data_type::I64.clone(),
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
        let field_types = vec![data_type::I64.clone(), data_type::U32.clone()];
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
        let field_types = vec![Type::StructRef("S".to_string()), data_type::U64.clone()];
        parent_ctx.type_definitions.insert(
            "SS".to_string(),
            TypeDefinitionMapping {
                field_names,
                field_types,
            },
        );
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        ctx.symbol_table
            .variable_types_stack
            .last_mut()
            .unwrap()
            .insert(
                VariableRef("s".to_string()),
                (Type::StructRef("S".to_string()), 0),
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
                to: RegisterName("0".to_string()),
                data_type: Type::StructRef("S".to_string()),
                from: RegisterName("s_0_addr".to_string()).into()
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[1],
            statement::SetField {
                target: RegisterName("1".to_string()),
                source: 42.into(),
                origin_root: RegisterName("0".to_string()),
                field_chain: vec![(Type::StructRef("S".to_string()), 1)],
                final_type: data_type::U32.clone(),
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[2],
            statement::Store {
                data_type: Type::StructRef("S".to_string()),
                source: RegisterName("1".to_string()).into(),
                target: RegisterName("s_0_addr".to_string()).into(),
            }
            .into()
        );
        parent_ctx.next_register_id = 0;
        let mut ctx = IRGeneratingContext::new(&mut parent_ctx);
        ctx.symbol_table
            .variable_types_stack
            .last_mut()
            .unwrap()
            .insert(
                VariableRef("ss".to_string()),
                (Type::StructRef("SS".to_string()), 0),
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
                to: RegisterName("0".to_string()),
                data_type: Type::StructRef("SS".to_string()),
                from: RegisterName("ss_0_addr".to_string()).into()
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[1],
            statement::SetField {
                target: RegisterName("1".to_string()),
                source: 42.into(),
                origin_root: RegisterName("0".to_string()),
                field_chain: vec![
                    (Type::StructRef("SS".to_string()), 0),
                    (Type::StructRef("S".to_string()), 0),
                ],
                final_type: data_type::I64.clone(),
            }
            .into()
        );
        assert_eq!(
            basic_blocks[0].content[2],
            statement::Store {
                data_type: Type::StructRef("SS".to_string()),
                source: RegisterName("1".to_string()).into(),
                target: RegisterName("ss_0_addr".to_string()).into(),
            }
            .into()
        );
    }
}
