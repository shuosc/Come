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
use serde::{Deserialize, Serialize};
use std::fmt;
/// [`BinaryOperation`] represents a binary operation operator.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BinaryOperation {
    Add,
    LessThan,
    LessOrEqualThan,
    GreaterThan,
    GreaterOrEqualThan,
    Equal,
    NotEqual,
    Sub,
    Or,
    Xor,
    And,
    LogicalShiftLeft,
    LogicalShiftRight,
    AthematicShiftRight,
}

impl BinaryOperation {
    pub fn inverse(self) -> Option<Self> {
        match self {
            BinaryOperation::LessThan => Some(BinaryOperation::GreaterOrEqualThan),
            BinaryOperation::LessOrEqualThan => Some(BinaryOperation::GreaterThan),
            BinaryOperation::GreaterThan => Some(BinaryOperation::LessOrEqualThan),
            BinaryOperation::GreaterOrEqualThan => Some(BinaryOperation::LessThan),
            BinaryOperation::Equal => Some(BinaryOperation::NotEqual),
            BinaryOperation::NotEqual => Some(BinaryOperation::Equal),
            _ => None,
        }
    }
}

static BINARY_OPERATION_MAP: phf::Map<&'static str, BinaryOperation> = phf_map! {
    "+" => BinaryOperation::Add,
    "-" => BinaryOperation::Sub,
    "==" => BinaryOperation::Equal,
    "<" => BinaryOperation::LessThan,
    "&&" => BinaryOperation::And,
};

impl fmt::Display for BinaryOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperation::Add => write!(f, "add"),
            BinaryOperation::LessThan => write!(f, "slt"),
            BinaryOperation::LessOrEqualThan => write!(f, "sle"),
            BinaryOperation::GreaterThan => write!(f, "sgt"),
            BinaryOperation::GreaterOrEqualThan => write!(f, "sge"),
            BinaryOperation::Equal => write!(f, "eq"),
            BinaryOperation::NotEqual => write!(f, "ne"),
            BinaryOperation::Sub => write!(f, "sub"),
            BinaryOperation::Or => write!(f, "or"),
            BinaryOperation::Xor => write!(f, "xor"),
            BinaryOperation::And => write!(f, "and"),
            BinaryOperation::LogicalShiftLeft => write!(f, "shl"),
            BinaryOperation::LogicalShiftRight => write!(f, "shr"),
            BinaryOperation::AthematicShiftRight => write!(f, "sra"),
        }
    }
}

/// Parse ir code to get a [`BinaryOperation`].
fn binary_operation(code: &str) -> IResult<&str, BinaryOperation> {
    alt((
        map(tag("add"), |_| BinaryOperation::Add),
        map(tag("slt"), |_| BinaryOperation::LessThan),
        map(tag("sle"), |_| BinaryOperation::LessOrEqualThan),
        map(tag("sgt"), |_| BinaryOperation::GreaterThan),
        map(tag("sge"), |_| BinaryOperation::GreaterOrEqualThan),
        map(tag("eq"), |_| BinaryOperation::Equal),
        map(tag("ne"), |_| BinaryOperation::NotEqual),
        map(tag("sub"), |_| BinaryOperation::Sub),
        map(tag("or"), |_| BinaryOperation::Or),
        map(tag("xor"), |_| BinaryOperation::Xor),
        map(tag("and"), |_| BinaryOperation::And),
        map(tag("sll"), |_| BinaryOperation::LogicalShiftLeft),
        map(tag("srl"), |_| BinaryOperation::LogicalShiftRight),
        map(tag("sra"), |_| BinaryOperation::AthematicShiftRight),
    ))(code)
}

/// [`BinaryCalculate`] represents a binary operation statement.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Deserialize, Serialize)]
pub struct BinaryCalculate {
    pub operation: BinaryOperation,
    pub operand1: Quantity,
    pub operand2: Quantity,
    pub to: RegisterName,
    pub data_type: Type,
}

