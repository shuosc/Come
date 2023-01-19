use crate::utility::parsing::{self, in_multispace};
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
    pub parts: Vec<Part>,
}

impl Template {
    fn param_bit_count(&self, param_id: usize) -> usize {
        let mut current_result = 0;
        for part in &self.parts {
            match part {
                Part::ParamTransformer((id, ParamTransformer::BitsAt(bits_at)))
                    if param_id == *id =>
                {
                    if bits_at.end as usize > current_result {
                        current_result = bits_at.end as usize;
                    }
                }
                _ => (),
            }
        }
        current_result
    }
    /// Render an instruction into bits with param and address.
    pub fn render(&self, params: &[Param], address: u64) -> BitVec<u32> {
        let mut bits = BitVec::new();
        for part in &self.parts {
            match part {
                Part::BitPattern(bit_pattern) => bits.extend_from_bitslice(bit_pattern),
                Part::ParamTransformer((param_id, transformer)) => bits.extend_from_bitslice(
                    &transformer.param_to_instruction_part(address, &params[*param_id]),
                ),
            }
        }
        bits
    }
    /// Parse an instruction to get its params.
    pub fn parse_binary<'a>(
        &'a self,
        bits: &'a BitSlice<u32>,
    ) -> IResult<&'a BitSlice<u32>, Vec<Param>> {
        let mut bits = bits;
        let mut params = Vec::new();
        for part in &self.parts {
            match part {
                Part::BitPattern(bit_pattern) => {
                    if bits.len() < bit_pattern.len() {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            bits,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                    if bits[0..bit_pattern.len()] != bit_pattern[..] {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            bits,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                    bits = &bits[bit_pattern.len()..];
                }
                Part::ParamTransformer((param_id, transformer)) => {
                    while params.len() <= *param_id {
                        params.push(None);
                    }
                    let param = params
                        .get_mut(*param_id)
                        .unwrap()
                        .get_or_insert(transformer.default_param());
                    transformer.update_param(&bits[0..transformer.bit_count()], param);
                    bits = &bits[transformer.bit_count()..];
                }
            }
        }
        let mut params = params.into_iter().map(|it| it.unwrap()).collect_vec();
        for (param_id, param) in params.iter_mut().enumerate() {
            if let Param::Immediate(imm) = param {
                let imm_bits = *imm as u32;
                let bits = imm_bits.view_bits::<Lsb0>();
                *imm = bits[0..self.param_bit_count(param_id)].load_le();
            }
        }
        Ok((bits, params))
    }
}

/// Parse the string form of a template to get a [`Template`] object.
pub fn parse(code: &str) -> IResult<&str, Template> {
    // for human beings, we prefer to write the MSB first
    // so we need to reverse the parts
    map(many1(parse_template_part), |mut parts| {
        parts.reverse();
        Template { parts }
    })(code)
}

#[cfg(test)]
mod tests {

    use crate::backend::riscv::instruction::param_transformer::{BitsAt, Register};

    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(
            parse("{{ params[0] | bits_at(0,5) }}100"),
            Ok((
                "",
                Template {
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
            parts: vec![
                Part::BitPattern(bitvec![u32, Lsb0; 0, 0, 1]),
                Part::ParamTransformer((0, BitsAt::new(0, 5).into())),
            ],
        };
        assert_eq!(
            template.render(&[Param::Immediate(0b11101)], 0),
            bits![0, 0, 1, 1, 0, 1, 1, 1]
        );

        let template = Template {
            parts: vec![
                Part::BitPattern(bitvec![u32, Lsb0; 1, 0, 1, 1, 0]),
                Part::ParamTransformer((1, BitsAt::new(5, 8).into())),
                Part::BitPattern(bitvec![u32, Lsb0; 0, 0, 1]),
                Part::ParamTransformer((0, Register.into())),
            ],
        };
        assert_eq!(
            template.render(
                &[Param::Register(0b11101), Param::Immediate(0b0010_0000)],
                0
            ),
            bits![1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1]
        );
    }
}
