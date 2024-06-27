use crate::backend::riscv::simple_instruction::{param::Decided, Param};
use bitvec::prelude::*;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::IsParamTransformer;

/// A transformer that extract the bits of an imm used for a jal instruction.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct JalForm;

impl Default for JalForm {
    fn default() -> Self {
        Self::new()
    }
}

impl JalForm {
    pub const fn new() -> Self {
        Self
    }
}
pub fn parse(code: &str) -> IResult<&str, JalForm> {
    map(tag("jal_form"), |_| JalForm::new())(code)
}
impl IsParamTransformer for JalForm {
    fn param_to_instruction_part(&self, _offset: u64, param: &Param) -> BitVec<u32> {
        let param_bits_store = param.unwrap_immediate() as u32;
        let param_bits = param_bits_store.view_bits::<Lsb0>();
        let mut instruction_part = BitVec::new();
        instruction_part.extend_from_bitslice(&param_bits[12..20]);
        instruction_part.push(param_bits[11]);
        instruction_part.extend_from_bitslice(&param_bits[1..11]);
        instruction_part.push(param_bits[20]);
        instruction_part
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut Param) {
        if let Param::Decided(Decided::Immediate(param_value))
        | Param::Resolved(_, Decided::Immediate(param_value)) = param
        {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[12..20].copy_from_bitslice(&instruction_part[0..8]);
            param_bits.set(11, instruction_part[8]);
            param_bits[1..11].copy_from_bitslice(&instruction_part[9..19]);
            param_bits.set(20, instruction_part[19]);
            *param_value = param_bits_store as i32;
        }
    }

    fn default_param(&self) -> Param {
        Param::Decided(Decided::Immediate(0))
    }
    fn bit_count(&self) -> usize {
        20
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::riscv::simple_instruction::Param;

    #[test]
    fn test_argument_to_bits() {
        let transformer = JalForm::new();
        let bits =
            transformer.param_to_instruction_part(0, &Param::Decided(Decided::Immediate(-4)));
        assert_eq!(
            bits,
            bits![u32, Lsb0; 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
        );
        let bits = transformer.param_to_instruction_part(0, &Param::Decided(Decided::Immediate(4)));
        assert_eq!(
            bits,
            bits![u32, Lsb0;0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        let bits =
            transformer.param_to_instruction_part(0, &Param::Decided(Decided::Immediate(0x998)));
        assert_eq!(
            bits,
            bits![u32, Lsb0; 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0]
        );
    }

    #[test]
    fn test_update_argument() {
        let transformer = JalForm::new();
        let mut param = Param::Decided(Decided::Immediate(0));
        transformer.update_param(
            bits![u32, Lsb0; 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0],
            &mut param,
        );
        assert_eq!(
            param,
            Param::Decided(Decided::Immediate(0b0000_0000_1001_1001_1000))
        );
    }
}
