use crate::{
    ast::{self, expression::VariableRef},
    ir::quantity::RegisterName,
    utility::{data_type, data_type::Type, parsing},
};
use basic_block::BasicBlock;
use ir_generator::{compound_from_ast, IRGeneratingContext};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, space0},
    combinator::map,
    multi::{many0, separated_list0},
    sequence::{delimited, tuple},
    IResult,
};
use parameter::Parameter;
use serde::{Deserialize, Serialize};
use statement::*;
use std::{
    fmt, mem,
    ops::{Index, IndexMut},
};

/// Data structure, parser and ir generator for basic blocks.
pub mod basic_block;
/// Functions to generate IR from AST.
pub mod ir_generator;
/// Data structure, parser and ir generator for function's parameter.
pub mod parameter;
/// Data structure, parser and ir generator for ir statements.
pub mod statement;

#[cfg(test)]
pub use statement::test_util;

/// Index to access statements in a function.
/// (block_index, statement_index)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionDefinitionIndex(pub usize, pub usize);

impl<U: Into<usize>, V: Into<usize>> From<(U, V)> for FunctionDefinitionIndex {
    fn from((block_index, statement_index): (U, V)) -> Self {
        Self(block_index.into(), statement_index.into())
    }
}

pub struct Iter<'a> {
    function_definition: &'a FunctionDefinition,
    index: FunctionDefinitionIndex,
}

impl<'a> Iter<'a> {
    fn next_index(&mut self) -> Option<FunctionDefinitionIndex> {
        let FunctionDefinitionIndex(block_index, statement_index) = self.index;
        if block_index >= self.function_definition.content.len() {
            None
        } else {
            let current_block = &self.function_definition.content[block_index];
            if statement_index >= current_block.content.len() {
                self.index = FunctionDefinitionIndex(block_index + 1, 0);
                self.next_index()
            } else {
                let result = mem::replace(
                    &mut self.index,
                    FunctionDefinitionIndex(block_index, statement_index + 1),
                );
                Some(result)
            }
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a IRStatement;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(FunctionDefinitionIndex(block_index, statement_index)) = self.next_index() {
            Some(&self.function_definition.content[block_index].content[statement_index])
        } else {
            None
        }
    }
}

pub struct FunctionDefinitionIndexEnumerate<'a>(Iter<'a>);

impl<'a> Iterator for FunctionDefinitionIndexEnumerate<'a> {
    type Item = (FunctionDefinitionIndex, &'a IRStatement);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.0.next_index();
        if let Some(FunctionDefinitionIndex(block_index, statement_index)) = index {
            Some((
                index.unwrap(),
                &self.0.function_definition.content[block_index].content[statement_index],
            ))
        } else {
            None
        }
    }
}

impl<'a> Iter<'a> {
    pub fn function_definition_index_enumerate(self) -> FunctionDefinitionIndexEnumerate<'a> {
        FunctionDefinitionIndexEnumerate(self)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionHeader {
    /// Name of the function.
    pub name: String,
    /// Parameters of the function.
    pub parameters: Vec<Parameter>,
    /// Return type of the function.
    pub return_type: Type,
}

/// [`FunctionDefinition`] represents a function definition.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub header: FunctionHeader,
    /// Basic blocks of the function.
    pub content: Vec<BasicBlock>,
}

impl fmt::Display for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}(", self.header.name)?;
        for (i, parameter) in self.header.parameters.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} {}", parameter.data_type, parameter.name)?;
        }
        writeln!(f, ") -> {} {{", self.header.return_type)?;
        for basic_block in &self.content {
            write!(f, "{basic_block}")?;
        }
        write!(f, "}}")
    }
}

impl Index<usize> for FunctionDefinition {
    type Output = BasicBlock;

    fn index(&self, index: usize) -> &Self::Output {
        &self.content[index]
    }
}

impl IndexMut<usize> for FunctionDefinition {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.content[index]
    }
}

impl Index<FunctionDefinitionIndex> for FunctionDefinition {
    type Output = IRStatement;

    fn index(&self, index: FunctionDefinitionIndex) -> &Self::Output {
        &self.content[index.0].content[index.1]
    }
}

impl IndexMut<FunctionDefinitionIndex> for FunctionDefinition {
    fn index_mut(&mut self, index: FunctionDefinitionIndex) -> &mut Self::Output {
        &mut self.content[index.0].content[index.1]
    }
}

impl FunctionDefinition {
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            function_definition: self,
            index: FunctionDefinitionIndex(0, 0),
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut IRStatement> {
        self.content.iter_mut().flat_map(|it| it.content.iter_mut())
    }

    pub fn remove(&mut self, index: &FunctionDefinitionIndex) {
        self.content[index.0].remove(index.1);
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
                separated_list0(parsing::in_multispace(tag(",")), parameter::parse),
                tag(")"),
            ),
            multispace0,
            tag("->"),
            multispace0,
            data_type::parse,
            multispace0,
            delimited(
                tag("{"),
                many0(parsing::in_multispace(basic_block::parse)),
                tag("}"),
            ),
        )),
        |(_, _, name, parameters, _, _, _, return_type, _, basic_blocks)| {
            formalize(FunctionDefinition {
                header: FunctionHeader {
                    name,
                    parameters,
                    return_type,
                },
                content: basic_blocks,
            })
        },
    )(code)
}

/// Generate [`FunctionDefinition`] from [`ast::function_definition::FunctionDefinition`].
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
    let parameters: Vec<_> = parameters.iter().map(parameter::from_ast).collect();
    for param in &parameters {
        let variable = VariableRef(param.name.0.clone());
        let param_register = RegisterName(variable.0.clone());
        ctx.symbol_table
            .register_type
            .insert(param_register.clone(), param.data_type.clone());
        let address_register = ctx
            .symbol_table
            .create_register_for(&variable, &param.data_type);
        ctx.current_basic_block.append_statement(Alloca {
            to: address_register.clone(),
            alloc_type: param.data_type.clone(),
        });
        ctx.current_basic_block.append_statement(Store {
            data_type: param.data_type.clone(),
            source: param_register.into(),
            target: address_register.into(),
        });
    }
    compound_from_ast(content, &mut ctx);
    let header = FunctionHeader {
        name: name.clone(),
        parameters,
        return_type: return_type.clone(),
    };
    ctx.parent_context
        .function_definitions
        .insert(name.clone(), header.clone());
    formalize(FunctionDefinition {
        header,
        content: ctx.done(),
    })
}

pub fn formalize(mut function: FunctionDefinition) -> FunctionDefinition {
    if function.content[0].name.is_none() {
        function.content[0].name = Some(format!("{}_entry", function.header.name));
    }
    for (this_index, next_index) in (0..function.content.len()).tuple_windows() {
        let next_item_name = function.content[next_index].name.clone().unwrap();
        let this = &mut function.content[this_index];
        if let Some(last) = this.content.last()
            && !matches!(
                last,
                IRStatement::Jump(_) | IRStatement::Branch(_) | IRStatement::Ret(_)
            )
        {
            this.content.push(
                Jump {
                    label: next_item_name,
                }
                .into(),
            )
        }
    }
    function
}

#[cfg(test)]
mod tests {
    use super::*;
    // todo: more tests
    #[test]
    fn test_parse() {
        let code = r"fn main() -> () {
              %0 = add i32 1, 2
            }";
        assert!(parse(code).is_ok());
    }
}
