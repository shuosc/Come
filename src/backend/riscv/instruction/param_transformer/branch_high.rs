use crate::backend::riscv::instruction::ParsedParam;
use bitvec::{vec::BitVec, view::BitView};

use nom::{bytes::complete::tag, combinator::map, IResult};

use super::IsParamTransformer;
use bitvec::prelude::*;
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct BranchHigh;

impl BranchHigh {
    pub const fn new() -> Self {
        Self
    }
}

pub fn parse(code: &str) -> IResult<&str, BranchHigh> {
    map(tag("branch_high"), |_| BranchHigh::new())(code)
}

impl IsParamTransformer for BranchHigh {
    fn param_to_instruction_part(&self, address: u64, param: &ParsedParam) -> BitVec<u32> {
        let param_bits_store = (param.unwrap_immediate() as i64 - address as i64) as u32;
        // todo: check whether offset cannot be hold in 12bits width
        let param_bits = param_bits_store.view_bits::<Lsb0>();
        let mut instruction_part = BitVec::new();
        instruction_part.extend_from_bitslice(&param_bits[5..11]);
        instruction_part.push(param_bits[12]);
        instruction_part
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut ParsedParam) {
        if let ParsedParam::Immediate(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[5..11].copy_from_bitslice(&instruction_part[0..6]);
            param_bits.set(12, instruction_part[6]);
            *param_value = param_bits_store as i32;
        }
    }

    fn default_param(&self) -> ParsedParam {
        ParsedParam::Immediate(0)
    }

    fn bit_count(&self) -> usize {
        7
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::riscv::instruction::ParsedParam;

    #[test]
    fn test_argument_to_bits() {
        let transformer = BranchHigh::new();
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(-4));
        assert_eq!(bits, bits![1, 1, 1, 1, 1, 1, 1]);
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(4));
        assert_eq!(bits, bits![0, 0, 0, 0, 0, 0, 0]);
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(0x998));
        assert_eq!(bits, bits![0, 0, 1, 1, 0, 0, 0]);
    }

    #[test]
    fn test_update_argument() {
        let transformer = BranchHigh::new();
        let mut param = ParsedParam::Immediate(0);
        let instruction_part = bits![u32, Lsb0; 0, 0, 1, 1, 0, 0, 0];
        transformer.update_param(instruction_part, &mut param);
        assert_eq!(param, ParsedParam::Immediate(0b0001_1000_0000));
    }
}
