use crate::{
    ast,
    ir::{
        function::{
            ir_generator::{rvalue_from_ast, IRGeneratingContext},
            IsIRStatement,
        },
        quantity::{self, local, Quantity},
        RegisterName,
    },
    utility::{data_type, data_type::Type, parsing},
};
use itertools::Itertools;
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
    fn on_register_change(&mut self, from: &RegisterName, to: Quantity) {
        if let Some(result_to) = &self.to
            && result_to == from
        {
            self.to = Some(to.clone().unwrap_local());
        }
        for param in self.params.iter_mut() {
            if let Quantity::RegisterName(param_val) = param {
                if param_val == from {
                    *param = to.clone();
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
            write!(f, "{to_register} = ")?;
        }
        write!(f, "call {} {}(", self.data_type, self.name)?;
        write!(
            f,
            "{}",
            self.params
                .iter()
                .map(|it| format!("{it}"))
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

/// Generate a [`Call`] from an [`ast::expression::FunctionCall`],
/// and append it to the current basic block.
/// Return a [`RegisterName`] which contains the result.
pub fn from_ast(
    ast: &ast::expression::FunctionCall,
    ctx: &mut IRGeneratingContext,
) -> RegisterName {
    let ast::expression::FunctionCall { name, arguments } = ast;
    let function_info = ctx
        .parent_context
        .function_definitions
        .get(name)
        .unwrap()
        .clone();
    let result_register = ctx.next_register_with_type(&function_info.return_type);
    let params = arguments
        .iter()
        .map(|it| rvalue_from_ast(it, ctx))
        .collect_vec();
    ctx.current_basic_block.append_statement(Call {
        to: Some(result_register.clone()),
        name: name.clone(),
        data_type: function_info.return_type.clone(),
        params,
    });
    result_register
}
#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]
    use crate::{
        ast::expression::IntegerLiteral,
        ir::{function::parameter::Parameter, FunctionHeader},
    };

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

    #[test]
    fn test_from_ast() {
        let ast = ast::expression::FunctionCall {
            name: "f".to_string(),
            arguments: vec![IntegerLiteral(1i64).into()],
        };
        let mut parent_ctx = crate::ir::IRGeneratingContext::new();
        parent_ctx.function_definitions.insert(
            "f".to_string(),
            FunctionHeader {
                name: "f".to_string(),
                parameters: vec![Parameter {
                    name: RegisterName("a".to_string()),
                    data_type: data_type::I32.clone(),
                }],
                return_type: data_type::I32.clone(),
            },
        );
        let mut ctx = super::IRGeneratingContext::new(&mut parent_ctx);
        let result = from_ast(&ast, &mut ctx);
        assert_eq!(result, RegisterName("0".to_string()));
        let call_statement = ctx.current_basic_block.content.pop().unwrap();
        assert_eq!(
            call_statement,
            Call {
                to: Some(RegisterName("0".to_string())),
                data_type: data_type::I32.clone(),
                name: "f".to_string(),
                params: vec![1.into()]
            }
            .into()
        );
    }
}
