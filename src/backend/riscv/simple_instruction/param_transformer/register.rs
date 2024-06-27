use crate::backend::riscv::simple_instruction::param::{Decided, Param};
use bitvec::prelude::*;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::IsParamTransformer;

/// A transformer that extract the register form of the parameter.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Register;

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

impl Register {
    pub const fn new() -> Self {
        Self
    }
}

pub fn parse(code: &str) -> IResult<&str, Register> {
    map(tag("register"), |_| Register::new())(code)
}

impl IsParamTransformer for Register {
    fn param_to_instruction_part(&self, _offset: u64, param: &Param) -> BitVec<u32> {
        let param_bits_store = param.unwrap_register() as u32;
        let param_bits = param_bits_store.view_bits::<Lsb0>();
        param_bits[0..5].to_bitvec()
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut Param) {
        if let Param::Decided(Decided::Register(param_value)) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[0..5].copy_from_bitslice(instruction_part);
            *param_value = param_bits_store as u8;
        }
    }

    fn default_param(&self) -> Param {
        Param::Decided(Decided::Register(0))
    }
    fn bit_count(&self) -> usize {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_to_bits() {
        let transformer = Register::new();
        let bits =
            transformer.param_to_instruction_part(0, &Param::Decided(Decided::Register(0x1f)));
        assert_eq!(bits, bits![1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_update_argument() {
        let transformer = Register::new();
        let mut param = Param::Decided(Decided::Register(0));
        transformer.update_param(bits![u32, Lsb0; 1, 1, 1, 1, 1], &mut param);
        assert_eq!(param, Param::Decided(Decided::Register(0x1f)));
    }
}
