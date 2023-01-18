use crate::backend::riscv::ParsedParam;
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
    fn argument_to_bits(&self, _address: u64, argument: &ParsedParam) -> Vec<bool> {
        let n = argument.unwrap_immediate() as u64;
        let mut bit_select: Vec<usize> = vec![];
        bit_select.extend(12..20);
        bit_select.push(11);
        bit_select.extend(1..11);
        bit_select.push(20);
        bits_at(n as _, bit_select)
    }

    fn update_argument(&self, instruction_part: &[bool], param: &mut ParsedParam) {
        if let ParsedParam::Immediate(value) = param {
            let mut bit_select: Vec<usize> = vec![];
            bit_select.extend(12..20);
            bit_select.push(11);
            bit_select.extend(1..11);
            bit_select.push(20);
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
    use crate::backend::riscv::ParsedParam;

    #[test]
    fn test_argument_to_bits() {
        let transformer = JalForm::new();
        let bits = transformer.argument_to_bits(0, &ParsedParam::Immediate(-4));
        assert_eq!(
            bits,
            vec![
                true, true, true, true, true, true, true, true, true, false, true, true, true,
                true, true, true, true, true, true, true
            ]
        );
        let bits = transformer.argument_to_bits(0, &ParsedParam::Immediate(4));
        assert_eq!(
            bits,
            vec![
                false, false, false, false, false, false, false, false, false, false, true, false,
                false, false, false, false, false, false, false, false
            ]
        );
        let bits = transformer.argument_to_bits(0, &ParsedParam::Immediate(0x998));
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
        transformer.update_argument(
            &[
                false, false, false, false, false, false, false, false, true, false, false, true,
                true, false, false, true, true, false, false, false,
            ],
            &mut param,
        );
        assert_eq!(param, ParsedParam::Immediate(0b0000_0000_1001_1001_1000));
    }
}
