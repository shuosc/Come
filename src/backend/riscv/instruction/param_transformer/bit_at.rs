use crate::{backend::riscv::instruction::ParsedParam, utility::parsing};
use nom::{bytes::complete::tag, combinator::map, sequence::delimited, IResult};

use super::{bit_at, IsParamTransformer};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct BitAt(u8);

impl BitAt {
    pub const fn new(index: u8) -> Self {
        Self(index)
    }

    pub fn parse(code: &str) -> IResult<&str, Self> {
        map(
            delimited(tag("bit_at("), parsing::integer, tag(")")),
            Self::new,
        )(code)
    }

    pub const fn bit_count(&self) -> usize {
        1
    }
}

impl IsParamTransformer for BitAt {
    fn argument_to_bits(&self, _address: u64, argument: &ParsedParam) -> Vec<bool> {
        // it is ok to use `as u32` here, see
        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions
        vec![bit_at(argument.unwrap_immediate() as u32, self.0)]
    }

    fn update_argument(&self, instruction_part: &[bool], param: &mut ParsedParam) {
        if let ParsedParam::Immediate(value) = param {
            let bit = instruction_part[0];
            *value |= (bit as i32) << self.0;
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
        let result = BitAt::parse("bit_at(0)").unwrap().1;
        assert_eq!(result, BitAt(0));
        let result = BitAt::parse("bit_at(1)").unwrap().1;
        assert_eq!(result, BitAt(1));
        assert!(BitAt::parse("bits_at(0, 7)").is_err());
    }

    #[test]
    fn argument_to_bits() {
        let transformer = BitAt(0);
        let param = ParsedParam::Immediate(0b1010);
        assert_eq!(transformer.argument_to_bits(0, &param), vec![false]);
        let transformer = BitAt(1);
        assert_eq!(transformer.argument_to_bits(0, &param), vec![true]);
        let transformer = BitAt(7);
        assert_eq!(transformer.argument_to_bits(0, &param), vec![false]);
    }

    #[test]
    fn update_argument() {
        let transformer = BitAt(0);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(&[true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(1));

        let transformer = BitAt(1);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(&[false], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0));

        let transformer = BitAt(30);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(&[true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0x40000000));

        let transformer = BitAt(31);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(&[true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(-0x8000_0000));
    }
}
