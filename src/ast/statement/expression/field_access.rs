use super::{variable_ref, LValue};
use crate::utility::parsing;
use nom::{bytes::complete::tag, combinator::map, multi::fold_many0, sequence::preceded, IResult};

/// [`FieldAccess`] represents result of accessing field in a struct.
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct FieldAccess {
    /// Which struct instance to access from.
    pub from: Box<LValue>,
    /// The field name.
    pub name: String,
}

pub fn higher_than_field_access(code: &str) -> IResult<&str, LValue> {
    map(variable_ref::parse, LValue::VariableRef)(code)
}

/// Parse source code to get a [`FieldAccess`].
pub fn parse(code: &str) -> IResult<&str, FieldAccess> {
    let (rest, first) = higher_than_field_access(code)?;
    let (rest, second) = preceded(tag("."), parsing::ident)(rest)?;
    fold_many0(
        preceded(parsing::in_multispace(tag(".")), parsing::ident),
        move || FieldAccess {
            from: Box::new(first.clone()),
            name: second.clone(),
        },
        |acc, next| FieldAccess {
            from: Box::new(LValue::FieldAccess(acc)),
            name: next,
        },
    )(rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_parse() {
        let result = parse("a.b").unwrap().1;
        assert_eq!(
            result,
            FieldAccess {
                from: Box::new(LValue::VariableRef(variable_ref::VariableRef(
                    "a".to_string()
                ))),
                name: "b".to_string()
            }
        );
        let result = parse("a.b.c").unwrap().1;
        assert_eq!(result.name, "c".to_string(),);
    }
}
