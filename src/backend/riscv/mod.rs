use crate::{ir, utility::data_type};
use std::{collections::HashMap, str};
/// Compiling a function.
mod function;
/// Register assign.
mod register_assign;

/// Context for compiling IR to asm.
pub struct Context {
    /// Struct type definitions
    pub struct_definitions: HashMap<String, ir::TypeDefinition>,
}

/// Implement by the [`data_type::Type`] struct for calculating the size of a type.
pub trait HasSize {
    fn size(&self, ctx: &Context) -> usize;
}

impl HasSize for data_type::Type {
    fn size(&self, ctx: &Context) -> usize {
        match self {
            data_type::Type::Integer(integer) => integer.width,
            data_type::Type::StructRef(name) => {
                let struct_definition = ctx.struct_definitions.get(name).unwrap();
                struct_definition
                    .fields
                    .iter()
                    .map(|field_type| field_type.size(ctx))
                    .sum()
            }
            data_type::Type::None => 0,
            data_type::Type::Address => 32,
        }
    }
}

impl Context {
    pub fn field_offset(&self, struct_name: &str, field_index: usize) -> usize {
        let struct_definition = self.struct_definitions.get(struct_name).unwrap();
        let mut offset = 0;
        for i in 0..field_index {
            offset += struct_definition.fields[i].size(self);
        }
        offset
    }
}

/// Emit assembly code for ir.
pub fn emit_code(ir: &[ir::IR]) -> String {
    let mut code = String::new();
    let mut ctx = Context {
        struct_definitions: HashMap::new(),
    };
    for ir in ir {
        match ir {
            ir::IR::FunctionDefinition(function_definition) => {
                code.push_str(function::emit_code(function_definition, &mut ctx).as_str());
            }
            ir::IR::TypeDefinition(type_definition) => {
                ctx.struct_definitions
                    .insert(type_definition.name.clone(), type_definition.clone());
            }
            ir::IR::GlobalDefinition(_) => todo!(),
        }
    }
    code
}
