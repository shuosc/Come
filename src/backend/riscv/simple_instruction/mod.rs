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
use itertools::Itertools;
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

use super::UnparsedInstruction;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleInstruction {
    template: &'static Template,
    params: Vec<Param>,
    offset_bytes: Option<u32>,
}

impl Display for SimpleInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.template.name)?;
        // todo: find a way to handle `offset(register)` format
        for (i, param) in self.params.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{param}")?;
        }
        Ok(())
    }
}

impl SimpleInstruction {
    pub fn set_offset_bytes(&mut self, offset: u32) {
        self.offset_bytes = Some(offset);
    }
    pub fn offset_bytes(&self) -> u32 {
        self.offset_bytes.unwrap()
    }
    pub fn pending_symbol(&self) -> Option<PendingSymbol> {
        self.params
            .iter()
            .find(|param| matches!(param, Param::Unresolved(_)))
            .map(|param| PendingSymbol {
                name: param.unwrap_symbol().to_string(),
                pending_instructions_offset_bytes: self.offset_bytes.iter().cloned().collect(),
            })
    }
    pub fn decide_symbol(&mut self, symbol: &Symbol) {
        assert!(self.offset_bytes.is_some());
        for param in self.params.iter_mut() {
            if matches!(param, Param::Unresolved(_)) && param.unwrap_symbol() == symbol.name {
                param.fill_symbol(symbol, self.offset_bytes.unwrap());
            }
        }
    }
    pub fn bit_count(&self) -> usize {
        self.template.bit_count()
    }
    pub fn render(&self) -> BitVec<u32> {
        self.template
            .render(&self.params, self.offset_bytes.unwrap() as _)
    }
}

impl TryFrom<UnparsedInstruction> for SimpleInstruction {
    type Error = ();

    fn try_from(value: UnparsedInstruction) -> Result<Self, Self::Error> {
        let template = template::templates().get(value.name.as_str()).ok_or(())?;
        Ok(Self {
            template,
            params: value
                .params
                .into_iter()
                .map(|it| param::parse(&it).unwrap().1)
                .collect(),
            offset_bytes: None,
        })
    }
}

/// Parse asm code into instruction.
pub fn parse(code: &str) -> IResult<&str, SimpleInstruction> {
    map(
        tuple((
            ident,
            multispace1,
            separated_list0(in_multispace(alt((tag(","), tag("(")))), param::parse),
            opt(tag(")")),
        )),
        |(name, _, params, _)| SimpleInstruction {
            template: template::templates().get(name.as_str()).unwrap(),
            params,
            offset_bytes: None,
        },
    )(code)
}

/// Parse binary form of instruction.
pub fn parse_binary<'a>(
    bits_and_offset: (&'a BitSlice<u32>, usize),
    // todo: symbols: &'a [Symbol],
    pending_symbols: &'a [PendingSymbol],
) -> IResult<(&'a BitSlice<u32>, usize), SimpleInstruction> {
    let offset_bytes = (bits_and_offset.1 / 8) as u32;
    // todo: speed up matching process
    for (_name, template) in template::templates() {
        if let Ok((rest, params)) = template.parse_binary(bits_and_offset, pending_symbols) {
            return Ok((
                rest,
                SimpleInstruction {
                    template,
                    params,
                    offset_bytes: Some(offset_bytes),
                },
            ));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(
        bits_and_offset,
        nom::error::ErrorKind::Tag,
    )))
}

pub fn parse_whole_binary(
    bits: &BitSlice<u32>,
    pending_symbols: &[PendingSymbol],
) -> Vec<SimpleInstruction> {
    let mut result = Vec::new();
    let mut rest = (bits, 0);
    while !rest.0.is_empty() {
        if let Ok((new_rest, instruction)) = parse_binary(rest, pending_symbols) {
            result.push(instruction);
            rest = new_rest;
        } else {
            panic!("Failed to parse binary form of instruction");
        }
    }
    result
}

