/// Result of a binary operator.
pub mod binary_operator;
/// Result of accessing field in a struct.
pub mod field_access;
/// Result of a function call.
pub mod function_call;
/// An expression in brackets.
pub mod in_brackets;
/// An integer literal.
pub mod integer_literal;
/// Result of a unary operator.
pub mod unary_operator;
/// Refer to a variable.
pub mod variable_ref;
// todo: mod subscripting;

/// Enumeration of all expressions which can be assigned to.
pub mod lvalue;
/// Enumeration of all expressions which has a value.
pub mod rvalue;

pub use binary_operator::BinaryOperatorResult;
pub use field_access::FieldAccess;
pub use function_call::FunctionCall;
pub use in_brackets::InBrackets;
pub use integer_literal::IntegerLiteral;
pub use lvalue::LValue;
pub use rvalue::RValue;
pub use unary_operator::UnaryOperatorResult;
pub use variable_ref::VariableRef;
