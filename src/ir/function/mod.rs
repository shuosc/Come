use std::{collections::HashSet, fmt};

use crate::{
    ast::{self, statement},
    ir::{
        basic_block,
        basic_block::BasicBlock,
        quantity::{local, Local},
    },
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

use super::{
    quantity::{LocalOrGlobal, LocalOrNumberLiteral},
    statements::{Alloca, IRStatement},
};
mod assign;
mod declare;
mod expression;
mod if_statement;
mod return_statement;
mod while_statement;
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Parameter {
    pub name: Local,
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
        name: Local(name.clone()),
        data_type: data_type.clone(),
    }
}

#[enum_dispatch]
pub trait HasRegister {
    fn get_registers(&self) -> HashSet<Local>;
}

impl HasRegister for Parameter {
    fn get_registers(&self) -> HashSet<Local> {
        let mut result = HashSet::new();
        result.insert(self.name.clone());
        result
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
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

pub fn parse(code: &str) -> IResult<&str, FunctionDefinition> {
    map(
        tuple((
            tag("fn"),
            space0,
            parsing::ident,
            delimited(
                tag("("),
                separated_list0(tuple((multispace0, tag(","), multispace0)), parse_parameter),
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

pub struct IRGeneratingContext<'a> {
    pub parent_context: &'a mut super::IRGeneratingContext,

    pub done_basic_blocks: Vec<BasicBlock>,
    pub current_basic_block: BasicBlock,
}

pub fn from_ast(
    ast: &ast::function_definition::FunctionDefinition,
    ctx: &mut super::IRGeneratingContext,
) -> FunctionDefinition {
    let ast::function_definition::FunctionDefinition {
        name,
        parameters,
        return_type,
        content,
    } = ast;
    let mut ctx = IRGeneratingContext {
        parent_context: ctx,
        done_basic_blocks: Vec::new(),
        current_basic_block: BasicBlock {
            name: None,
            phis: Vec::new(),
            content: Vec::new(),
            terminator: None,
        },
    };
    let parameters: Vec<_> = parameters.iter().map(parameter_from_ast).collect();
    for param in &parameters {
        ctx.current_basic_block
            .content
            .push(IRStatement::Alloca(Alloca {
                to: Local(format!("{}_addr", param.name.0)),
                alloc_type: param.data_type.clone(),
            }));
        ctx.current_basic_block
            .content
            .push(IRStatement::Store(super::statements::Store {
                data_type: param.data_type.clone(),
                source: LocalOrNumberLiteral::Local(param.name.clone()),
                target: LocalOrGlobal::Local(Local(format!("{}_addr", param.name.0))),
            }));
    }
    compound_from_ast(content, &mut ctx);
    let mut content = ctx.done_basic_blocks;
    content.push(ctx.current_basic_block);
    FunctionDefinition {
        name: name.clone(),
        parameters,
        return_type: return_type.clone(),
        content,
    }
}

fn compound_from_ast(ast: &ast::statement::compound::Compound, ctx: &mut IRGeneratingContext) {
    for statement in &ast.0 {
        match statement {
            statement::Statement::Declare(declare) => declare::from_ast(declare, ctx),
            statement::Statement::Assign(assign) => assign::from_ast(assign, ctx),
            statement::Statement::Return(return_statement) => {
                return_statement::from_ast(return_statement, ctx)
            }
            statement::Statement::If(if_statement) => if_statement::from_ast(if_statement, ctx),
            statement::Statement::While(while_statement) => {
                while_statement::from_ast(while_statement, ctx)
            }
            statement::Statement::FunctionCall(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let code = "fn reduce(%s: S) -> i32 {
    %0 = loadfield i32 %s, 0
    %1 = loadfield i32 %s, 1
    %2 = add i32 %0, %1
    ret %2
}";
        let function_definition = parse(code).unwrap().1;
        println!("{:?}", function_definition);
        let code = "fn main() -> () {
    %1 = alloca i32
    store i32 1, address %1
    %2 = alloca i32
    store i32 2, address %2
    %3 = alloca i32
    %4 = load i32 %1
    %5 = load i32 %2
    %6 = add i32 %3, %4
WHILE_0_JUDGE:
    %7 = load i32 @g
    blt 0, %7, WHILE_0_TRUE, WHILE_0_FALSE
WHILE_0_TRUE:
    %8 = load i32 %3
    %9 = load i32 %1
    %10 = sub i32 %8, %9
    %11 = load i32 @g
    %12 = sub i32 %11, 1
    store i32 %12, address @g
    j WHILE_0_JUDGE
WHILE_0_FALSE:
    %13 = load i32 @g
    blt 0, %13, IF_0_TRUE, IF_0_FALSE
IF_0_TRUE:
    %14 = load i32 %1
    store i32 %14, address %2
    j IF_0_END
IF_0_FALSE:
    %14 = load i32 %1
    store i32 %14, address %2
    j IF_0_END
IF_0_END:
    ret
}";
        let function_definition = parse(code).unwrap().1;
        println!("{:?}", function_definition);
    }
}
