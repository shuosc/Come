use std::{
    cell::{Ref, RefCell, RefMut},
    ops::Index,
    rc::Rc,
};

use function::statement::{phi::PhiSource, IRStatement, IsIRStatement, Phi};
use itertools::Itertools;

use crate::ir::{
    function::{self, basic_block::BasicBlock, FunctionDefinitionIndex},
    quantity::Quantity,
    FunctionDefinition, RegisterName,
};

use super::analyzer::Analyzer;

// variable name, inserted at basic block index, [from_block, value]
type PhiEntry = (RegisterName, usize, Vec<(usize, Quantity)>);

#[derive(Debug, Default)]
pub struct EditActionsBatch {
    remove: Vec<FunctionDefinitionIndex>,
    insert_phis: Vec<PhiEntry>,
    replace_register: Vec<(RegisterName, Quantity)>,
}

impl EditActionsBatch {
    pub fn remove(&mut self, index: impl Into<FunctionDefinitionIndex>) {
        self.remove.push(index.into());
    }
    pub fn insert_phi(
        &mut self,
        variable_name: RegisterName,
        from_basic_block_index: usize,
        to_basic_block_index: usize,
        value: Quantity,
    ) {
        if let Some((_, _, existing)) =
            self.insert_phis
                .iter_mut()
                .find(|(existing_variable_name, bb_index, _)| {
                    existing_variable_name == &variable_name && bb_index == &to_basic_block_index
                })
        {
            existing.push((from_basic_block_index, value));
        } else {
            self.insert_phis.push((
                variable_name,
                to_basic_block_index,
                vec![(from_basic_block_index, value)],
            ));
        }
    }
    pub fn replace(&mut self, from: RegisterName, to: Quantity) {
        self.replace_register.push((from, to));
    }

    pub fn then(mut self, other: Self) -> Self {
        let Self {
            remove,
            insert_phis,
            replace_register,
        } = other;
        self.remove.extend(remove);
        for (variable_name, to_bb_index, from) in insert_phis {
            for (from_bb_index, value) in from {
                self.insert_phi(variable_name.clone(), from_bb_index, to_bb_index, value);
            }
        }
        self.replace_register.extend(replace_register);
        self
    }
}

pub struct EditActionsBatches {
    batches: Vec<EditActionsBatch>,
}

pub struct IRFunctionEditor {
    content: Rc<RefCell<FunctionDefinition>>,
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

    pub fn push_front_statement(
        &mut self,
        basic_block_index: usize,
        statement: impl Into<IRStatement>,
    ) {
        let mut basic_block = self.index_mut(basic_block_index);
        basic_block.content.insert(0, statement.into());
        drop(basic_block);
        self.analyzer.on_statement_insert();
    }

    pub fn remove_statement(&mut self, index: &function::FunctionDefinitionIndex) {
        Rc::as_ref(&self.content).borrow_mut().remove(index);
        self.analyzer.on_statement_remove(index);
    }

    pub fn replace_register(&mut self, register: &RegisterName, value: Quantity) {
        Rc::as_ref(&self.content)
            .borrow_mut()
            .iter_mut()
            .for_each(|statement| {
                statement.on_register_change(register, value.clone());
            });
    }

    fn generate_phi_node(
        &mut self,
        variable_name: &RegisterName,
        to_be_putted_block_index: usize,
        from: impl Iterator<Item = (String, Quantity)>,
    ) -> Phi {
        let sources = from.map(|(from, value)| PhiSource {
            name: value,
            block: from,
        });
        let data_type = self.analyzer.alloca_register_type(variable_name);
        let to_be_putted_block = self.index(to_be_putted_block_index);
        let to_be_putted_block_name = to_be_putted_block.name.as_ref().unwrap();
        Phi {
            to: RegisterName(format!("{}_{}", variable_name.0, to_be_putted_block_name)),
            data_type,
            from: sources.collect(),
        }
    }

    pub fn execute_batch(&mut self, batch: EditActionsBatch) {
        let EditActionsBatch {
            mut remove,
            replace_register,
            insert_phis: push_front,
        } = batch;
        remove.sort();
        remove.dedup();
        for index_to_remove in remove.iter().rev() {
            self.remove_statement(index_to_remove);
        }
        for (variable_name, basic_block_index, from) in push_front {
            let from = from
                .into_iter()
                .map(|it| (self.index(it.0).name.clone().unwrap(), it.1))
                .collect_vec();
            let phi_node =
                self.generate_phi_node(&variable_name, basic_block_index, from.into_iter());
            self.push_front_statement(basic_block_index, phi_node);
        }
        for (register_name, value) in replace_register {
            self.replace_register(&register_name, value);
        }
    }

    pub fn done(self) -> FunctionDefinition {
        let IRFunctionEditor { content, analyzer } = self;
        drop(analyzer);
        Rc::try_unwrap(content).unwrap().into_inner()
    }

    pub fn content(&self) -> Ref<FunctionDefinition> {
        Rc::as_ref(&self.content).borrow()
    }

    pub fn index(&self, index: usize) -> Ref<BasicBlock> {
        Ref::map(Rc::as_ref(&self.content).borrow(), |it| {
            it.content.get(index).unwrap()
        })
    }

    pub fn index_mut(&mut self, index: usize) -> RefMut<BasicBlock> {
        RefMut::map(Rc::as_ref(&self.content).borrow_mut(), |it| {
            it.content.get_mut(index).unwrap()
        })
    }

    pub fn index_statement<Idx: Into<FunctionDefinitionIndex>>(
        &self,
        index: Idx,
    ) -> Ref<IRStatement> {
        let index = index.into();
        Ref::map(Rc::as_ref(&self.content).borrow(), |it| it.index(index))
    }
}