// /// An unparsed instruction.
// /// "Unparsed" means we regard all parts of this instruction as string.
// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct Unparsed {
//     /// The name of the instruction.
//     pub name: String,
//     /// The parameters of the instruction.
//     pub params: Vec<String>,
// }

// /// A parsed instruction.
// #[derive(Debug, Clone, PartialEq)]
// pub struct Parsed {
//     /// The name of the instruction.
//     pub name: String,
//     /// The parameters of the instruction.
//     pub params: Vec<Param>,
// }

// impl Parsed {
//     pub fn fill_symbol(&mut self, instruction_offset_bytes: u32, symbol: &Symbol) {
//         for param in self.params.iter_mut() {
//             if let Param::Symbol(s) = param && s == &symbol.name {
//                 // fixme: use direct offset calculation here maybe incorrect
//                 *param = Param::Decided(Decided::Immediate(symbol.offset_bytes as i32 - instruction_offset_bytes as i32);
//             }
//         }
//     }

//     pub fn binary(&self, offset: u64, symbol_offsets: &HashMap<String, u32>) -> BitVec<u32> {
//         let template = templates().get(self.name.as_str()).unwrap();
//         dbg!(&self);
//         let params = self
//             .params
//             .iter()
//             .map(|it| {
//                 if let Param::Symbol(symbol_name) = it {
//                     let symbol_offset = symbol_offsets[symbol_name];
//                     Param::Decided(Decided::Immediate(symbol_offset as _)
//                 } else {
//                     it.clone()
//                 }
//             })
//             .collect_vec();
//         template.render(&params, offset).into_iter().collect()
//     }
// }

// impl From<Unparsed> for Parsed {
//     fn from(unparsed: Unparsed) -> Self {
//         Parsed {
//             name: unparsed.name,
//             params: unparsed
//                 .params
//                 .into_iter()
//                 .map(|it| param::parse(&it).unwrap().1)
//                 .collect(),
//         }
//     }
// }

// impl Display for Parsed {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{} ", self.name)?;
//         // todo: find a better way to handle `offset(register)` format
//         const MEMORY_INSTRUCTIONS: [&str; 8] = ["lb", "lh", "lw", "lbu", "lhu", "sb", "sh", "sw"];
//         if MEMORY_INSTRUCTIONS.contains(&self.name.as_str()) {
//             write!(
//                 f,
//                 "{}, {}({})",
//                 self.params[0], self.params[1], self.params[2]
//             )?;
//         } else {
//             for (i, param) in self.params.iter().enumerate() {
//                 if i != 0 {
//                     write!(f, ", ")?;
//                 }
//                 write!(f, "{param}")?;
//             }
//         }
//         Ok(())
//     }
// }

// fn templates() -> &'static HashMap<&'static str, Template> {
//     static TEMPLATE_MAPPING: OnceLock<HashMap<&'static str, Template>> = OnceLock::new();
//     TEMPLATE_MAPPING.get_or_init(|| {
//         let mut mapping = HashMap::new();
//         let templates_str = include_str!("../spec/instructions.spec");
//         let templates = templates_str
//             .split('\n')
//             .map(|it| it.trim())
//             .filter(|it| !it.is_empty());
//         for template in templates {
//             let (name, template) = template.split_once(' ').unwrap();
//             mapping.insert(name, template::parse(template.trim()).unwrap().1);
//         }
//         mapping
//     })
// }

// pub fn parse_whole_binary(
//     bin: &BitSlice<u32>,
//     offset_pending_symbol_map: &HashMap<u32, &PendingSymbol>,
// ) -> Vec<Parsed> {
//     let mut content = (bin, 0usize);
//     let mut result = Vec::new();
//     let mut offset = 0;
//     while !content.0.is_empty() {
//         let (rest, result_instruction) = parse_bin(content, &offset_pending_symbol_map).unwrap();
//         content = rest;
//         result.push(result_instruction);
//     }
//     result
// }

