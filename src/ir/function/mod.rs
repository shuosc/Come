use self::{
    basic_block::BasicBlock,
    ir_generator::{compound_from_ast, IRGeneratingContext},
};
use crate::{
    ast,
    ir::quantity::{local, LocalVariableName},
    utility::{data_type, data_type::Type, parsing},
};
use enum_dispatch::enum_dispatch;
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, space0},
    combinator::map,
    multi::{many0, separated_list0},
    sequence::{delimited, tuple},
    IResult,
};
use statement::*;
use std::fmt;

/// Data structure, parser and ir generator for basic blocks.
mod basic_block;
/// Functions to generate IR from AST.
mod ir_generator;
/// Data structure, parser and ir generator for ir statements.
pub mod statement;

/// [`Parameter`] represents a function's parameter.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Parameter {
    /// Name of the parameter.
    pub name: LocalVariableName,
    /// Type of the parameter.
    pub data_type: Type,
}

fn parse_parameter(code: &str) -> IResult<&str, Parameter> {
    map(
        tuple((local::parse, space0, tag(":"), space0, data_type::parse)),
        |(name, _, _, _, data_type)| Parameter { name, data_type },
    )(code)
}

fn parameter_from_ast(ast: &ast::function_definition::Parameter) -> Parameter {
    let ast::function_definition::Parameter { name, data_type } = ast;
    Parameter {
        name: LocalVariableName(name.clone()),
        data_type: data_type.clone(),
    }
}

/// This trait should be implemented by IR statements that may generate a local variable.
#[enum_dispatch]
pub trait GenerateRegister {
    fn register(&self) -> Option<LocalVariableName>;
}

impl GenerateRegister for Parameter {
    fn register(&self) -> Option<LocalVariableName> {
        Some(self.name.clone())
    }
}

/// [`FunctionDefinition`] represents a function definition.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FunctionDefinition {
    /// Name of the function.
    pub name: String,
    /// Parameters of the function.
    pub parameters: Vec<Parameter>,
    /// Return type of the function.
    pub return_type: Type,
    /// Basic blocks of the function.
    pub content: Vec<BasicBlock>,
}

impl fmt::Display for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}(", self.name)?;
        for (i, parameter) in self.parameters.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} {}", parameter.data_type, parameter.name)?;
        }
        writeln!(f, ") -> {} {{", self.return_type)?;
        for basic_block in &self.content {
            write!(f, "{}", basic_block)?;
        }
        write!(f, "}}")
    }
}

/// Parse the ir code to get a [`FunctionDefinition`].
pub fn parse(code: &str) -> IResult<&str, FunctionDefinition> {
    map(
        tuple((
            tag("fn"),
            space0,
            parsing::ident,
            delimited(
                tag("("),
                separated_list0(parsing::in_multispace(tag(",")), parse_parameter),
                tag(")"),
            ),
            multispace0,
            tag("->"),
            multispace0,
            data_type::parse,
            multispace0,
            delimited(tag("{"), many0(basic_block::parse), tag("}")),
        )),
        |(_, _, name, parameters, _, _, _, return_type, _, basic_blocks)| FunctionDefinition {
            name,
            parameters,
            return_type,
            content: basic_blocks,
        },
    )(code)
}

/// Generate [`FunctionDefinition`] from [`ast::FunctionDefinition`].
pub fn from_ast(
    ast: &ast::function_definition::FunctionDefinition,
    ctx: &mut crate::ir::IRGeneratingContext,
) -> FunctionDefinition {
    let ast::function_definition::FunctionDefinition {
        name,
        parameters,
        return_type,
        content,
    } = ast;
    let mut ctx = IRGeneratingContext::new(ctx);
    let parameters: Vec<_> = parameters.iter().map(parameter_from_ast).collect();
    for param in &parameters {
        ctx.current_basic_block.append_statement(Alloca {
            to: LocalVariableName(format!("{}_addr", param.name.0)),
            alloc_type: param.data_type.clone(),
        });
        ctx.current_basic_block.append_statement(Store {
            data_type: param.data_type.clone(),
            source: param.name.clone().into(),
            target: LocalVariableName(format!("{}_addr", param.name.0)).into(),
        });
    }
    compound_from_ast(content, &mut ctx);
    FunctionDefinition {
        name: name.clone(),
        parameters,
        return_type: return_type.clone(),
        content: ctx.done(),
    }
}

// todo: test
