use backend::riscv::emit_function_code;

mod ast;
pub mod backend;
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
            println!("{}", emit_function_code(&f));
        }
    }
}
