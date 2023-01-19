use super::{bits_at, IsParamTransformer};
use crate::{
    backend::riscv::instruction::{param, ParsedParam},
    utility::parsing::{self, in_multispace},
};
use bitvec::{prelude::*, vec::BitVec, view::BitView};
use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{delimited, tuple},
    IResult,
};

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
    fn param_to_instruction_part(&self, _address: u64, param: &ParsedParam) -> BitVec<u32> {
        let param_bits = param.unwrap_immediate() as u32;
        param_bits.view_bits::<Lsb0>()[self.start as usize..self.end as usize].to_bitvec()
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut ParsedParam) {
        if let ParsedParam::Immediate(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[self.start as usize..self.end as usize].copy_from_bitslice(instruction_part);
            *param_value = param_bits_store as i32;
        }
    }

    fn default_param(&self) -> ParsedParam {
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
    fn param_to_instruction_part() {
        let transformer = BitsAt::new(0, 2);
        let param = ParsedParam::Immediate(0b1010);
        assert_eq!(
            transformer.param_to_instruction_part(0, &param),
            vec![false, true]
        );
        let transformer = BitsAt::new(1, 3);
        assert_eq!(
            transformer.param_to_instruction_part(0, &param),
            vec![true, false]
        );
        let transformer = BitsAt::new(3, 8);
        assert_eq!(
            transformer.param_to_instruction_part(0, &param),
            vec![true, false, false, false, false]
        );
    }

    #[test]
    fn update_param() {
        let transformer = BitsAt::new(0, 3);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_param(&[true, false, true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0b101));

        let transformer = BitsAt::new(1, 3);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_param(&[false, true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0b100));

        let transformer = BitsAt::new(24, 32);
        let mut param = ParsedParam::Immediate(0);
        transformer.update_param(
            &[true, false, true, false, false, false, false, false],
            &mut param,
        );
        assert_eq!(
            param,
            ParsedParam::Immediate(0b0000_0101_0000_0000_0000_0000_0000_0000)
        );
    }
}
