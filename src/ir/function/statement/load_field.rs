use crate::{
    ast::{
        self,
        expression::{FieldAccess, LValue},
    },
    ir::{
        function::{ir_generator::IRGeneratingContext, IsIRStatement},
        quantity::{local, Quantity, RegisterName},
    },
    utility::{
        data_type,
        data_type::Type,
        parsing::{self, in_multispace},
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};
use std::fmt;

use super::Load;

/// [`LoadField`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct LoadField {
    /// Where to store the result of the load.
    pub target: RegisterName,
    /// Where to load from.
    pub source: RegisterName,
    /// Access `.0`th field of the struct, which is `.1` type.
    pub field_chain: Vec<(Type, usize)>,
    /// `to`'s type.
    pub leaf_type: Type,
}

impl fmt::Display for LoadField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = load_field {} {}.[{}]",
            self.target,
            self.leaf_type,
            self.source,
            self.field_chain
                .iter()
                .map(|(t, i)| format!("{}.{}", t, i))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

impl IsIRStatement for LoadField {
    fn on_register_change(&mut self, from: &RegisterName, to: &Quantity) {
        if &self.target == from {
            self.target = to.clone().unwrap_local();
        }
        if &self.source == from {
            self.source = to.clone().unwrap_local();
        }
    }
    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        Some((self.target.clone(), self.leaf_type.clone()))
    }
    fn use_register(&self) -> Vec<RegisterName> {
        vec![self.source.clone()]
    }
}

fn parse_field(code: &str) -> IResult<&str, (Type, usize)> {
    map(
        tuple((data_type::parse, tag("."), parsing::integer)),
        |(t, _, i)| (t, i as usize),
    )(code)
}

/// Parse ir code to get a [`LoadField`] instruction.
pub fn parse(code: &str) -> IResult<&str, LoadField> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            tag("loadfield"),
            space1,
            data_type::parse,
            space1,
            local::parse,
            tag("."),
            delimited(
                tag("["),
                separated_list1(tag(","), in_multispace(parse_field)),
                tag("]"),
            ),
        )),
        |(to, _, _equal, _, _loadfield, _space, final_type, _, source, _dot, field_chain)| {
            LoadField {
                target: to,
                leaf_type: final_type,
                source,
                field_chain,
            }
        },
    )(code)
}

/// Generate IR from an [`ast::expression::FieldAccess`] AST node.
pub fn from_ast(ast: &ast::expression::FieldAccess, ctx: &mut IRGeneratingContext) -> RegisterName {
    let ast::expression::FieldAccess { from, name } = ast;
    let mut current = *from.clone();
    let mut field_chain_rev = vec![name.clone()];
    while let LValue::FieldAccess(field_access) = current {
        let FieldAccess { from, name } = field_access;
        field_chain_rev.push(name);
        current = *from.clone();
    }
    let root = if let LValue::VariableRef(root) = from.as_ref() {
        root
    } else {
        unreachable!()
    };
    let mut current_type = ctx.symbol_table.type_of_variable(root);
    let mut field_chain = vec![];
    for field in field_chain_rev.into_iter().rev() {
        let current_type_name = if let Type::StructRef(name) = &current_type {
            name
        } else {
            unreachable!()
        };
        let mapping = ctx
            .parent_context
            .type_definitions
            .get(current_type_name)
            .unwrap();
        let index = mapping.field_names.get(&field).unwrap();
        let data_type = mapping.field_types[*index].clone();
        field_chain.push((current_type, *index));
        current_type = data_type;
    }
    let root_variable_addr = ctx.symbol_table.current_variable_address_register(root);
    let load_to = ctx.next_register_with_type(&field_chain[0].0);
    ctx.current_basic_block.append_statement(Load {
        to: load_to.clone(),
        data_type: field_chain[0].0.clone(),
        from: root_variable_addr.into(),
    });
    let target = ctx.next_register_with_type(&field_chain[0].0);
    ctx.current_basic_block.append_statement(LoadField {
        target: target.clone(),
        source: load_to,
        field_chain,
        leaf_type: current_type,
    });
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("%1 = loadfield i32 %0.[S.0]").unwrap().1;
        assert_eq!(
            result,
            LoadField {
                target: RegisterName("1".to_string()),
                source: RegisterName("0".to_string()),
                field_chain: vec![(Type::StructRef("S".to_string()), 0)],
                leaf_type: data_type::I32.clone()
            },
        );

        let result = parse("%1 = loadfield i32 %0.[SS.1, S.0]").unwrap().1;
        assert_eq!(
            result,
            LoadField {
                target: RegisterName("1".to_string()),
                source: RegisterName("0".to_string()),
                field_chain: vec![
                    (Type::StructRef("SS".to_string()), 1),
                    (Type::StructRef("S".to_string()), 0)
                ],
                leaf_type: data_type::I32.clone()
            },
        );
    }
}
