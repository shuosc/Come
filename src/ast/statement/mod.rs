

use enum_dispatch::enum_dispatch;
pub mod assign;
pub mod compound;
pub mod declare;
pub mod function_call;
pub mod if_statement;
pub mod return_statement;
pub mod while_statement;

use assign::Assign;
use declare::Declare;
use nom::{branch::alt, combinator::map, IResult};
use return_statement::Return;



use self::{function_call::FunctionCall, if_statement::If, while_statement::While};

#[enum_dispatch]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Statement {
    Declare,
    Assign,
    Return,
    If,
    While,
    FunctionCall,
}

pub fn parse(code: &str) -> IResult<&str, Statement> {
    alt((
        map(declare::parse, Statement::Declare),
        map(assign::parse, Statement::Assign),
        map(return_statement::parse, Statement::Return),
        map(if_statement::parse, Statement::If),
        map(while_statement::parse, Statement::While),
        map(function_call::parse, Statement::FunctionCall),
    ))(code)
}
