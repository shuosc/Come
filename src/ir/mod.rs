use std::{collections::HashMap, fmt};

use enum_dispatch::enum_dispatch;

/// Data structure, parser and ir generator for functions.
pub mod function;
mod global_definition;
mod integer_literal;
/// Data structure and parser for variables (global or local) and literals.
pub mod quantity;
mod type_definition;

pub use function::statement;
use crate::ast::{ASTNode, Ast};
pub use function::FunctionDefinition;
pub use global_definition::GlobalDefinition;
use nom::{
    branch::alt, character::complete::multispace0, combinator::map, multi::many0,
    sequence::delimited, IResult,
};
pub use quantity::LocalVariableName;
pub use type_definition::TypeDefinition;
use self::type_definition::TypeDefinitionMapping;

/// The root nodes of IR.
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

/// Parses the ir code to get an [`IR`].
pub fn parse(code: &str) -> IResult<&str, IR> {
    alt((
        map(type_definition::parse, IR::TypeDefinition),
        map(function::parse, IR::FunctionDefinition),
        map(global_definition::parse, IR::GlobalDefinition),
    ))(code)
}

/// Parses all the ir code to get a list of [`IR`].
pub fn from_source(source: &str) -> IResult<&str, Vec<IR>> {
    many0(delimited(multispace0, parse, multispace0))(source)
}

/// Context for generating IR.
pub struct IRGeneratingContext {
    /// Known struct types.
    pub type_definitions: HashMap<String, TypeDefinitionMapping>,
    /// Next local variable id.
    pub next_register_id: usize,
    /// Next `if` statement's id, used in generating label.
    pub next_if_id: usize,
    /// Next `while` statement's id, used in generating label.
    pub next_loop_id: usize,
}

impl IRGeneratingContext {
    pub fn new() -> Self {
        Self {
            type_definitions: HashMap::new(),
            next_register_id: 0,
            next_if_id: 0,
            next_loop_id: 0,
        }
    }

    pub fn next_register(&mut self) -> LocalVariableName {
        let register_id = self.next_register_id;
        self.next_register_id += 1;
        LocalVariableName(format!("{}", register_id))
    }
}

/// Generate IR from AST.
pub fn from_ast(ast: &Ast) -> Vec<IR> {
    let mut context = IRGeneratingContext::new();
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

// todo: tests