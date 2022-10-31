// use backend::riscv::emit_function_code;

/// Definitions of AST nodes and their parser.
mod ast;
// pub mod backend;
/// Definitions of IR nodes and their parser, and ir generator functions for generating ir from ast.
mod ir;
pub mod utility;
fn main() {
    let ast = ast::from_source(
        r#"fn f(a: i32) -> i32 {
        let b: i32 = 1;
        let c: i32 = a + b;
        return c;
    }"#,
    )
    .unwrap()
    .1;
    let result = ir::from_ast(&ast);
    for r in result {
        if let ir::IR::FunctionDefinition(f) = r {
            println!("{}", f);
        }
    }
}
