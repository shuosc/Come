use std::{collections::HashMap, sync::OnceLock};

use crate::{
    backend::riscv::simple_instruction::template,
    binary_format::clef::{PendingSymbol, Symbol},
    utility::parsing::{self, in_multispace},
};
use bitvec::prelude::*;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::many1,
    sequence::{delimited, tuple},
    IResult,
};

use super::{
    param::Decided,
    param_transformer::{self, IsParamTransformer, ParamTransformer},
    Param,
};

/// A part of a template.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Part {
    /// Several determined bits.
    BitPattern(BitVec<u32>),
    /// A parameter transformer for transform `Self.0`th param into expected form.
    ParamTransformer((usize, ParamTransformer)),
}

fn parse_param_transformer(code: &str) -> IResult<&str, (usize, ParamTransformer)> {
    map(
        delimited(
            tag("{{"),
            in_multispace(tuple((
                delimited(tag("params["), parsing::integer, tag("]")),
                in_multispace(tag("|")),
                param_transformer::parse,
            ))),
            tag("}}"),
        ),
        |(param_id, _, transformer)| (param_id, transformer),
    )(code)
}

fn parse_template_part(code: &str) -> IResult<&str, Part> {
    // for human beings, we prefer to write the MSB first
    // so we need to reverse the bits
    alt((
        map(many1(alt((tag("0"), tag("1")))), |char_bits| {
            Part::BitPattern(
                char_bits
                    .into_iter()
                    .rev()
                    .map(|char_bit| match char_bit {
                        "0" => false,
                        "1" => true,
                        _ => unreachable!(),
                    })
                    .collect(),
            )
        }),
        map(parse_param_transformer, Part::ParamTransformer),
    ))(code)
}

/// A template of an instruction.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Template {
    pub name: &'static str,
    pub parts: Vec<Part>,
}

impl Template {
    fn param_bit_count(&self, param_id: usize) -> usize {
        let mut current_result = 0;
        for part in &self.parts {
            match part {
                // todo: handle these fucking things better
                Part::ParamTransformer((id, ParamTransformer::BitsAt(bits_at)))
                    if param_id == *id =>
                {
                    if bits_at.end as usize > current_result {
                        current_result = bits_at.end as usize;
                    }
                }
                Part::ParamTransformer((id, ParamTransformer::JalForm(_))) if param_id == *id => {
                    if 20usize > current_result {
                        current_result = 20;
                    }
                }
                _ => (),
            }
        }
        current_result
    }
    pub fn bit_count(&self) -> usize {
        self.parts
            .iter()
            .map(|part| match part {
                Part::BitPattern(bit_pattern) => bit_pattern.len(),
                Part::ParamTransformer((param_id, _)) => self.param_bit_count(*param_id),
            })
            .sum()
    }
    /// Render an instruction into bits with param and offset.
    pub fn render(&self, params: &[Param], offset: u64) -> BitVec<u32> {
        let mut bits = BitVec::new();
        for part in &self.parts {
            match part {
                Part::BitPattern(bit_pattern) => bits.extend_from_bitslice(bit_pattern),
                Part::ParamTransformer((param_id, transformer)) => bits.extend_from_bitslice(
                    &transformer.param_to_instruction_part(offset, &params[*param_id]),
                ),
            }
        }
        bits
    }
    /// Parse the binary form of an instruction
    /// If matched success, return its params.
    pub fn parse_binary<'a>(
        &'a self,
        (mut bits, offset_bits): (&'a BitSlice<u32>, usize),
        pending_symbols: &[PendingSymbol],
    ) -> IResult<(&'a BitSlice<u32>, usize), Vec<Param>> {
        let mut params = Vec::new();
        for part in &self.parts {
            match part {
                Part::BitPattern(bit_pattern) => {
                    if bits.len() < bit_pattern.len() {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            (bits, offset_bits),
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                    if bits[0..bit_pattern.len()] != bit_pattern[..] {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            (bits, offset_bits),
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                    bits = &bits[bit_pattern.len()..];
                }
                Part::ParamTransformer((param_id, transformer)) => {
                    let param_id = *param_id;
                    while params.len() <= param_id {
                        params.push(None);
                    }
                    // the param is a symbol
                    let offset_bits = offset_bits as u32;
                    let offset_bytes = offset_bits / 8;
                    let symbol_param = pending_symbols
                        .iter()
                        .find(|it| it.used_by_instruction_at_offset(offset_bytes));
                    if let Some(pending_symbol) = symbol_param {
                        let param = Param::Unresolved(pending_symbol.name.clone());
                        params[param_id] = Some(param);
                    } else {
                        let param = params
                            .get_mut(param_id)
                            .unwrap()
                            .get_or_insert(transformer.default_param());
                        transformer.update_param(&bits[0..transformer.bit_count()], param);
                        bits = &bits[transformer.bit_count()..];
                    }
                }
            }
        }
        let mut params = params.into_iter().map(|it| it.unwrap()).collect_vec();
        // extend the immediate values to 32 bits
        for (param_id, param) in params.iter_mut().enumerate() {
            if let Param::Decided(Decided::Immediate(imm))
            | Param::Resolved(_, Decided::Immediate(imm)) = param
            {
                let imm_bits = *imm as u32;
                let bits = imm_bits.view_bits::<Lsb0>();
                *imm = bits[0..self.param_bit_count(param_id)].load_le();
            }
        }
        Ok(((bits, offset_bits + self.bit_count()), params))
    }
}

/// Parse the string form of a template to get a [`Template`] object.
fn parse<'a>(code: &'a str, name: &'static str) -> IResult<&'a str, Template> {
    // for human beings, we prefer to write the MSB first
    // so we need to reverse the parts
    map(many1(parse_template_part), |mut parts| {
        parts.reverse();
        Template { name, parts }
    })(code)
}

pub fn templates() -> &'static HashMap<&'static str, Template> {
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
            mapping.insert(name, template::parse(template.trim(), name).unwrap().1);
        }
        mapping
    })
}

#[cfg(test)]
mod tests {

    use crate::backend::riscv::simple_instruction::param_transformer::{BitsAt, Register};

    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("{{ params[0] | bits_at(0,5) }}100", "test"),
            Ok((
                "",
                Template {
                    name: "test",
                    parts: vec![
                        Part::BitPattern(bitvec![u32, Lsb0; 0, 0, 1]),
                        Part::ParamTransformer((0, BitsAt::new(0, 5).into())),
                    ]
                }
            ))
        );
    }

    #[test]
    fn test_render() {
        let template = Template {
            name: "test",
            parts: vec![
                Part::BitPattern(bitvec![u32, Lsb0; 0, 0, 1]),
                Part::ParamTransformer((0, BitsAt::new(0, 5).into())),
            ],
        };
        assert_eq!(
            template.render(&[Param::Decided(Decided::Immediate(0b11101))], 0),
            bits![0, 0, 1, 1, 0, 1, 1, 1]
        );

        let template = Template {
            parts: vec![
                Part::BitPattern(bitvec![u32, Lsb0; 1, 0, 1, 1, 0]),
                Part::ParamTransformer((1, BitsAt::new(5, 8).into())),
                Part::BitPattern(bitvec![u32, Lsb0; 0, 0, 1]),
                Part::ParamTransformer((0, Register.into())),
            ],
            name: "test",
        };
        assert_eq!(
            template.render(
                &[
                    Param::Decided(Decided::Register(0b11101)),
                    Param::Decided(Decided::Immediate(0b0010_0000))
                ],
                0
            ),
            bits![1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1]
        );
    }
}
