use crate::utility::parsing;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{map, opt},
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult,
};
use std::fmt;

use super::statement::{self, IRStatement};

/// A basic block.
#[derive(Debug, Eq, PartialEq, Clone, Hash, Default)]
pub struct BasicBlock {
    /// Name of the basic block.
    pub name: Option<String>,
    /// Statements of the basic block.
    pub content: Vec<IRStatement>,
}

impl BasicBlock {
    pub fn new(name: String) -> Self {
        Self {
            name: Some(name),
            content: Vec::new(),
        }
    }
    /// Append a statement to the basic block.
    pub fn append_statement(&mut self, statement: impl Into<IRStatement>) {
        self.content.push(statement.into());
    }

    /// Whether the basic block is empty.
    pub fn empty(&self) -> bool {
        self.name.is_none() && self.content.is_empty()
    }

    /// Remove a statement from the basic block.
    pub fn remove(&mut self, index: usize) {
        self.content.remove(index);
    }

    pub fn is_branch(&self) -> bool {
        matches!(self.content.last(), Some(IRStatement::Branch(_)))
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            writeln!(f, "  {name}:")?;
        }
        for statement in &self.content {
            writeln!(f, "    {statement}")?;
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
    // `Basicblock` which
    //   - Has only a name and no content or
    //   - Has no name but only content
    //  are both valid.
    // However, `(opt(parse_tag), many0(IRStatement::parse))` can match literal nothing, which is not valid.
    // So we need to construct two parsers which stands for these two cases:

    // There is a tag, but the body can be empty.
    let has_tag = tuple((
        map(parse_tag, Some),
        multispace0,
        many0(parsing::in_multispace(statement::parse)),
    ));
    // There is no tag, but there exists at least one statement in the body.
    let has_ir = tuple((
        opt(parse_tag),
        multispace0,
        many1(parsing::in_multispace(statement::parse)),
    ));
    map(alt((has_tag, has_ir)), |(name, _, content)| BasicBlock {
        name,
        content,
    })(code)
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
