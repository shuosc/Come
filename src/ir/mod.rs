use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use enum_dispatch::enum_dispatch;

/// Data structure, parser and ir generator for functions.
pub mod function;
/// Data structure and parser for variables (global or local) and literals.
pub mod quantity;

mod global_definition;
mod integer_literal;
pub mod optimize;
mod type_definition;

use crate::{
    ast::{ASTNode, Ast},
    utility::data_type::{self, Integer},
};
pub use function::{statement, FunctionDefinition, FunctionHeader};
pub use global_definition::GlobalDefinition;
use nom::{
    branch::alt, character::complete::multispace0, combinator::map, multi::many0,
    sequence::delimited, IResult,
};
pub use quantity::RegisterName;
pub use type_definition::TypeDefinition;
use type_definition::TypeDefinitionMapping;

mod editor;
use self::function::parameter::Parameter;
pub use editor::analyzer;

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
pub fn from_ir_code(source: &str) -> IResult<&str, Vec<IR>> {
    many0(delimited(multispace0, parse, multispace0))(source)
}

/// Context for generating IR.
#[derive(Debug)]
pub struct IRGeneratingContext {
    /// Known function definitions.
    pub function_definitions: HashMap<String, FunctionHeader>,
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
        let mut built_in_functions = HashMap::new();
        built_in_functions.insert(
            "load_u32".to_string(),
            FunctionHeader {
                name: "load_u32".to_string(),
                parameters: vec![Parameter {
                    name: RegisterName("address".to_string()),
                    data_type: data_type::Type::Address,
                }],
                return_type: Integer {
                    signed: false,
                    width: 32,
                }
                .into(),
            },
        );
        built_in_functions.insert(
            "store_u32".to_string(),
            FunctionHeader {
                name: "store_u32".to_string(),
                parameters: vec![
                    Parameter {
                        name: RegisterName("address".to_string()),
                        data_type: data_type::Type::Address,
                    },
                    Parameter {
                        name: RegisterName("value".to_string()),
                        data_type: Integer {
                            signed: false,
                            width: 32,
                        }
                        .into(),
                    },
                ],
                return_type: data_type::Type::None,
            },
        );
        Self {
            type_definitions: HashMap::new(),
            global_definitions: HashMap::new(),
            next_register_id: 0,
            next_if_id: 0,
            next_loop_id: 0,
            function_definitions: built_in_functions,
        }
    }

    /// Generate a new local variable name.
    pub fn next_register(&mut self) -> RegisterName {
        let register_id = self.next_register_id;
        self.next_register_id += 1;
        RegisterName(format!("{register_id}"))
    }
}

impl Default for IRGeneratingContext {
    fn default() -> Self {
        Self::new()
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