// /// Parse binary instruction into parsed instruction.
// pub fn parse_bin<'a>(
//     bits_and_offset: (&'a BitSlice<u32>, usize),
//     pending_symbols: &'a HashMap<u32, &'a PendingSymbol>,
// ) -> IResult<(&'a BitSlice<u32>, usize), Parsed> {
//     // todo: speed up matching process
//     if let Some((name, (rest, params))) = templates().iter().find_map(|(name, template)| {
//         template
//             .parse_binary(bits_and_offset, pending_symbols)
//             .ok()
//             .map(|it| (name, it))
//     }) {
//         Ok((
//             rest,
//             Parsed {
//                 name: name.to_string(),
//                 params,
//             },
//         ))
//     } else {
//         unreachable!()
//     }
// }

// /// Macro for easily constructing an instruction.
// /// Currently used only in tests, but hopefully will be used in the asm generator the future.
// #[cfg(test)]
// macro_rules! instruction {
//     ($name:ident) => {
//         crate::binary_format::clef::tests::instruction::Parsed {
//             name: stringify!($name).to_string(),
//             params: vec![],
//         }
//     };
//     ($name:ident, $param1:expr) => {
//         crate::binary_format::clef::tests::instruction::Parsed {
//             name: stringify!($name).to_string(),
//             params: vec![crate::backend::riscv::instruction::param::AsParam::as_param(&$param1)],
//         }
//     };
//     ($name:ident, $param1:ident) => {
//         crate::binary_format::clef::tests::instruction::Parsed {
//             name: stringify!($name).to_string(),
//             params: vec![
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
//             ],
//         }
//     };
//     ($name:ident, $param1:ident, $param2:expr) => {
//         crate::binary_format::clef::tests::instruction::Parsed {
//             name: stringify!($name).to_string(),
//             params: vec![
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&$param2),
//             ],
//         }
//     };
//     ($name:ident, $param1:ident, $param2:expr, $param3:ident) => {
//         crate::binary_format::clef::tests::instruction::Parsed {
//             name: stringify!($name).to_string(),
//             params: vec![
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&$param2),
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param3)),
//             ],
//         }
//     };
//     ($name:ident, $param1:ident, $param2:ident, $param3:expr) => {
//         crate::binary_format::clef::tests::instruction::Parsed {
//             name: stringify!($name).to_string(),
//             params: vec![
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param2)),
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&$param3),
//             ],
//         }
//     };
//     ($name:ident, $param1:ident, $param2:ident, $param3:ident) => {
//         crate::binary_format::clef::tests::instruction::Parsed {
//             name: stringify!($name).to_string(),
//             params: vec![
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param1)),
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param2)),
//                 crate::backend::riscv::instruction::param::AsParam::as_param(&stringify!($param3)),
//             ],
//         }
//     };
// }

// #[cfg(test)]
// pub(crate) use instruction;

// #[cfg(test)]
// mod tests {
//     use std::future::pending;

//     use super::*;

//     #[test]
//     fn test_parse() {
//         let code = "lui x1, 0x1234";
//         let (rest, parsed) = parse(code).unwrap();
//         assert_eq!(rest, "");
//         assert_eq!(parsed.name, "lui");
//         assert_eq!(parsed.params.len(), 2);
//         assert_eq!(parsed.params[0], Param::Decided(Decided::Register(1));
//         assert_eq!(parsed.params[1], Param::Decided(Decided::Immediate(0x1234));
//     }

//     #[test]
//     fn test_parse_bin() {
//         let instruction = 0x009980b7u32;
//         let instruction_bits = instruction.view_bits::<Lsb0>();
//         let pending_symbols = HashMap::new();
//         let parsed = parse_bin((instruction_bits, 0), &pending_symbols)
//             .unwrap()
//             .1;
//         assert_eq!(
//             parsed,
//             Parsed {
//                 name: "lui".to_string(),
//                 params: vec![Param::Decided(Decided::Register(1), Param::Decided(Decided::Immediate(0x998)]
//             }
//         );
//     }
// }
