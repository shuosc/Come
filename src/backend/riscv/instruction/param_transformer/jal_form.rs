use crate::backend::riscv::instruction::{self, ParsedParam};
use bitvec::prelude::*;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::{bits_at, IsParamTransformer};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct JalForm;

impl JalForm {
    pub const fn new() -> Self {
        Self
    }
    pub fn parse(code: &str) -> IResult<&str, Self> {
        map(tag("jal_form"), |_| Self::new())(code)
    }
    pub const fn bit_count(&self) -> usize {
        20
    }
}

impl IsParamTransformer for JalForm {
    fn param_to_instruction_part(&self, _address: u64, param: &ParsedParam) -> BitVec<u32> {
        let param_bits_store = param.unwrap_immediate() as u64;
        let param_bits = param_bits_store.view_bits::<Lsb0>();
        let mut instruction_part = BitVec::new();
        instruction_part.extend_from_bitslice(&param_bits[12..20]);
        instruction_part.push(param_bits[11]);
        instruction_part.extend_from_bitslice(&param_bits[1..11]);
        instruction_part.push(param_bits[20]);
        instruction_part
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut ParsedParam) {
        if let ParsedParam::Immediate(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[12..20].copy_from_bitslice(&instruction_part[0..8]);
            param_bits.set(11, instruction_part[8]);
            param_bits[1..11].copy_from_bitslice(&instruction_part[9..19]);
            param_bits.set(20, instruction_part[19]);
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
        let transformer = JalForm::new();
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(-4));
        assert_eq!(
            bits,
            vec![
                true, true, true, true, true, true, true, true, true, false, true, true, true,
                true, true, true, true, true, true, true
            ]
        );
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(4));
        assert_eq!(
            bits,
            vec![
                false, false, false, false, false, false, false, false, false, false, true, false,
                false, false, false, false, false, false, false, false
            ]
        );
        let bits = transformer.param_to_instruction_part(0, &ParsedParam::Immediate(0x998));
        assert_eq!(
            bits,
            vec![
                false, false, false, false, false, false, false, false, true, false, false, true,
                true, false, false, true, true, false, false, false
            ]
        );
    }

    #[test]
    fn test_update_argument() {
        let transformer = JalForm::new();
        let mut param = ParsedParam::Immediate(0);
        transformer.update_param(
            &[
                false, false, false, false, false, false, false, false, true, false, false, true,
                true, false, false, true, true, false, false, false,
            ],
            &mut param,
        );
        assert_eq!(param, ParsedParam::Immediate(0b0000_0000_1001_1001_1000));
    }
}
