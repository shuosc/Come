use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ir::{
    function::FunctionDefinitionIndex,
    quantity::Quantity,
    statement::{IRStatement, IsIRStatement},
    FunctionDefinition, RegisterName,
};

#[derive(Debug)]
pub struct MemoryAccessInfo {
    pub alloca: FunctionDefinitionIndex,
    // store statements index, in order
    pub store: Vec<FunctionDefinitionIndex>,
    // load statements index, in order
    pub load: Vec<FunctionDefinitionIndex>,
}

impl MemoryAccessInfo {
    // For each store statement, this function will find all load statements in this basic block which
    // load the value it just stored.
    pub fn dorminate_in_basic_block(
        &self,
    ) -> HashMap<FunctionDefinitionIndex, Vec<FunctionDefinitionIndex>> {
        let mut result: HashMap<_, Vec<FunctionDefinitionIndex>> = HashMap::new();
        let mut store_iter = self.store.iter().peekable();
        let mut load_iter = self.load.iter().peekable();
        while let Some(store) = store_iter.next() {
            while let Some(&next_load) = load_iter.peek() && next_load < store {
                load_iter.next();
            }
            let end_index = if let Some(&next_store) = store_iter.peek() {
                let in_same_bb = store.0 == next_store.0;
                if in_same_bb {
                    Some(next_store)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(_end_index) = end_index {
                while let Some(&next_load) = load_iter.peek() && next_load < store {
                    result.entry(store.clone()).or_default().push(load_iter.next().unwrap().clone());
                }
            } else {
                while let Some(&next_load) = load_iter.peek() && next_load.0 == store.0 {
                    result.entry(store.clone()).or_default().push(load_iter.next().unwrap().clone());
                }
            }
        }
        result
    }
}

#[derive(Debug)]
pub struct Analyzer {
    pub content: Rc<RefCell<FunctionDefinition>>,
    pub definition_index: HashMap<RegisterName, FunctionDefinitionIndex>,
    pub use_indexes: HashMap<RegisterName, Vec<FunctionDefinitionIndex>>,
    pub memory_access: HashMap<RegisterName, MemoryAccessInfo>,
}

impl Analyzer {
    pub fn new(content: Rc<RefCell<FunctionDefinition>>) -> Self {
        Self {
            content,
            definition_index: HashMap::new(),
            use_indexes: HashMap::new(),
            memory_access: HashMap::new(),
        }
    }

    pub fn on_statement_remove(&mut self, _index: &FunctionDefinitionIndex) {
        // todo: we should profile which is better: clear all the cache or update (at least part of) the cache
        self.definition_index.clear();
        self.use_indexes.clear();
        self.memory_access.clear();
    }

    pub fn memory_access_info(&mut self) -> &HashMap<RegisterName, MemoryAccessInfo> {
        if self.memory_access.is_empty() {
            let content = self.content.borrow();
            for (index, statement) in content.iter().function_definition_index_enumerate() {
                if matches!(statement, IRStatement::Alloca(_)) {
                    self.memory_access
                        .entry(statement.generate_register().unwrap().0)
                        .or_insert_with(|| MemoryAccessInfo {
                            alloca: index,
                            store: Vec::new(),
                            load: Vec::new(),
                        })
                        .alloca = index.clone();
                } else if matches!(statement, IRStatement::Store(_)) {
                    if let Quantity::RegisterName(local) = &statement.as_store().target {
                        self.memory_access
                            .entry(local.clone())
                            .or_insert_with(|| MemoryAccessInfo {
                                // it's ok to use `index` as the index here, because it will definitly be updated later
                                alloca: index.clone(),
                                store: Vec::new(),
                                load: Vec::new(),
                            })
                            .store
                            .push(index);
                    }
                } else if matches!(statement, IRStatement::Load(_)) {
                    if let Quantity::RegisterName(local) = &statement.as_load().from {
                        self.memory_access
                            .entry(local.clone())
                            .or_insert_with(|| MemoryAccessInfo {
                                // it's ok to use `index` as the index here, because it will definitly be updated later
                                alloca: index.clone(),
                                store: Vec::new(),
                                load: Vec::new(),
                            })
                            .load
                            .push(index);
                    }
                }
            }
        }
        &self.memory_access
    }

    pub fn use_indexes(&mut self, register: &RegisterName) -> &Vec<FunctionDefinitionIndex> {
        if self.use_indexes.is_empty() {
            for (index, statement) in self
                .content
                .borrow()
                .iter()
                .function_definition_index_enumerate()
            {
                let use_registers = statement.use_register();
                for register in use_registers {
                    self.use_indexes
                        .entry(register)
                        .or_insert_with(Vec::new)
                        .push(index.clone());
                }
            }
        }
        self.use_indexes.get(register).unwrap()
    }
}
