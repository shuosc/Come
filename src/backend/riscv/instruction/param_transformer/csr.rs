use crate::backend::riscv::instruction::Param;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::IsParamTransformer;
use bitvec::prelude::*;
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Csr;

impl Csr {
    pub const fn new() -> Self {
        Self
    }
}
pub fn parse(code: &str) -> IResult<&str, Csr> {
    map(tag("csr"), |_| Csr::new())(code)
}
impl IsParamTransformer for Csr {
    fn param_to_instruction_part(&self, _address: u64, param: &Param) -> BitVec<u32> {
        let param_bits_store = param.unwrap_csr() as u32;
        let param_bits = &param_bits_store.view_bits::<Lsb0>();
        param_bits[0..12].to_bitvec()
    }

    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut Param) {
        if let Param::Csr(param_value) = param {
            let mut param_bits_store = *param_value as u32;
            let param_bits = param_bits_store.view_bits_mut::<Lsb0>();
            param_bits[0..12].copy_from_bitslice(instruction_part);
            *param_value = param_bits_store as u16;
        }
    }

    fn default_param(&self) -> Param {
        Param::Csr(0)
    }
    fn bit_count(&self) -> usize {
        12
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::riscv::instruction::Param;

    #[test]
    fn test_argument_to_bits() {
        let transformer = Csr::new();
        let bits = transformer.param_to_instruction_part(0, &Param::Csr(0x7c0));
        assert_eq!(bits, bits![u32, Lsb0; 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0]);
    }

    #[test]
    fn test_update_argument() {
        let transformer = Csr::new();
        let mut param = Param::Csr(0);
        transformer.update_param(
            bits![u32, Lsb0; 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0],
            &mut param,
        );
        assert_eq!(param, Param::Csr(0x7c0));
    }
}
