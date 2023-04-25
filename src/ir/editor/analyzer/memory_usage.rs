use std::{cell::OnceCell, collections::HashMap};

use itertools::Itertools;

use crate::{
    ir::{
        self,
        editor::action::Action,
        function::FunctionDefinitionIndex,
        quantity::Quantity,
        statement::{IRStatement, IsIRStatement},
        FunctionDefinition, RegisterName,
    },
    utility::data_type::Type,
};

use super::IsAnalyzer;

/// [`MemoryAccessInfo`] is about how a range of memory space is accessed.
#[derive(Debug, Clone, Default)]
pub struct MemoryAccessInfo {
    /// alloca statement index
    pub alloca: Option<FunctionDefinitionIndex>,
    // store statements index, in order
    pub store: Vec<FunctionDefinitionIndex>,
    // load statements index, in order
    pub load: Vec<FunctionDefinitionIndex>,
    store_group_by_basic_block: OnceCell<HashMap<usize, Vec<usize>>>,
    load_group_by_basic_block: OnceCell<HashMap<usize, Vec<usize>>>,
}

impl MemoryAccessInfo {
    /// Group store statements by basic block.
    fn store_group_by_basic_block(&self) -> &HashMap<usize, Vec<usize>> {
        self.store_group_by_basic_block.get_or_init(|| {
            self.store
                .iter()
                .group_by(|it| it.0)
                .into_iter()
                .map(|(basic_block_id, it)| {
                    (basic_block_id, it.into_iter().map(|it| it.1).collect())
                })
                .collect()
        })
    }

    /// Group load statements by basic block.
    fn load_group_by_basic_block(&self) -> &HashMap<usize, Vec<usize>> {
        self.load_group_by_basic_block.get_or_init(|| {
            self.load
                .iter()
                .group_by(|it| it.0)
                .into_iter()
                .map(|(basic_block_id, it)| {
                    (basic_block_id, it.into_iter().map(|it| it.1).collect())
                })
                .collect()
        })
    }

    /// Find all loades which are
    /// - in the same basic block as the given store
    /// - appear after the given store
    pub fn loads_dorminated_by_store_in_block(
        &self,
        store: &FunctionDefinitionIndex,
    ) -> Vec<FunctionDefinitionIndex> {
        let store_in_basic_block = self.store_group_by_basic_block().get(&store.0).unwrap();
        let next_store_index = store_in_basic_block
            .iter()
            .find(|&&it| it > store.1)
            .cloned()
            .unwrap_or(usize::MAX);
        self.load_group_by_basic_block()
            .get(&store.0)
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|&&it| it > store.1 && it < next_store_index)
            .map(|it| (store.0, *it).into())
            .collect_vec()
    }
}

/// [`MemoryUsage`] is for analyzing how a function uses stack memory.
#[derive(Debug, Default)]
pub struct MemoryUsage {
    memory_access: OnceCell<HashMap<RegisterName, MemoryAccessInfo>>,
}

impl MemoryUsage {
    /// Create a new [`MemoryUsage`].
    pub fn new() -> Self {
        Self {
            memory_access: OnceCell::new(),
        }
    }

    /// Get the [`MemoryAccessInfo`] of the given variable.
    fn memory_access_info(
        &self,
        function: &ir::FunctionDefinition,
        variable_name: &RegisterName,
    ) -> &MemoryAccessInfo {
        self.memory_access(function).get(variable_name).unwrap()
    }

    /// All variables which are allocated on stack.
    fn memory_access_variables(
        &self,
        function: &ir::FunctionDefinition,
    ) -> impl Iterator<Item = &RegisterName> {
        self.memory_access(function).keys()
    }

    /// All variables and their type which are allocated on stack.
    fn memory_access_variables_and_types(
        &self,
        function: &ir::FunctionDefinition,
    ) -> HashMap<RegisterName, Type> {
        self.memory_access(function)
            .iter()
            .map(|(variable, info)| {
                let data_type = function[info.alloca.clone().unwrap()]
                    .as_alloca()
                    .alloc_type
                    .clone();
                (variable.clone(), data_type)
            })
            .collect()
    }

    fn memory_access(
        &self,
        function: &ir::FunctionDefinition,
    ) -> &HashMap<RegisterName, MemoryAccessInfo> {
        self.memory_access
            .get_or_init(|| self.init_memory_access(function))
    }

    fn init_memory_access(
        &self,
        function: &ir::FunctionDefinition,
    ) -> HashMap<RegisterName, MemoryAccessInfo> {
        let mut memory_access: HashMap<RegisterName, MemoryAccessInfo> = HashMap::new();
        for (index, statement) in function.iter().function_definition_index_enumerate() {
            match statement {
                IRStatement::Alloca(_) => {
                    memory_access
                        .entry(statement.generate_register().unwrap().0)
                        .or_default()
                        .alloca = Some(index.clone());
                }
                IRStatement::Store(store) => {
                    if let Quantity::RegisterName(local) = &store.target {
                        memory_access
                            .entry(local.clone())
                            .or_default()
                            .store
                            .push(index);
                    }
                }
                IRStatement::Load(load) => {
                    if let Quantity::RegisterName(local) = &load.from {
                        memory_access
                            .entry(local.clone())
                            .or_default()
                            .load
                            .push(index);
                    }
                }
                _ => (),
            }
        }
        memory_access
    }
}

pub struct BindedMemoryUsage<'item, 'bind: 'item> {
    bind_on: &'bind FunctionDefinition,
    item: &'item MemoryUsage,
}

impl<'item, 'bind: 'item> BindedMemoryUsage<'item, 'bind> {
    pub fn memory_access_info(&self, variable_name: &RegisterName) -> &MemoryAccessInfo {
        self.item.memory_access_info(self.bind_on, variable_name)
    }
    pub fn memory_access_variables(&self) -> impl Iterator<Item = &RegisterName> {
        self.item.memory_access_variables(self.bind_on)
    }
    pub fn memory_access_variables_and_types(&self) -> HashMap<RegisterName, Type> {
        self.item.memory_access_variables_and_types(self.bind_on)
    }
}

impl<'item, 'bind: 'item> IsAnalyzer<'item, 'bind> for MemoryUsage {
    fn on_action(&mut self, _action: &Action) {
        // todo: optimization
        self.memory_access.take();
    }

    type Binded = BindedMemoryUsage<'item, 'bind>;

    fn bind(&'item self, content: &'bind ir::FunctionDefinition) -> Self::Binded {
        BindedMemoryUsage {
            bind_on: content,
            item: self,
        }
    }
}
