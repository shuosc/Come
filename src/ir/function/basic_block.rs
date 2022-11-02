use crate::{ir::LocalVariableName, utility::parsing};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{map, opt},
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult,
};
use std::{collections::HashSet, fmt};

use super::{
    statement::{
        self, parse_terminator,
        phi::{self, Phi},
        IRStatement, Terminator,
    },
    GenerateRegister,
};

/// A basic block.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BasicBlock {
    /// Name of the basic block.
    pub name: Option<String>,
    /// [`Phi`] statements of the basic block.
    pub phis: Vec<Phi>,
    /// Statements of the basic block.
    pub content: Vec<IRStatement>,
    /// Terminator of the basic block.
    pub terminator: Option<Terminator>,
}

impl BasicBlock {
    /// Create an empty basic block.
    pub fn new() -> Self {
        Self {
            name: None,
            phis: Vec::new(),
            content: Vec::new(),
            terminator: None,
        }
    }

    /// Registers created in the basic block.
    pub fn registers(&self) -> HashSet<LocalVariableName> {
        let mut result = HashSet::new();
        for phi in &self.phis {
            result.insert(phi.to.clone());
        }
        for statement in &self.content {
            result.extend(statement.register());
        }
        result
    }

    /// Append a statement to the basic block.
    pub fn append_statement(&mut self, statement: impl Into<IRStatement>) {
        self.content.push(statement.into());
    }

    /// Whether the basic block is empty.
    pub fn empty(&self) -> bool {
        self.name.is_none()
            && self.phis.is_empty()
            && self.content.is_empty()
            && self.terminator.is_none()
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}:", name)?;
        }
        for phi in &self.phis {
            writeln!(f, "    {}", phi)?;
        }
        for statement in &self.content {
            writeln!(f, "    {}", statement)?;
        }
        if let Some(terminator) = &self.terminator {
            writeln!(f, "    {}", terminator)?;
        }
        Ok(())
    }
}

/// Parse a basic block's name.
fn parse_tag(code: &str) -> IResult<&str, String> {
    map(pair(parsing::ident, tag(":")), |(_, name)| name.to_string())(code)
}

/// Parse the ir code to get a [`BasicBlock`].
pub fn parse(code: &str) -> IResult<&str, BasicBlock> {
    let has_tag = tuple((
        map(parse_tag, Some),
        multispace0,
        many0(parsing::in_multispace(phi::parse)),
        multispace0,
        many0(parsing::in_multispace(statement::parse_ir_statement)),
        multispace0,
        opt(parse_terminator),
        multispace0,
    ));
    let has_phi = tuple((
        opt(parse_tag),
        multispace0,
        many1(parsing::in_multispace(phi::parse)),
        multispace0,
        many0(parsing::in_multispace(statement::parse_ir_statement)),
        multispace0,
        opt(parse_terminator),
        multispace0,
    ));
    let has_ir = tuple((
        opt(parse_tag),
        multispace0,
        many0(parsing::in_multispace(phi::parse)),
        multispace0,
        many1(parsing::in_multispace(statement::parse_ir_statement)),
        multispace0,
        opt(parse_terminator),
        multispace0,
    ));
    let has_terminator = tuple((
        opt(parse_tag),
        multispace0,
        many0(parsing::in_multispace(phi::parse)),
        multispace0,
        many0(parsing::in_multispace(statement::parse_ir_statement)),
        multispace0,
        map(parse_terminator, Some),
        multispace0,
    ));
    map(
        alt((has_tag, has_phi, has_ir, has_terminator)),
        |(name, _, phis, _, content, _, terminator, _)| BasicBlock {
            name,
            phis,
            content,
            terminator,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let code = "%1 = alloca i32
        store i32 1, address %1
        %2 = alloca i32
        store i32 2, address %2
        %3 = alloca i32
        %4 = load i32 %1
        %5 = load i32 %2
        %6 = add i32 %3, %4";
        let bb = parse(code).unwrap();
        assert_eq!(bb.0, "");
        let code = "WHILE_0_JUDGE:
        %7 = load i32 @g
        blt 0, %7, WHILE_0_TRUE, WHILE_0_FALSE";
        let bb = parse(code).unwrap();
        assert_eq!(bb.0, "");
        let mut multiple_parser = many0(parse);
        let code = "    %1 = alloca i32
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
    ";
        let bbs = multiple_parser(code).unwrap();
        assert_eq!(bbs.0.trim(), "");
        assert_eq!(bbs.1.len(), 2);
    }
}
