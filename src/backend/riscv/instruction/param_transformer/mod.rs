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
    fn argument_to_bits(&self, address: u64, argument: &ParsedParam) -> Vec<bool>;
    fn update_argument(&self, instruction_part: &[bool], param: &mut ParsedParam);
    fn default_argument(&self) -> ParsedParam;
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

const fn bit_at(num: u32, i: u8) -> bool {
    num & (1 << i) as u32 != 0
}

fn bits_at<I>(num: u32, range: I) -> Vec<bool>
where
    I: IntoIterator,
    I::Item: TryInto<u8>,
    <<I as IntoIterator>::Item as TryInto<u8>>::Error: std::fmt::Debug,
{
    let mut result = Vec::new();
    for bit_id in range.into_iter() {
        result.push(bit_at(num, bit_id.try_into().unwrap()));
    }
    result
}
