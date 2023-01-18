use crate::{
    backend::riscv::ParsedParam,
    utility::parsing::{self, in_multispace},
};
use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{delimited, tuple},
    IResult,
};

use super::{bits_at, IsParamTransformer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitsAt {
    pub start: u8,
    pub end: u8,
}

impl BitsAt {
    pub const fn new(start: u8, end: u8) -> Self {
        Self { start, end }
    }
    pub fn parse(code: &str) -> IResult<&str, Self> {
        map(
            delimited(
                tag("bits_at("),
                tuple((parsing::integer, in_multispace(tag(",")), parsing::integer)),
                tag(")"),
            ),
            |(start, _, end)| Self::new(start, end),
        )(code)
    }
    pub const fn bit_count(&self) -> usize {
        (self.end - self.start) as _
    }
}

impl IsParamTransformer for BitsAt {
    fn argument_to_bits(&self, _address: u64, argument: &ParsedParam) -> Vec<bool> {
        bits_at(argument.unwrap_immediate() as u32, self.start..self.end)
    }

    fn update_argument(&self, instruction_part: &[bool], param: &mut ParsedParam) {
        if let ParsedParam::Immediate(value) = param {
            for (index, &bit) in (self.start..self.end).zip(instruction_part.iter()) {
                *value |= (bit as i32) << index;
            }
        }
    }

    fn default_argument(&self) -> ParsedParam {
        ParsedParam::Immediate(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            BitsAt::parse("bits_at(0, 12)").unwrap().1,
            BitsAt::new(0, 12)
        );
        assert!(BitsAt::parse("bit_at(0)").is_err());
    }

    #[test]
    fn argument_to_bits() {
        let transformer = BitsAt::new(0, 2);
        let param = ParsedParam::Immediate(0b1010);
        assert_eq!(transformer.argument_to_bits(0, &param), vec![false, true]);
        let transformer = BitsAt::new(1, 3);
        assert_eq!(transformer.argument_to_bits(0, &param), vec![true, false]);
        let transformer = BitsAt::new(3, 8);
        assert_eq!(
            transformer.argument_to_bits(0, &param),
            vec![true, false, false, false, false]
        );
    }

    #[test]
    fn update_argument() {
        let transformer = BitsAt::new(0, 3);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(&[true, false, true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0b101));

        let transformer = BitsAt::new(1, 3);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(&[false, true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0b100));

        let transformer = BitsAt::new(24, 32);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(
            &[true, false, true, false, false, false, false, false],
            &mut param,
        );
        assert_eq!(
            param,
            ParsedParam::Immediate(0b0000_0101_0000_0000_0000_0000_0000_0000)
        );
    }
}
