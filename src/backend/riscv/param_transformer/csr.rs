use crate::backend::riscv::ParsedParam;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::{bits_at, IsParamTransformer};

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
    fn argument_to_bits(&self, _address: u64, argument: &ParsedParam) -> Vec<bool> {
        bits_at(argument.unwrap_csr() as u32, 0..12)
    }

    fn update_argument(&self, instruction_part: &[bool], param: &mut ParsedParam) {
        if let ParsedParam::Csr(value) = param {
            for (index, &bit) in (0usize..12).zip(instruction_part.iter()) {
                *value |= (bit as u16) << index;
            }
        }
    }

    fn default_argument(&self) -> ParsedParam {
        ParsedParam::Csr(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::riscv::ParsedParam;

    #[test]
    fn test_argument_to_bits() {
        let transformer = Csr::new();
        let bits = transformer.argument_to_bits(0, &ParsedParam::Csr(0x7c0));
        assert_eq!(
            bits,
            vec![false, false, false, false, false, false, true, true, true, true, true, false]
        );
    }

    #[test]
    fn test_update_argument() {
        let transformer = Csr::new();
        let mut param = ParsedParam::Csr(0);
        transformer.update_argument(
            &[
                false, false, false, false, false, false, true, true, true, true, true, false,
            ],
            &mut param,
        );
        assert_eq!(param, ParsedParam::Csr(0x7c0));
    }
}
