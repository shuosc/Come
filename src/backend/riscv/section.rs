use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult};
use std::fmt::Display;

/// The sections of the program.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SectionName {
    /// I REALLY want to call this section `code` TAT
    Text,
    Data,
    Rodata,
    Bss,
}

impl Display for SectionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SectionName::Text => ".text",
                SectionName::Data => ".data",
                SectionName::Rodata => ".rodata",
                SectionName::Bss => ".bss",
            }
        )
    }
}

pub fn parse_section(code: &str) -> IResult<&str, SectionName> {
    alt((
        map(tag(".text"), |_| SectionName::Text),
        map(tag(".data"), |_| SectionName::Data),
        map(tag(".rodata"), |_| SectionName::Rodata),
        map(tag(".bss"), |_| SectionName::Bss),
    ))(code)
}
