use crate::{
    ast,
    ir::{
        function::{
            ir_generator::{rvalue_from_ast, IRGeneratingContext},
            GenerateRegister,
        },
        quantity::{self, local, Quantity},
        LocalVariableName,
    },
    utility::data_type::{self, Integer, Type},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    sequence::tuple,
    IResult,
};
use phf::phf_map;
use std::fmt;

/// [`UnaryOperation`] represents a unary operation operator.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UnaryOperation {
    Neg,
    Not,
}

static UNARY_OPERATION_MAP: phf::Map<&'static str, UnaryOperation> = phf_map! {
    "-" => UnaryOperation::Neg,
    "!" => UnaryOperation::Not,
};

/// Parse ir code to get a [`UnaryOperation`].
fn unary_operation(code: &str) -> IResult<&str, UnaryOperation> {
    alt((
        map(tag("neg"), |_| UnaryOperation::Neg),
        map(tag("not"), |_| UnaryOperation::Not),
    ))(code)
}

impl fmt::Display for UnaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperation::Neg => write!(f, "neg"),
            UnaryOperation::Not => write!(f, "not"),
        }
    }
}

/// [`UnaryCalculate`] represents the result of a unary operator.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UnaryCalculate {
    pub operation: UnaryOperation,
    pub operand: Quantity,
    pub to: LocalVariableName,
    pub data_type: Type,
}

impl GenerateRegister for UnaryCalculate {
    fn register(&self) -> Option<LocalVariableName> {
        Some(self.to.clone())
    }
}

impl fmt::Display for UnaryCalculate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {} {} {}",
            self.to, self.operand, self.data_type, self.operation
        )
    }
}

/// Parse ir code to get a [`UnaryCalculate`].
pub fn parse(code: &str) -> IResult<&str, UnaryCalculate> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            unary_operation,
            space1,
            data_type::parse,
            space1,
            quantity::parse,
        )),
        |(to_register, _, _, _, operation, _, data_type, _, operand)| UnaryCalculate {
            operation,
            operand,
            to: to_register,
            data_type,
        },
    )(code)
}

/// Generate a [`UnaryCalculate`] from an [`ast::expression::UnaryOperatorResult`],
/// and append it to the current basic block.
/// Return a [`Quantity`] which contains the result.
pub fn from_ast(
    ast: &ast::expression::UnaryOperatorResult,
    ctx: &mut IRGeneratingContext,
) -> Quantity {
    let ast::expression::unary_operator::UnaryOperatorResult { operator, operand } = ast;
    let result_register = ctx.parent_context.next_register();
    let rvalue_register = rvalue_from_ast(operand.as_ref(), ctx);
    match operator.as_str() {
        "+" => {
            ctx.parent_context.next_register_id -= 1;
            return rvalue_register;
        }
        operator => {
            let operation = UNARY_OPERATION_MAP[operator];
            ctx.current_basic_block.append_statement(UnaryCalculate {
                operation,
                operand: rvalue_register,
                to: result_register.clone(),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32,
                }),
            })
        }
    }
    result_register.into()
}
