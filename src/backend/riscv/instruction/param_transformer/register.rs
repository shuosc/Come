use crate::backend::riscv::instruction::param::ParsedParam;
use bitvec::prelude::*;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::{bits_at, IsParamTransformer};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Register;

impl Register {
    pub const fn new() -> Self {
        Self
    }
    pub fn parse(code: &str) -> IResult<&str, Self> {
        map(tag("register"), |_| Self::new())(code)
    }
    pub const fn bit_count(&self) -> usize {
        5
    }
}

impl IsParamTransformer for Register {
    fn param_to_instruction_part(&self, _address: u64, param: &ParsedParam) -> BitVec<u32> {
        let param_bits_store = param.unwrap_register() as u32;
        let param_bits = param_bits_store.view_bits::<Lsb0>();
        param_bits[0..5].to_bitvec()
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut ParsedParam) {
        if let ParsedParam::Register(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[0..5].copy_from_bitslice(instruction_part);
            *param_value = param_bits_store as u8;
        }
    }

    fn default_param(&self) -> ParsedParam {
        ParsedParam::Register(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_to_bits() {
        let transformer = Register::new();
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Register(0x1f));
        assert_eq!(bits, vec![true, true, true, true, true]);
    }

    #[test]
    fn test_update_argument() {
        let transformer = Register::new();
        let mut param = ParsedParam::Register(0);
        transformer.update_param(&[true, true, true, true, true], &mut param);
        assert_eq!(param, ParsedParam::Register(0x1f));
    }
}
