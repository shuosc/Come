use std::{cell::RefCell, rc::Rc};

use function::statement::IsIRStatement;

use crate::ir::{function, quantity::Quantity, FunctionDefinition, RegisterName};

use super::analyzer::Analyzer;

pub struct IRFunctionEditor {
    pub content: Rc<RefCell<FunctionDefinition>>,
    pub analyzer: Analyzer,
}

impl IRFunctionEditor {
    pub fn new(content: FunctionDefinition) -> Self {
        let content = Rc::new(RefCell::new(content));
        Self {
            analyzer: Analyzer::new(content.clone()),
            content,
        }
    }

    pub fn remove_statement(&mut self, index: &function::FunctionDefinitionIndex) {
        self.content.borrow_mut().remove(index);
        self.analyzer.on_statement_remove(index);
    }

    pub fn replace_register(&mut self, register: &RegisterName, value: &Quantity) {
        self.content.borrow_mut().iter_mut().for_each(|statement| {
            statement.on_register_change(register, value);
        });
    }

    pub fn done(self) -> FunctionDefinition {
        let IRFunctionEditor { content, analyzer } = self;
        drop(analyzer);
        Rc::try_unwrap(content).unwrap().into_inner()
    }
}
