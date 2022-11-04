use super::{
    basic_block::BasicBlock,
    statement::{Ret, Terminator},
};
use crate::{
    ast::{self, expression::VariableRef, statement::Statement},
    ir::{quantity::Quantity, LocalVariableName},
    utility::data_type::{Integer, Type},
};
use std::{collections::HashMap, mem, vec};

mod assign;
mod declare;
pub mod expression;
mod if_statement;
mod return_statement;
mod while_statement;
pub use expression::{lvalue_from_ast, rvalue_from_ast};

/// [`IRGeneratingContext`] is used to collect the basic blocks generated.
pub struct IRGeneratingContext<'a> {
    /// Parent [`crate::ir::IRGeneratingContext`]
    pub parent_context: &'a mut crate::ir::IRGeneratingContext,
    /// [`BasicBlock`]s that are already generated.
    pub done_basic_blocks: Vec<BasicBlock>,
    /// The [`BasicBlock`] that are in construction.
    pub current_basic_block: BasicBlock,
    /// Types of variables. The latter in the [`Vec`] has higher priority.
    pub variable_types_stack: Vec<HashMap<VariableRef, Type>>,
    /// Typed of local variables.
    pub local_variable_types: HashMap<LocalVariableName, Type>,
}

impl<'a> IRGeneratingContext<'a> {
    /// Create a new [`IRGeneratingContext`].
    pub fn new(parent_context: &'a mut crate::ir::IRGeneratingContext) -> Self {
        Self {
            parent_context,
            done_basic_blocks: Vec::new(),
            current_basic_block: BasicBlock::new(),
            variable_types_stack: vec![HashMap::new()],
            local_variable_types: HashMap::new(),
        }
    }

    /// Finish the current [`BasicBlock`] with `terminator` and start a new one.
    pub fn end_current_basic_block_with(&mut self, terminator: impl Into<Terminator>) {
        self.current_basic_block.terminator = Some(terminator.into());
        self.done_basic_blocks.push(mem::replace(
            &mut self.current_basic_block,
            BasicBlock::new(),
        ));
    }

    /// Finish generating [`BasicBlock`]s for the current function.
    /// Return the collected [`BasicBlock`]s.
    pub fn done(mut self) -> Vec<BasicBlock> {
        if !self.current_basic_block.empty() {
            if self.current_basic_block.terminator.is_none() {
                self.current_basic_block.terminator = Some(Ret { value: None }.into());
            }
            self.done_basic_blocks.push(self.current_basic_block);
        }
        self.done_basic_blocks
            .into_iter()
            .filter(|it| !it.empty())
            .collect()
    }

    /// Decide a variable's type.
    pub fn type_of_variable(&self, variable: &VariableRef) -> Type {
        self.variable_types_stack
            .iter()
            .rev()
            .find_map(|it| it.get(variable))
            .unwrap()
            .clone()
    }

    /// Decide a field's type.
    pub fn type_of_field(&self, field_access: &ast::expression::FieldAccess) -> Type {
        let ast::expression::FieldAccess { from: _, name } = field_access;
        let parent_type = match field_access.from.as_ref() {
            ast::expression::LValue::VariableRef(variable) => self.type_of_variable(variable),
            ast::expression::LValue::FieldAccess(field_access) => self.type_of_field(field_access),
        };
        match parent_type {
            Type::StructRef(s) => {
                let struct_definition = self.parent_context.type_definitions.get(&s).unwrap();
                let field_index = struct_definition.field_names.get(name).unwrap();
                struct_definition.field_types[*field_index].clone()
            }
            _ => panic!("Cannot access field from non-struct type"),
        }
    }

    /// Decide a local variable's type.
    pub fn type_of_quantity(&self, variable: &Quantity) -> Type {
        match variable {
            Quantity::LocalVariableName(name) => self.local_variable_types[name].clone(),
            Quantity::GlobalVariableName(name) => self.parent_context.global_definitions[&name.0]
                .data_type
                .clone(),
            Quantity::NumberLiteral(_) => {
                // todo: auto decide integer types
                Type::Integer(Integer {
                    signed: true,
                    width: 32,
                })
            }
        }
    }

    /// Generate a [`LocalVariableName`] and record its type
    pub fn next_register_with_type(&mut self, data_type: &Type) -> LocalVariableName {
        let register = self.parent_context.next_register();
        self.local_variable_types
            .insert(register.clone(), data_type.clone());
        register
    }
}

/// Generate IR from [`ast::statement::compound::Compound`].
pub fn compound_from_ast(ast: &ast::statement::compound::Compound, ctx: &mut IRGeneratingContext) {
    for statement in &ast.0 {
        match statement {
            Statement::Declare(declare) => declare::from_ast(declare, ctx),
            Statement::Assign(assign) => assign::from_ast(assign, ctx),
            Statement::Return(return_statement) => {
                return_statement::from_ast(return_statement, ctx)
            }
            Statement::If(if_statement) => if_statement::from_ast(if_statement, ctx),
            Statement::While(while_statement) => while_statement::from_ast(while_statement, ctx),
            Statement::FunctionCall(_) => todo!(),
        }
    }
}
