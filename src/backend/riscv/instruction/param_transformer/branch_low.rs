use crate::backend::riscv::instruction::ParsedParam;
use nom::{bytes::complete::tag, combinator::map, IResult};

use super::{bits_at, IsParamTransformer};

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
    fn argument_to_bits(&self, address: u64, argument: &ParsedParam) -> Vec<bool> {
        let n = argument.unwrap_immediate() as i64 - address as i64;
        let mut bit_select: Vec<usize> = vec![11];
        bit_select.extend(1..5);
        bits_at(n as _, bit_select)
    }

    fn update_argument(&self, instruction_part: &[bool], param: &mut ParsedParam) {
        if let ParsedParam::Immediate(value) = param {
            let mut bit_select: Vec<usize> = vec![11];
            bit_select.extend(1..5);
            for (index, &bit) in bit_select.into_iter().zip(instruction_part.iter()) {
                *value |= (bit as i32) << index;
            }
        }
    }

    fn default_argument(&self) -> ParsedParam {
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
        let bits = transformer.argument_to_bits(0, &ParsedParam::Immediate(-4));
        assert_eq!(bits, vec![true, false, true, true, true]);
        let bits = transformer.argument_to_bits(0, &ParsedParam::Immediate(4));
        assert_eq!(bits, vec![false, false, true, false, false]);
        let bits = transformer.argument_to_bits(0, &ParsedParam::Immediate(0x998));
        assert_eq!(bits, vec![true, false, false, true, true]);
    }

    #[test]
    fn test_update_argument() {
        let transformer = BranchLow::new();
        let mut param = ParsedParam::Immediate(0);
        transformer.update_argument(&[true, false, true, true, true], &mut param);
        assert_eq!(param, ParsedParam::Immediate(0b1000_0001_1100));
    }
}
