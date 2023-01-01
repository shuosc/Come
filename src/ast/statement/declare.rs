use super::expression::rvalue::{self, RValue};
use crate::utility::{
    data_type::{self, Type},
    parsing,
};
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, space0, space1},
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

/// [`Declare`] represents a variable declaration.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Declare {
    /// The name of the variable.
    pub variable_name: String,
    /// The type of the variable.
    pub data_type: Type,
    /// The value of the variable.
    pub init_value: Option<RValue>,
}

/// Parse source code to get an [`Declare`].
pub fn parse(code: &str) -> IResult<&str, Declare> {
    map(
        tuple((
            tag("let"),
            space1,
            parsing::ident,
            space0,
            tag(":"),
            space0,
            data_type::parse,
            space0,
            opt(map(
                tuple((space0, tag("="), multispace0, rvalue::parse, multispace0)),
                |(_, _, _, x, _)| x,
            )),
            tag(";"),
        )),
        |(_, _, variable_name, _, _, _, data_type, _, init_value, _)| Declare {
            variable_name,
            data_type,
            init_value,
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let declare = parse("let a: i32;").unwrap().1;
        assert_eq!(declare.variable_name, "a");
        assert_eq!(declare.init_value, None);
        let declare = parse("let b: i32 = 1;").unwrap().1;
        assert_eq!(declare.variable_name, "b");
        assert!(declare.init_value.is_some());
        let declare = parse("let current_value: u32 = load_u32(gpio_address);")
            .unwrap()
            .1;
        assert_eq!(declare.variable_name, "current_value");
        assert!(declare.init_value.is_some());
        // Hope we can make it pass one day!
        let fail = parse("let b = 1;");
        assert!(fail.is_err());
    }
}
