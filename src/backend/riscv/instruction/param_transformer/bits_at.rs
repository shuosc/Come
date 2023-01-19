use super::IsParamTransformer;
use crate::{
    backend::riscv::instruction::Param,
    utility::parsing::{self, in_multispace},
};
use bitvec::prelude::*;
use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{delimited, tuple},
    IResult,
};

/// A transformer that takes a part of a parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitsAt {
    pub start: u8,
    pub end: u8,
}

impl BitsAt {
    pub const fn new(start: u8, end: u8) -> Self {
        Self { start, end }
    }
}

pub fn parse(code: &str) -> IResult<&str, BitsAt> {
    map(
        delimited(
            tag("bits_at("),
            tuple((parsing::integer, in_multispace(tag(",")), parsing::integer)),
            tag(")"),
        ),
        |(start, _, end)| BitsAt::new(start, end),
    )(code)
}

impl IsParamTransformer for BitsAt {
    fn param_to_instruction_part(&self, _address: u64, param: &Param) -> BitVec<u32> {
        let param_bits = param.unwrap_immediate() as u32;
        param_bits.view_bits::<Lsb0>()[self.start as usize..self.end as usize].to_bitvec()
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut Param) {
        if let Param::Immediate(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[self.start as usize..self.end as usize].copy_from_bitslice(instruction_part);
            *param_value = param_bits_store as i32;
        }
    }

    fn default_param(&self) -> Param {
        Param::Immediate(0)
    }

    fn bit_count(&self) -> usize {
        (self.end - self.start) as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(parse("bits_at(0, 12)").unwrap().1, BitsAt::new(0, 12));
        assert!(parse("bit_at(0)").is_err());
    }

    #[test]
    fn param_to_instruction_part() {
        let transformer = BitsAt::new(0, 2);
        let param = Param::Immediate(0b1010);
        assert_eq!(
            transformer.param_to_instruction_part(0, &param),
            bits![u32, Lsb0; 0, 1]
        );
        let transformer = BitsAt::new(1, 3);
        assert_eq!(
            transformer.param_to_instruction_part(0, &param),
            bits![u32, Lsb0; 1, 0]
        );
        let transformer = BitsAt::new(3, 8);
        assert_eq!(
            transformer.param_to_instruction_part(0, &param),
            bits![u32, Lsb0; 1, 0, 0, 0, 0]
        );
    }

    #[test]
    fn update_param() {
        let transformer = BitsAt::new(0, 3);
        let mut param = Param::Immediate(0);
        transformer.update_param(bits![u32, Lsb0; 1, 0, 1], &mut param);
        assert_eq!(param, Param::Immediate(0b101));

        let transformer = BitsAt::new(1, 3);
        let mut param = Param::Immediate(0);
        transformer.update_param(bits![u32, Lsb0; 0, 1], &mut param);
        assert_eq!(param, Param::Immediate(0b100));

        let transformer = BitsAt::new(24, 32);
        let mut param = Param::Immediate(0);
        transformer.update_param(bits![u32, Lsb0; 1, 0, 1, 0, 0, 0, 0, 0], &mut param);
        assert_eq!(
            param,
            Param::Immediate(0b0000_0101_0000_0000_0000_0000_0000_0000)
        );
    }
}
