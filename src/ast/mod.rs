use enum_dispatch::enum_dispatch;
use nom::{
    branch::alt, character::complete::multispace0, combinator::map, multi::many0,
    sequence::delimited, IResult,
};

use self::{
    function_definition::FunctionDefinition, global_definition::VariableDefinition,
    type_definition::TypeDefinition,
};
pub use statement::expression;

/// Data structure and parser for a function definition.
pub mod function_definition;
/// Data structure and parser for a global variable definition.
pub mod global_definition;
/// Data structure and parser for a statement.
pub mod statement;
/// Data structure and parser for a type definition.
pub mod type_definition;

/// [`ASTNode`] is the level 0 nodes of the ast.
#[enum_dispatch]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum ASTNode {
    TypeDefinition,
    FunctionDefinition,
    GlobalVariableDefinition(VariableDefinition),
}

/// Parse source code to get a [`ASTNode`].
pub fn parse(code: &str) -> IResult<&str, ASTNode> {
    alt((
        map(type_definition::parse, ASTNode::TypeDefinition),
        map(function_definition::parse, ASTNode::FunctionDefinition),
        map(global_definition::parse, ASTNode::GlobalVariableDefinition),
    ))(code)
}

/// `Ast` is the root node of the ast.
pub type Ast = Vec<ASTNode>;

/// Parse source code to get a [`Ast`].
pub fn from_source(source: &str) -> IResult<&str, Ast> {
    // todo: Maybe we should use our own error type, and handle the situation that the remain code is not empty.
    many0(delimited(multispace0, parse, multispace0))(source)
}