impl IsIRStatement for BinaryCalculate {
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if let Quantity::RegisterName(op1) = &self.operand1
            && op1 == from
        {
            self.operand1 = to.clone();
        }
        if let Quantity::RegisterName(op2) = &self.operand2
            && op2 == from
        {
            self.operand2 = to.clone();
        }
        if &self.to == from {
            // I cannot imaging a case that this is necessary
            // But I still keep it here, just in case
            self.to = to.unwrap_local();
        }
    }

    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        Some((self.to.clone(), self.data_type.clone()))
    }

    fn use_register(&self) -> Vec<RegisterName> {
        let mut result = Vec::new();
        if let Quantity::RegisterName(register) = &self.operand1 {
            result.push(register.clone());
        }
        if let Quantity::RegisterName(register) = &self.operand2 {
            result.push(register.clone());
        }
        result
    }
}

impl fmt::Display for BinaryCalculate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {} {} {}, {}",
            self.to, self.operation, self.data_type, self.operand1, self.operand2
        )
    }
}

/// Parse ir code to get a [`BinaryCalculate`].
pub fn parse(code: &str) -> IResult<&str, BinaryCalculate> {
    map(
        tuple((
            local::parse,
            space0,
            tag("="),
            space0,
            binary_operation,
            space1,
            data_type::parse,
            space1,
            quantity::parse,
            space0,
            tag(","),
            space0,
            quantity::parse,
        )),
        |(to_register, _, _, _, operation, _, data_type, _, operand1, _, _, _, operand2)| {
            BinaryCalculate {
                operation,
                operand1,
                operand2,
                to: to_register,
                data_type,
            }
        },
    )(code)
}

/// Generate a [`BinaryOperation`] from an [`ast::expression::BinaryOperatorResult`],
/// and append it to the current basic block.
/// Return a [`RegisterName`] which contains the result.
pub fn from_ast(
    ast: &ast::expression::BinaryOperatorResult,
    ctx: &mut IRGeneratingContext,
) -> RegisterName {
    let ast::expression::BinaryOperatorResult { operator, lhs, rhs } = ast;
    let result_register = ctx.next_register_with_type(&Type::Integer(Integer {
        signed: true,
        width: 32,
    }));
    let left_register = rvalue_from_ast(lhs.as_ref(), ctx);
    let right_register = rvalue_from_ast(rhs.as_ref(), ctx);
    let operation = BINARY_OPERATION_MAP[operator.as_str()];
    ctx.current_basic_block.append_statement(BinaryCalculate {
        operation,
        operand1: left_register,
        operand2: right_register,
        to: result_register.clone(),
        data_type: Type::Integer(Integer {
            signed: true,
            width: 32,
        }),
    });
    result_register
}

#[cfg(test)]
pub mod test_util {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;

    pub fn new(target: &str, source1: &str, source2: &str) -> BinaryCalculate {
        BinaryCalculate {
            operation: BinaryOperation::Add,
            operand1: RegisterName(source1.to_string()).into(),
            operand2: RegisterName(source2.to_string()).into(),
            to: RegisterName(target.to_string()),
            data_type: data_type::I32.clone(),
        }
    }

    pub fn new_constant(target: &str) -> BinaryCalculate {
        BinaryCalculate {
            operation: BinaryOperation::Add,
            operand1: 1.into(),
            operand2: 2.into(),
            to: RegisterName(target.to_string()),
            data_type: data_type::I32.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]
    use super::*;

    #[test]
    fn test_parse() {
        let code = "%t0 = add i32 1, 2";
        let (_, binary_calculate) = parse(code).unwrap();
        assert_eq!(
            binary_calculate,
            BinaryCalculate {
                operation: BinaryOperation::Add,
                operand1: 1.into(),
                operand2: 2.into(),
                to: RegisterName("t0".to_string()),
                data_type: data_type::I32.clone(),
            }
        );
    }

    #[test]
    fn test_from_ast() {
        let ast = ast::expression::BinaryOperatorResult {
            operator: "+".to_string(),
            lhs: Box::new(ast::expression::IntegerLiteral(1).into()),
            rhs: Box::new(ast::expression::IntegerLiteral(2).into()),
        };
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        let mut ctx = super::IRGeneratingContext::new(&mut parent_ctx);
        let result = from_ast(&ast, &mut ctx);
        assert_eq!(result, RegisterName("0".to_string()));
    }
}
