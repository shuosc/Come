/// Assign statement like `a = b`.
pub mod assign;
/// Compound statement like `{ let a: i32 = 1; }`. Used by [`If`] and [`While`].
pub mod compound;
/// Variable declaration like `let a: i32 = 1;`.
pub mod declare;
/// Expressions like `a + b`, used in other statements.
pub mod expression;
/// Function call statement like `a();`.
pub mod function_call;
/// `if` statement like `if a { let b: i32 = 1; } else { let c: i32 = 2; }`.
pub mod if_statement;
/// `return` statement like `return a;`.
pub mod return_statement;
/// `while` statement like `while a { let b: i32 = 1; a = a - 1; }`.
pub mod while_statement;

use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

pub use assign::Assign;
pub use declare::Declare;
pub use function_call::FunctionCall;
pub use if_statement::If;
pub use return_statement::Return;
pub use while_statement::While;

/// Tag trait for [`Statement`].
#[enum_dispatch]
trait IsStatement {}

/// A statement in the source code,
/// can be either a trivial statement which ends with a `;`
/// or a complex statement like `if` and `while`.
#[enum_dispatch(IsStatement)]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Statement {
    Declare,
    Assign,
    Return,
    If,
    While,
    FunctionCall,
}

/// Parse source code to get a [`Statement`].
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
