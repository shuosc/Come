use crate::{
    ir::quantity::{local_or_number_literal, LocalOrNumberLiteral},
    utility::parsing,
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
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BranchType {
    EQ,
    NE,
    LT,
    GE,
}

impl Display for BranchType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

fn branch_type(code: &str) -> IResult<&str, BranchType> {
    alt((
        map(tag("eq"), |_| BranchType::EQ),
        map(tag("ne"), |_| BranchType::NE),
        map(tag("lt"), |_| BranchType::LT),
        map(tag("ge"), |_| BranchType::GE),
    ))(code)
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Branch {
    pub branch_type: BranchType,
    pub operand1: LocalOrNumberLiteral,
    pub operand2: LocalOrNumberLiteral,
    pub success_label: String,
    pub failure_label: String,
}

impl Display for Branch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "b{} {}, {}, {}, {}",
            self.branch_type, self.operand1, self.operand2, self.success_label, self.failure_label
        )
    }
}

pub fn parse(code: &str) -> IResult<&str, Branch> {
    map(
        tuple((
            tag("b"),
            branch_type,
            space1,
            local_or_number_literal,
            space0,
            tag(","),
            space1,
            local_or_number_literal,
            space0,
            tag(","),
            space0,
            parsing::ident,
            space0,
            tag(","),
            space0,
            parsing::ident,
        )),
        |(
            _,
            branch_type,
            _,
            operand1,
            _,
            _,
            _,
            operand2,
            _,
            _,
            _,
            success_label,
            _,
            _,
            _,
            failure_label,
        )| Branch {
            branch_type,
            operand1,
            operand2,
            success_label,
            failure_label,
        },
    )(code)
}
