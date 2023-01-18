use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Section {
    Text,
    Data,
    Rodata,
    Bss,
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Section::Text => ".text",
                Section::Data => ".data",
                Section::Rodata => ".rodata",
                Section::Bss => ".bss",
            }
        )
    }
}

pub fn parse_section(code: &str) -> IResult<&str, Section> {
    alt((
        map(tag(".text"), |_| Section::Text),
        map(tag(".data"), |_| Section::Data),
        map(tag(".rodata"), |_| Section::Rodata),
        map(tag(".bss"), |_| Section::Bss),
    ))(code)
}
