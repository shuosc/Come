use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use enum_dispatch::enum_dispatch;

/// Data structure, parser and ir generator for functions.
pub mod function;
mod global_definition;
mod integer_literal;
mod optimize;
pub mod analyzer;
/// Data structure and parser for variables (global or local) and literals.
pub mod quantity;
mod type_definition;
use self::type_definition::TypeDefinitionMapping;
use crate::ast::{ASTNode, Ast};
pub use function::{statement, FunctionDefinition};
pub use global_definition::GlobalDefinition;
use nom::{
    branch::alt, character::complete::multispace0, combinator::map, multi::many0,
    sequence::delimited, IResult,
};
pub use quantity::RegisterName;
pub use type_definition::TypeDefinition;

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
            IR::TypeDefinition(type_definition) => fmt::Display::fmt(&type_definition, f),
            IR::FunctionDefinition(function_definition) => {
                fmt::Display::fmt(&function_definition, f)
            }
            IR::GlobalDefinition(global_definition) => fmt::Display::fmt(&global_definition, f),
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
#[derive(Debug)]
pub struct IRGeneratingContext {
    /// Known struct types.
    pub type_definitions: HashMap<String, TypeDefinitionMapping>,
    /// Known global variables.
    pub global_definitions: HashMap<String, GlobalDefinition>,
    /// Next local variable id.
    pub next_register_id: usize,
    /// Next `if` statement's id, used in generating label.
    pub next_if_id: usize,
    /// Next `while` statement's id, used in generating label.
    pub next_loop_id: usize,
}

impl IRGeneratingContext {
    /// Creates a new, empty [`IRGeneratingContext`].
    pub fn new() -> Self {
        Self {
            type_definitions: HashMap::new(),
            global_definitions: HashMap::new(),
            next_register_id: 0,
            next_if_id: 0,
            next_loop_id: 0,
        }
    }

    /// Generate a new local variable name.
    pub fn next_register(&mut self) -> RegisterName {
        let register_id = self.next_register_id;
        self.next_register_id += 1;
        RegisterName(format!("{}", register_id))
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
