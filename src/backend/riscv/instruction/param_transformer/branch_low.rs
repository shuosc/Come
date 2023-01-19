use crate::backend::riscv::instruction::ParsedParam;
use bitvec::{vec::BitVec, view::BitView};
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::{bits_at, IsParamTransformer};
use bitvec::prelude::*;
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct BranchLow;

impl BranchLow {
    pub const fn new() -> Self {
        Self
    }
    pub fn parse(code: &str) -> IResult<&str, Self> {
        map(tag("branch_low"), |_| Self::new())(code)
    }
    pub const fn bit_count(&self) -> usize {
        5
    }
}

impl IsParamTransformer for BranchLow {
    fn param_to_instruction_part(&self, address: u64, param: &ParsedParam) -> BitVec<u32> {
        let mut param_bits_store = (param.unwrap_immediate() as i64 - address as i64) as u32;
        let mut param_bits = param_bits_store.view_bits::<Lsb0>();
        let mut instruction_part = BitVec::new();
        instruction_part.push(param_bits[11]);
        instruction_part.extend_from_bitslice(&param_bits[1..5]);
        instruction_part
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut ParsedParam) {
        if let ParsedParam::Immediate(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits.set(11, instruction_part[0]);
            param_bits[1..5].copy_from_bitslice(&instruction_part[1..5]);
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
    use crate::backend::riscv::instruction::ParsedParam;

    #[test]
    fn test_argument_to_bits() {
        let transformer = BranchLow::new();
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(-4));
        assert_eq!(bits, vec![true, false, true, true, true]);
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(4));
        assert_eq!(bits, vec![false, false, true, false, false]);
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(0x998));
        assert_eq!(bits, vec![true, false, false, true, true]);
    }

    #[test]
    fn test_update_argument() {
        let transformer = BranchLow::new();
        let mut param = ParsedParam::Immediate(0);
        transformer.update_param(&[true, false, true, true, true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0b1000_0001_1100));
    }
}
