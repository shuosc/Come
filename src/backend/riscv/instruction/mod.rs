/// Parameter information and parser.
pub(crate) mod param;
/// Parameter transformers are used to convert back and forth from a parameter to fields in
/// a binary form of instruction.
mod param_transformer;
/// Instruction template information, parser and related functions.
mod template;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
    sync::OnceLock,
};

use bitvec::prelude::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace1,
    combinator::{map, opt},
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

use crate::{
    binary_format::clef::{PendingSymbol, Symbol},
    utility::parsing::{ident, in_multispace},
};

use param::Param;

use self::template::Template;

/// A parsed instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    /// The name of the instruction.
    pub name: String,
    /// The parameters of the instruction.
    pub params: Vec<Param>,
}

impl Parsed {
    pub fn fill_symbol(&mut self, instruction_offset: u32, symbol: &Symbol) {
        for param in self.params.iter_mut() {
            if let Param::Symbol(s) = param && s == &symbol.name {
                // todo: offset calculation maybe depend on the address
                *param = Param::Immediate(symbol.offset as i32 - instruction_offset as i32);
            } else if let Param::Immediate(x) = param && *x == 0 {
                *param = Param::Immediate(symbol.offset as i32 - instruction_offset as i32);
            }
        }
    }
}

/// An unparsed instruction.
/// "Unparsed" means we regard all parts of this instruction as string.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Unparsed {
    /// The name of the instruction.
    pub name: String,
    /// The parameters of the instruction.
    pub params: Vec<String>,
}

impl From<Unparsed> for Parsed {
    fn from(unparsed: Unparsed) -> Self {
        Parsed {
            name: unparsed.name,
            params: unparsed
                .params
                .into_iter()
                .map(|it| param::parse(&it).unwrap().1)
                .collect(),
        }
    }
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

/// Parse asm code into parsed instruction.
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

/// Parse binary instruction into parsed instruction.
pub fn parse_bin_with_pending<'a>(
    bin: &'a BitSlice<u32>,
    pending_symbol: &'a PendingSymbol,
) -> IResult<&'a BitSlice<u32>, Parsed> {
    if let Some((name, (rest, params))) = templates().iter().find_map(|(name, template)| {
        template
            .parse_binary_with_pending_symbol(bin, pending_symbol)
            .ok()
            .map(|it| (name, it))
    }) {
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

/// Parse binary instruction into parsed instruction.
pub fn parse_bin(bin: &BitSlice<u32>) -> IResult<&BitSlice<u32>, Parsed> {
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
    pub fn binary(&self, address: u64) -> BitVec<u32> {
        let template = templates().get(self.name.as_str()).unwrap();
        template.render(&self.params, address).into_iter().collect()
    }
}

/// Macro for easily constructing an instruction.
macro_rules! instruction {
    ($name:ident) => {
        crate::binary_format::clef::tests::instruction::Parsed {
            name: stringify!($name).to_string(),
            params: vec![],
        }
    };
    ($name:ident, $param1:expr) => {
        crate::binary_format::clef::tests::instruction::Parsed {
            name: stringify!($name).to_string(),
            params: vec![crate::backend::riscv::instruction::param::AsParam::as_param(&$param1)],
        }
    };
    ($name:ident, $param1:ident) => {
        crate::binary_format::clef::tests::instruction::Parsed {
            name: stringify!($name).to_string(),
            params: vec![
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
            ],
        }
    };
    ($name:ident, $param1:ident, $param2:expr) => {
        crate::binary_format::clef::tests::instruction::Parsed {
            name: stringify!($name).to_string(),
            params: vec![
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
                crate::backend::riscv::instruction::param::AsParam::as_param(&$param2),
            ],
        }
    };
    ($name:ident, $param1:ident, $param2:expr, $param3:ident) => {
        crate::binary_format::clef::tests::instruction::Parsed {
            name: stringify!($name).to_string(),
            params: vec![
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
                crate::backend::riscv::instruction::param::AsParam::as_param(&$param2),
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param3)),
            ],
        }
    };
    ($name:ident, $param1:ident, $param2:ident, $param3:expr) => {
        crate::binary_format::clef::tests::instruction::Parsed {
            name: stringify!($name).to_string(),
            params: vec![
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param2)),
                crate::backend::riscv::instruction::param::AsParam::as_param(&$param3),
            ],
        }
    };
    ($name:ident, $param1:ident, $param2:ident, $param3:ident) => {
        crate::binary_format::clef::tests::instruction::Parsed {
            name: stringify!($name).to_string(),
            params: vec![
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param2)),
                crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param3)),
            ],
        }
    };
}

pub(crate) use instruction;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let code = "lui x1, 0x1234";
        let (rest, parsed) = parse(code).unwrap();
        assert_eq!(rest, "");
        assert_eq!(parsed.name, "lui");
        assert_eq!(parsed.params.len(), 2);
        assert_eq!(parsed.params[0], Param::Register(1));
        assert_eq!(parsed.params[1], Param::Immediate(0x1234));
    }

    #[test]
    fn test_parse_bin() {
        let instruction = 0x009980b7u32;
        let instruction_bits = instruction.view_bits::<Lsb0>();
        let parsed = parse_bin(instruction_bits).unwrap().1;
        assert_eq!(
            parsed,
            Parsed {
                name: "lui".to_string(),
                params: vec![Param::Register(1), Param::Immediate(0x998)]
            }
        );
    }
}
