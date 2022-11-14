use crate::{
    ir::{
        function::IsIRStatement,
        quantity::{self, local, Quantity},
        RegisterName,
    },
    utility::{data_type, data_type::Type, parsing},
};
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};
use std::fmt::{self, Display, Formatter};

/// [`Call`] instruction.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Call {
    /// Where to store the result of the call.
    pub to: Option<RegisterName>,
    /// Name of the function to call.
    pub name: String,
    /// Result type.
    pub data_type: Type,
    /// Arguments to pass to the function.
    pub params: Vec<Quantity>,
}

impl IsIRStatement for Call {
    fn on_register_change(&mut self, from: &RegisterName, to: &Quantity) {
        if let Some(result_to) = &self.to && result_to == from {
            self.to = Some(to.clone().unwrap_local());
        }
        for param in self.params.iter_mut() {
            if let Quantity::RegisterName(param_val) = param {
                if param_val == from {
                    *param_val = to.clone().unwrap_local();
                }
            }
        }
    }

    fn generate_register(&self) -> Option<(RegisterName, Type)> {
        self.to.clone().map(|it| (it, self.data_type.clone()))
    }

    fn use_register(&self) -> Vec<RegisterName> {
        self.params
            .iter()
            .filter_map(|it| {
                if let Quantity::RegisterName(register) = it {
                    Some(register.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(to_register) = &self.to {
            write!(f, "{} = ", to_register)?;
        }
        write!(f, "call {} {}(", self.data_type, self.name)?;
        write!(
            f,
            "{}",
            self.params
                .iter()
                .map(|it| format!("{}", it))
                .collect::<Vec<_>>()
                .join(",")
        )?;
        write!(f, ")")
    }
}

/// Parse a [`Call`] instruction.
pub fn parse(code: &str) -> IResult<&str, Call> {
    map(
        tuple((
            opt(map(tuple((local::parse, space0, tag("="), space0)), |x| {
                x.0
            })),
            tag("call"),
            space0,
            data_type::parse,
            space0,
            parsing::ident,
            delimited(
                tag("("),
                separated_list0(tuple((space0, tag(","), space0)), quantity::parse),
                tag(")"),
            ),
        )),
        |(result, _, _, data_type, _, name, params)| Call {
            to: result,
            data_type,
            name,
            params,
        },
    )(code)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("call i32 foo()").unwrap().1;
        assert_eq!(
            result,
            Call {
                to: None,
                data_type: data_type::I32.clone(),
                name: "foo".to_string(),
                params: vec![]
            }
        );
        let result = parse("%1 = call i32 foo(%0)").unwrap().1;
        assert_eq!(
            result,
            Call {
                to: Some(RegisterName("1".to_string())),
                data_type: data_type::I32.clone(),
                name: "foo".to_string(),
                params: vec![RegisterName("0".to_string()).into()]
            }
        );
    }
}
