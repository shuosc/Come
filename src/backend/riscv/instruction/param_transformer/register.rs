use nom::{bytes::complete::tag, combinator::map, IResult};

use crate::backend::riscv::instruction::param::ParsedParam;

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
    fn argument_to_bits(&self, _address: u64, argument: &ParsedParam) -> Vec<bool> {
        bits_at(argument.unwrap_register() as u32, 0..5)
    }
    fn update_argument(&self, instruction_part: &[bool], param: &mut ParsedParam) {
        if let ParsedParam::Register(value) = param {
            for (index, &bit) in (0usize..5).zip(instruction_part.iter()) {
                *value |= (bit as u8) << index;
            }
        }
    }

    fn default_argument(&self) -> ParsedParam {
        ParsedParam::Register(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_to_bits() {
        let transformer = Register::new();
        let bits = transformer.argument_to_bits(0, &ParsedParam::Register(0x1f));
        assert_eq!(bits, vec![true, true, true, true, true]);
    }

    #[test]
    fn test_update_argument() {
        let transformer = Register::new();
        let mut param = ParsedParam::Register(0);
        transformer.update_argument(&[true, true, true, true, true], &mut param);
        assert_eq!(param, ParsedParam::Register(0x1f));
    }
}
