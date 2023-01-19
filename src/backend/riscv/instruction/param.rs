use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::{is_alphanumeric, is_space},
    combinator::map,
    AsBytes, IResult,
};
use serde::{Deserialize, Serialize};

use crate::utility::parsing;

/// Parameter of an instruction.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Param {
    /// An unresolved symbol.
    Symbol(String),
    /// A register.
    Register(u8),
    /// A csr.
    Csr(u16),
    /// An immediate value.
    Immediate(i32),
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Param::Symbol(s) => write!(f, "{s}"),
            // todo: mapping csr/register -> name
            Param::Register(r) => write!(f, "x{r}"),
            Param::Csr(c) => write!(f, "0x{c:04x}"),
            Param::Immediate(i) => write!(f, "{i}"),
        }
    }
}

impl Param {
    pub fn unwrap_immediate(&self) -> i32 {
        match self {
            Param::Immediate(i) => *i,
            _ => panic!("Expected immediate!"),
        }
    }
    pub fn unwrap_register(&self) -> u8 {
        match self {
            Param::Register(r) => *r,
            _ => panic!("Expected register!"),
        }
    }
    pub fn unwrap_csr(&self) -> u16 {
        match self {
            Param::Csr(r) => *r,
            _ => panic!("Expected CSR!"),
        }
    }
}

fn parse_csr_bytes(code: &[u8]) -> IResult<&[u8], u16> {
    static CSRS: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();
    let csrs = CSRS.get_or_init(|| {
        let mut csrs = HashMap::new();
        let csrs_str = include_str!("../spec/csr.spec");
        for line in csrs_str
            .lines()
            .map(|it| it.trim())
            .filter(|it| !it.is_empty())
        {
            let (name, address) = line.split_once(' ').unwrap();
            let address = u16::from_str_radix(address.trim().trim_start_matches("0x"), 16).unwrap();
            csrs.insert(name, address);
        }
        csrs
    });
    let code = code.as_bytes();
    let (code, _) = take_while(is_space)(code)?;
    let (code, name) = take_while(is_alphanumeric)(code)?;
    let name_str = std::str::from_utf8(name).unwrap();
    if let Some(csr) = csrs.get(name_str) {
        Ok((code, *csr))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            code,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_csr(code: &str) -> IResult<&str, u16> {
    parse_csr_bytes(code.as_bytes())
        .map(|(code, csr)| (std::str::from_utf8(code).unwrap(), csr))
        .map_err(|_| nom::Err::Error(nom::error::Error::new(code, nom::error::ErrorKind::Tag)))
}

fn parse_register_bytes(code: &[u8]) -> IResult<&[u8], u8> {
    static REGISTERS: OnceLock<HashMap<&'static str, u8>> = OnceLock::new();
    let registers = REGISTERS.get_or_init(|| {
        let mut registers = HashMap::new();
        let registers_str = include_str!("../spec/registers.spec");
        for line in registers_str
            .lines()
            .map(|it| it.trim())
            .filter(|it| !it.is_empty())
        {
            let (index, names) = line.split_once(' ').unwrap();
            let names = names.split(',').map(|it| it.trim());
            for name in names {
                registers.insert(name, index.parse::<u8>().unwrap());
            }
        }
        registers
    });
    let code = code.as_bytes();
    let (code, _) = take_while(is_space)(code)?;
    let (code, name) = take_while(is_alphanumeric)(code)?;
    let name_str = std::str::from_utf8(name).unwrap();
    if let Some(register) = registers.get(name_str) {
        Ok((code, *register))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            code,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_register(code: &str) -> IResult<&str, u8> {
    parse_register_bytes(code.as_bytes())
        .map(|(code, register)| (std::str::from_utf8(code).unwrap(), register))
        .map_err(|_| nom::Err::Error(nom::error::Error::new(code, nom::error::ErrorKind::Tag)))
}

/// Parses asm code to get a [`Param`] instance.
pub fn parse(code: &str) -> IResult<&str, Param> {
    alt((
        map(parse_register, Param::Register),
        map(parse_csr, Param::Csr),
        map(parsing::in_multispace(parsing::integer), Param::Immediate),
        map(parsing::ident, Param::Symbol),
    ))(code)
}

/// There are some types we hope can be converted to [`Param`].
/// So we can make `instruction!` macro easily.
pub trait AsParam {
    fn as_param(&self) -> Param;
}

impl AsParam for i32 {
    fn as_param(&self) -> Param {
        Param::Immediate(*self)
    }
}

impl AsParam for &str {
    fn as_param(&self) -> Param {
        parse(self).unwrap().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csr() {
        assert_eq!(parse_csr("cycle"), Ok(("", 0xc00)));
        assert_eq!(parse_csr("cycleh"), Ok(("", 0xc80)));
        assert!(parse_csr("shu").is_err());
    }

    #[test]
    fn test_parse_register() {
        assert_eq!(parse_register("x0"), Ok(("", 0)));
        assert_eq!(parse_register("x1"), Ok(("", 1)));
        assert_eq!(parse_register("x8"), Ok(("", 8)));
        assert_eq!(parse_register("s0"), Ok(("", 8)));
        assert_eq!(parse_register("fp"), Ok(("", 8)));
        assert_eq!(parse_register("zero"), Ok(("", 0)));
        assert_eq!(parse_register("x26"), Ok(("", 26)));
        assert_eq!(parse_register("s10"), Ok(("", 26)));
        assert!(parse_register("s12").is_err());
    }

    #[test]
    fn test_parse() {
        assert_eq!(parse("x0"), Ok(("", Param::Register(0))));
        assert_eq!(parse("x1"), Ok(("", Param::Register(1))));
        assert_eq!(parse("x8"), Ok(("", Param::Register(8))));
        assert_eq!(parse("s0"), Ok(("", Param::Register(8))));
        assert_eq!(parse("fp"), Ok(("", Param::Register(8))));
        assert_eq!(parse("zero"), Ok(("", Param::Register(0))));
        assert_eq!(parse("x26"), Ok(("", Param::Register(26))));
        assert_eq!(parse("s10"), Ok(("", Param::Register(26))));
        assert_eq!(parse("cycle"), Ok(("", Param::Csr(0xc00))));
        assert_eq!(parse("cycleh"), Ok(("", Param::Csr(0xc80))));
        assert_eq!(parse("0"), Ok(("", Param::Immediate(0))));
        assert_eq!(parse("1"), Ok(("", Param::Immediate(1))));
        assert_eq!(parse("0x1"), Ok(("", Param::Immediate(1))));
        assert_eq!(parse("-0x1"), Ok(("", Param::Immediate(-1))));
        assert_eq!(
            parse("stupid_function"),
            Ok(("", Param::Symbol("stupid_function".to_string())))
        );
        assert!(parse(",").is_err());
    }
}
