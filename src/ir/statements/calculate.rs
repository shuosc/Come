use crate::{
    ir::{
        function::HasRegister,
        quantity::{local, local_or_number_literal, Local, LocalOrNumberLiteral},
    },
    utility::{data_type, data_type::Type},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::map,
    sequence::tuple,
    IResult,
};
use std::{
    collections::HashSet,
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

fn binary_operation(code: &str) -> IResult<&str, BinaryOperation> {
    alt((
        map(tag("add"), |_| BinaryOperation::Add),
        map(tag("less"), |_| BinaryOperation::LessThan),
        map(tag("sub"), |_| BinaryOperation::Sub),
        map(tag("or"), |_| BinaryOperation::Or),
        map(tag("xor"), |_| BinaryOperation::Xor),
        map(tag("and"), |_| BinaryOperation::And),
        map(tag("sll"), |_| BinaryOperation::LogicalShiftLeft),
        map(tag("srl"), |_| BinaryOperation::LogicalShiftRight),
        map(tag("sra"), |_| BinaryOperation::AthematicShiftRight),
    ))(code)
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BinaryCalculate {
    pub operation: BinaryOperation,
    pub operand1: LocalOrNumberLiteral,
    pub operand2: LocalOrNumberLiteral,
    pub to: Local,
    pub data_type: Type,
}

impl HasRegister for BinaryCalculate {
    fn get_registers(&self) -> HashSet<Local> {
        let mut result = HashSet::new();
        result.insert(self.to.clone());
        result
    }
}

impl fmt::Display for BinaryCalculate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {} {} {}, {}",
            self.to, self.data_type, self.operation, self.operand1, self.operand2
        )
    }
}

pub fn parse_binary(code: &str) -> IResult<&str, BinaryCalculate> {
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
            local_or_number_literal,
            space0,
            tag(","),
            space0,
            local_or_number_literal,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UnaryOperation {
    Neg,
    Not,
}

fn unary_operation(code: &str) -> IResult<&str, UnaryOperation> {
    alt((
        map(tag("neg"), |_| UnaryOperation::Neg),
        map(tag("not"), |_| UnaryOperation::Not),
    ))(code)
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperation::Neg => write!(f, "neg"),
            UnaryOperation::Not => write!(f, "not"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UnaryCalculate {
    pub operation: UnaryOperation,
    pub operand: LocalOrNumberLiteral,
    pub to: Local,
    pub data_type: Type,
}

impl HasRegister for UnaryCalculate {
    fn get_registers(&self) -> HashSet<Local> {
        let mut result = HashSet::new();
        result.insert(self.to.clone());
        result
    }
}

impl fmt::Display for UnaryCalculate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {} {} {}",
            self.to, self.data_type, self.operand, self.operation
        )
    }
}

pub fn parse_unary(code: &str) -> IResult<&str, UnaryCalculate> {
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
            local_or_number_literal,
        )),
        |(to_register, _, _, _, operation, _, data_type, _, operand)| UnaryCalculate {
            operation,
            operand,
            to: to_register,
            data_type,
        },
    )(code)
}
