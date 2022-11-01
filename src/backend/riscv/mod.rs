use self::register_assign::RegisterAssign;
use crate::ir;
use std::collections::HashMap;
/// Compiling a function.
mod function;
/// Register assign.
mod register_assign;

/// Context for compiling a function.
pub struct FunctionCompileContext {
    /// Where a local variable is assigned to.
    pub local_assign: HashMap<ir::LocalVariableName, RegisterAssign>,
    /// Some times we need to do some cleanup before return (eg, pop the stack frame)
    /// So we can jump to this label instead of return directly.
    pub cleanup_label: Option<String>,
}

/// Emit assembly code for ir.
pub fn emit_code(ir: &[ir::IR]) -> String {
    let mut code = String::new();
    for ir in ir {
        code.push_str(
            match ir {
                ir::IR::FunctionDefinition(function_definition) => {
                    function::emit_code(function_definition)
                }
                ir::IR::TypeDefinition(_) => todo!(),
                ir::IR::GlobalDefinition(_) => todo!(),
            }
            .as_str(),
        )
    }
    code
}
