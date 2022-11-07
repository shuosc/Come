use crate::{
    ir::{
        function::GenerateRegister,
        quantity::{self, local, Quantity},
        LocalVariableName,
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
    pub to: Option<LocalVariableName>,
    /// Name of the function to call.
    pub name: String,
    /// Result type.
    pub data_type: Type,
    /// Arguments to pass to the function.
    pub params: Vec<Quantity>,
}

impl GenerateRegister for Call {
    fn register(&self) -> Option<(LocalVariableName, Type)> {
        self.to.clone().map(|it| (it, self.data_type.clone()))
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
    use crate::utility::data_type::Integer;

    use super::*;

    #[test]
    fn test_parse() {
        let result = parse("call i32 foo()").unwrap().1;
        assert_eq!(
            result,
            Call {
                to: None,
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32
                }),
                name: "foo".to_string(),
                params: vec![]
            }
        );
        let result = parse("%1 = call i32 foo(%0)").unwrap().1;
        assert_eq!(
            result,
            Call {
                to: Some(LocalVariableName("1".to_string())),
                data_type: Type::Integer(Integer {
                    signed: true,
                    width: 32
                }),
                name: "foo".to_string(),
                params: vec![LocalVariableName("0".to_string()).into()]
            }
        );
    }
}
