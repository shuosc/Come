use bitvec::{slice::BitSlice, vec::BitVec};
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

use super::param::ParsedParam;

#[enum_dispatch]
pub trait IsParamTransformer {
    // It suppose to have something like
    // `const fn bit_count(&self) -> usize;`
    // But it's not possible to have const fn in trait
    fn param_to_instruction_part(&self, address: u64, argument: &ParsedParam) -> BitVec<u32>;
    fn update_param(&self, instruction_part: &BitSlice<u32>, param: &mut ParsedParam);
    fn default_param(&self) -> ParsedParam;
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

impl ParamTransformer {
    pub const fn bit_count(&self) -> usize {
        match self {
            ParamTransformer::BitAt(x) => x.bit_count(),
            ParamTransformer::BitsAt(x) => x.bit_count(),
            ParamTransformer::Register(x) => x.bit_count(),
            ParamTransformer::Csr(x) => x.bit_count(),
            ParamTransformer::JalForm(x) => x.bit_count(),
            ParamTransformer::BranchHigh(x) => x.bit_count(),
            ParamTransformer::BranchLow(x) => x.bit_count(),
        }
    }
}

pub fn parse(code: &str) -> IResult<&str, ParamTransformer> {
    alt((
        map(BitAt::parse, ParamTransformer::BitAt),
        map(BitsAt::parse, ParamTransformer::BitsAt),
        map(Register::parse, ParamTransformer::Register),
        map(Csr::parse, ParamTransformer::Csr),
        map(JalForm::parse, ParamTransformer::JalForm),
        map(BranchHigh::parse, ParamTransformer::BranchHigh),
        map(BranchLow::parse, ParamTransformer::BranchLow),
    ))(code)
}
