use crate::{
    ast,
    ir::{
        function::{
            ir_generator::{rvalue_from_ast, IRGeneratingContext},
            IsIRStatement,
        },
        quantity::{self, local, Quantity},
        RegisterName,
    },
    utility::data_type::{self, Type},
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
use serde::{Deserialize, Serialize};
use std::fmt;
/// [`UnaryOperation`] represents a unary operation operator.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Eq, PartialEq, Clone, Hash, Deserialize, Serialize)]
pub struct UnaryCalculate {
    pub operation: UnaryOperation,
    pub operand: Quantity,
    pub to: RegisterName,
    pub data_type: Type,
}

impl IsIRStatement for UnaryCalculate {
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if let Quantity::RegisterName(operand) = &self.operand
            && operand == from
        {
            self.operand = to.clone();
        }
        if &self.to == from {
            self.to = to.unwrap_local();
        }
    }

    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        Some((self.to.clone(), self.data_type.clone()))
    }

    fn use_register(&self) -> Vec<RegisterName> {
        let mut result = Vec::new();
        if let Quantity::RegisterName(register) = &self.operand {
            result.push(register.clone());
        }
        result
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
    let rvalue_register = rvalue_from_ast(operand.as_ref(), ctx);
    let data_type = ctx.type_of_quantity(&rvalue_register);
    let result_register = ctx.next_register_with_type(&data_type);
    match operator.as_str() {
        "+" => {
            ctx.parent_context.next_register_id -= 1;
            ctx.symbol_table.register_type.remove(&result_register);
            return rvalue_register;
        }
        operator => {
            let operation = UNARY_OPERATION_MAP[operator];
            ctx.current_basic_block.append_statement(UnaryCalculate {
                operation,
                operand: rvalue_register,
                to: result_register.clone(),
                data_type,
            })
        }
    }
    result_register.into()
}
