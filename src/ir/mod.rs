use std::{collections::HashMap, fmt};

use enum_dispatch::enum_dispatch;

mod basic_block;
pub mod function;
mod global_definition;
mod integer_literal;
pub mod quantity;
pub mod statements;
mod type_definition;

use crate::ast::{ASTNode, Ast};
pub use basic_block::BasicBlock;
pub use function::FunctionDefinition;
pub use global_definition::GlobalDefinition;
use nom::{
    branch::alt, character::complete::multispace0, combinator::map, multi::many0,
    sequence::delimited, IResult,
};
pub use quantity::Local;
pub use statements::IRStatement;
pub use type_definition::TypeDefinition;

use self::type_definition::TypeDefinitionMapping;

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum IR {
    TypeDefinition,
    FunctionDefinition,
    GlobalDefinition,
}

impl fmt::Display for IR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IR::TypeDefinition(_type_definition) => todo!(),
            IR::FunctionDefinition(function_definition) => write!(f, "{}", function_definition),
            IR::GlobalDefinition(_global_definition) => todo!(),
        }
    }
}

pub fn parse(code: &str) -> IResult<&str, IR> {
    alt((
        map(type_definition::parse, IR::TypeDefinition),
        map(function::parse, IR::FunctionDefinition),
        map(global_definition::parse, IR::GlobalDefinition),
    ))(code)
}

pub fn from_source(source: &str) -> IResult<&str, Vec<IR>> {
    many0(delimited(multispace0, parse, multispace0))(source)
}

pub struct IRGeneratingContext {
    pub type_definitions: HashMap<String, TypeDefinitionMapping>,
    pub next_register_id: usize,
    pub next_if_id: usize,
    pub next_loop_id: usize,
}

pub fn from_ast(ast: &Ast) -> Vec<IR> {
    let mut context = IRGeneratingContext {
        type_definitions: HashMap::new(),
        next_register_id: 0,
        next_if_id: 0,
        next_loop_id: 0,
    };
    ast.iter()
        .map(|node| match node {
            ASTNode::TypeDefinition(type_definition) => {
                IR::TypeDefinition(type_definition::from_ast(type_definition, &mut context))
            }
            ASTNode::GlobalVariableDefinition(global_variable_definition) => IR::GlobalDefinition(
                global_definition::from_ast(global_variable_definition, &mut context),
            ),
            ASTNode::FunctionDefinition(ast) => {
                IR::FunctionDefinition(function::from_ast(ast, &mut context))
            }
        })
        .collect()
}
