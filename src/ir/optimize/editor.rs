use std::{cell::RefCell, rc::Rc};

use crate::ir::{function, FunctionDefinition};

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

    pub fn done(self) -> FunctionDefinition {
        let IRFunctionEditor { content, analyzer } = self;
        drop(analyzer);
        Rc::try_unwrap(content).unwrap().into_inner()
    }
}
