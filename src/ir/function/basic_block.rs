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

use super::statement::{
    self, parse_terminator,
    phi::{self, Phi},
    ContentStatement, StatementRef, StatementRefMut, Terminator,
};

/// A basic block.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct BasicBlock {
    /// Name of the basic block.
    pub name: Option<String>,
    /// [`Phi`] statements of the basic block.
    pub phis: Vec<Phi>,
    /// Statements of the basic block.
    pub content: Vec<ContentStatement>,
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

    /// Append a statement to the basic block.
    pub fn append_statement(&mut self, statement: impl Into<ContentStatement>) {
        self.content.push(statement.into());
    }

    /// Whether the basic block is empty.
    pub fn empty(&self) -> bool {
        self.name.is_none()
            && self.phis.is_empty()
            && self.content.is_empty()
            && self.terminator.is_none()
    }

    // todo: board check
    pub fn index(&self, n: usize) -> StatementRef<'_> {
        if n < self.phis.len() {
            StatementRef::Phi(&self.phis[n])
        } else if n - self.phis.len() < self.content.len() {
            StatementRef::Content(&self.content[n])
        } else {
            StatementRef::Terminator(self.terminator.as_ref().unwrap())
        }
    }

    pub fn index_mut(&mut self, n: usize) -> StatementRefMut<'_> {
        if n < self.phis.len() {
            StatementRefMut::Phi(&mut self.phis[n])
        } else if n - self.phis.len() < self.content.len() {
            StatementRefMut::Content(&mut self.content[n])
        } else {
            StatementRefMut::Terminator(self.terminator.as_mut().unwrap())
        }
    }

    pub fn iter(&self) -> BasicBlockIterator<'_> {
        BasicBlockIterator { bb: self, index: 0 }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.phis.len() {
            self.phis.remove(index);
        } else {
            let index = index - self.phis.len();
            if index < self.content.len() {
                self.content.remove(index);
            } else if index == self.content.len() && self.terminator.is_some() {
                self.terminator = None;
            }
        }
    }
}

pub struct BasicBlockIterator<'a> {
    bb: &'a BasicBlock,
    index: usize,
}

impl<'a> Iterator for BasicBlockIterator<'a> {
    type Item = StatementRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.bb.phis.len() {
            let ret = StatementRef::Phi(&self.bb.phis[self.index]);
            self.index += 1;
            Some(ret)
        } else if self.index - self.bb.phis.len() < self.bb.content.len() {
            let ret = StatementRef::Content(&self.bb.content[self.index]);
            self.index += 1;
            Some(ret)
        } else if self.index - self.bb.phis.len() == self.bb.content.len() {
            let terminator = self.bb.terminator.as_ref()?;
            let ret = StatementRef::Terminator(terminator);
            self.index += 1;
            Some(ret)
        } else {
            None
        }
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            writeln!(f, "  {}:", name)?;
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
