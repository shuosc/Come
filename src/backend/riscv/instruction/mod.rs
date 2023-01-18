mod param;
mod param_transformer;
mod template;
use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use bitvec::vec::BitVec;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace1,
    combinator::{map, opt},
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

use crate::utility::parsing::{ident, in_multispace};

use param::ParsedParam;

use self::template::Template;

#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub name: String,
    pub params: Vec<ParsedParam>,
}

impl Display for Parsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.name)?;
        // todo: find a better way to handle `offset(register)` format
        const MEMORY_INSTRUCTIONS: [&str; 8] = ["lb", "lh", "lw", "lbu", "lhu", "sb", "sh", "sw"];
        if MEMORY_INSTRUCTIONS.contains(&self.name.as_str()) {
            write!(
                f,
                "{}, {}({})",
                self.params[0], self.params[1], self.params[2]
            )?;
        } else {
            for (i, param) in self.params.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{param}")?;
            }
        }
        Ok(())
    }
}

fn templates() -> &'static HashMap<&'static str, Template> {
    static TEMPLATE_MAPPING: OnceLock<HashMap<&'static str, Template>> = OnceLock::new();
    TEMPLATE_MAPPING.get_or_init(|| {
        let mut mapping = HashMap::new();
        let templates_str = include_str!("../spec/instructions.spec");
        let templates = templates_str
            .split('\n')
            .map(|it| it.trim())
            .filter(|it| !it.is_empty());
        for template in templates {
            let (name, template) = template.split_once(' ').unwrap();
            mapping.insert(name, template::parse(template.trim()).unwrap().1);
        }
        mapping
    })
}

pub fn parse(code: &str) -> IResult<&str, Parsed> {
    map(
        tuple((
            ident,
            multispace1,
            separated_list0(in_multispace(alt((tag(","), tag("(")))), param::parse),
            opt(tag(")")),
        )),
        |(name, _, params, _)| Parsed { name, params },
    )(code)
}

pub fn from_unparsed(unparsed: Unparsed) -> Parsed {
    Parsed {
        name: unparsed.name,
        params: unparsed
            .params
            .into_iter()
            .map(|it| param::parse(&it).unwrap().1)
            .collect(),
    }
}

pub fn parse_bin(bin: &[bool]) -> IResult<&[bool], Parsed> {
    // todo: speed up matching process
    if let Some((name, (rest, params))) = templates()
        .iter()
        .find_map(|(name, template)| template.parse_binary(bin).ok().map(|it| (name, it)))
    {
        Ok((
            rest,
            Parsed {
                name: name.to_string(),
                params,
            },
        ))
    } else {
        unreachable!()
    }
}

impl Parsed {
    pub fn binary(&self, address: u64) -> BitVec {
        let template = templates().get(self.name.as_str()).unwrap();
        template.render(&self.params, address).into_iter().collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Unparsed {
    pub name: String,
    pub params: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;
    #[test]
    fn test_parse() {
        let code = "lui x1, 0x1234";
        let (rest, parsed) = parse(code).unwrap();
        assert_eq!(rest, "");
        assert_eq!(parsed.name, "lui");
        assert_eq!(parsed.params.len(), 2);
        assert_eq!(parsed.params[0], ParsedParam::Register(1));
        assert_eq!(parsed.params[1], ParsedParam::Immediate(0x1234));
    }

    #[test]
    fn test_parse_bin() {
        let instruction = 0x009980b7u32;
        let instruction_bits: Vec<_> = instruction
            .view_bits::<Lsb0>()
            .into_iter()
            .by_vals()
            .collect();
        let parsed = parse_bin(&instruction_bits).unwrap().1;
        assert_eq!(
            parsed,
            Parsed {
                name: "lui".to_string(),
                params: vec![ParsedParam::Register(1), ParsedParam::Immediate(0x998)]
            }
        );
    }
}
