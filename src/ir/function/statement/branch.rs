use crate::{
    ir::{
        function::GenerateRegister,
        quantity::{self, Quantity},
        LocalVariableName,
    },
    utility::{data_type::Type, parsing},
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

/// Enum of all possible branch types.
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

/// [`Branch`] instruction.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Branch {
    /// Type of the branch.
    pub branch_type: BranchType,
    /// Left operand.
    pub operand1: Quantity,
    /// Right operand.
    pub operand2: Quantity,
    /// Label to jump to if the branch is taken.
    pub success_label: String,
    /// Label to jump to if the branch is not taken.
    pub failure_label: String,
}

impl GenerateRegister for Branch {
    fn register(&self) -> Option<(LocalVariableName, Type)> {
        None
    }
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

/// Parses ir code to get a [`Branch`] instruction.
pub fn parse(code: &str) -> IResult<&str, Branch> {
    map(
        tuple((
            tag("b"),
            branch_type,
            space1,
            quantity::parse,
            space0,
            tag(","),
            space1,
            quantity::parse,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("beq 1, 2, success, failure"),
            Ok((
                "",
                Branch {
                    branch_type: BranchType::EQ,
                    operand1: 1.into(),
                    operand2: 2.into(),
                    success_label: "success".to_string(),
                    failure_label: "failure".to_string(),
                }
            ))
        );
    }
}
