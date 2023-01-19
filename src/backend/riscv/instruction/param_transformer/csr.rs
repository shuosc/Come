use crate::backend::riscv::instruction::ParsedParam;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::{bits_at, IsParamTransformer};
use bitvec::prelude::*;
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Csr;

impl Csr {
    pub const fn new() -> Self {
        Self
    }
    pub fn parse(code: &str) -> IResult<&str, Self> {
        map(tag("csr"), |_| Self::new())(code)
    }
    pub const fn bit_count(&self) -> usize {
        12
    }
}

impl IsParamTransformer for Csr {
    fn param_to_instruction_part(&self, _address: u64, param: &ParsedParam) -> BitVec<u32> {
        let param_bits_store = param.unwrap_csr() as u32;
        let param_bits = &param_bits_store.view_bits::<Lsb0>();
        param_bits[0..12].to_bitvec()
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut ParsedParam) {
        if let ParsedParam::Csr(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[0..12].copy_from_bitslice(instruction_part);
            *param_value = param_bits_store as u16;
        }
    }

    fn default_param(&self) -> ParsedParam {
        ParsedParam::Csr(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::riscv::instruction::ParsedParam;

    #[test]
    fn test_argument_to_bits() {
        let transformer = Csr::new();
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Csr(0x7c0));
        assert_eq!(
            bits,
            vec![false, false, false, false, false, false, true, true, true, true, true, false]
        );
    }

    #[test]
    fn test_update_argument() {
        let transformer = Csr::new();
        let mut param = ParsedParam::Csr(0);
        transformer.update_param(
            &[
                false, false, false, false, false, false, true, true, true, true, true, false,
            ],
            &mut param,
        );
        assert_eq!(param, ParsedParam::Csr(0x7c0));
    }
}
