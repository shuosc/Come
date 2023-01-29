use bitvec::prelude::*;
use enum_dispatch::enum_dispatch;
use nom::{branch::alt, combinator::map, IResult};

mod bit_at;
mod bits_at;
mod branch_high;
mod branch_low;
mod csr;
mod jal_form;
mod register;
pub use bit_at::BitAt;
pub use bits_at::BitsAt;
pub use branch_high::BranchHigh;
pub use branch_low::BranchLow;
pub use csr::Csr;
pub use jal_form::JalForm;
pub use register::Register;

use super::param::Param;

#[enum_dispatch]
pub trait IsParamTransformer {
    fn param_to_instruction_part(&self, offset: u64, argument: &Param) -> BitVec<u32>;
    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut Param);
    fn default_param(&self) -> Param;
    fn bit_count(&self) -> usize;
}

#[enum_dispatch(IsParamTransformer)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParamTransformer {
    BitAt,
    BitsAt,
    Register,
    Csr,
    JalForm,
    BranchHigh,
    BranchLow,
}

pub fn parse(code: &str) -> IResult<&str, ParamTransformer> {
    alt((
        map(bit_at::parse, ParamTransformer::BitAt),
        map(bits_at::parse, ParamTransformer::BitsAt),
        map(register::parse, ParamTransformer::Register),
        map(csr::parse, ParamTransformer::Csr),
        map(jal_form::parse, ParamTransformer::JalForm),
        map(branch_high::parse, ParamTransformer::BranchHigh),
        map(branch_low::parse, ParamTransformer::BranchLow),
    ))(code)
}
