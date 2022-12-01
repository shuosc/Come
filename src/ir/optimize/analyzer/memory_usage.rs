use std::{cell::OnceCell, collections::HashMap};

use itertools::Itertools;

use crate::{
    ir::{
        function::FunctionDefinitionIndex,
        quantity::Quantity,
        statement::{IRStatement, IsIRStatement},
        FunctionDefinition, RegisterName,
    },
    utility::data_type::Type,
};

#[derive(Debug, Clone)]
pub struct MemoryAccessInfo {
    pub alloca: FunctionDefinitionIndex,
    // store statements index, in order
    pub store: Vec<FunctionDefinitionIndex>,
    // load statements index, in order
    pub load: Vec<FunctionDefinitionIndex>,
    store_group_by_basic_block: OnceCell<HashMap<usize, Vec<usize>>>,
    load_group_by_basic_block: OnceCell<HashMap<usize, Vec<usize>>>,
}

impl MemoryAccessInfo {
    // for any variable, only the last store in each basic block may affect other basic blocks
    pub fn stores_used_by_other_blocks(&self) -> Vec<FunctionDefinitionIndex> {
        self.store
            .iter()
            .group_by(|it| it.0)
            .into_iter()
            .map(|(_, it)| it.into_iter().last().unwrap())
            .cloned()
            .collect_vec()
    }

    pub fn store_group_by_basic_block(&self) -> &HashMap<usize, Vec<usize>> {
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

    pub fn load_group_by_basic_block(&self) -> &HashMap<usize, Vec<usize>> {
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

    pub fn loads_dorminated_by_store(
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

pub struct MemoryUsageAnalyzer<'a> {
    content: &'a FunctionDefinition,
    memory_access: OnceCell<HashMap<RegisterName, MemoryAccessInfo>>,
}

impl<'a> MemoryUsageAnalyzer<'a> {
    pub fn new(content: &'a FunctionDefinition) -> Self {
        Self {
            content,
            memory_access: OnceCell::new(),
        }
    }

    pub fn memory_access_info(&self, variable_name: &RegisterName) -> &MemoryAccessInfo {
        self.memory_access().get(variable_name).unwrap()
    }

    pub fn variables(&self) -> impl Iterator<Item = &RegisterName> {
        self.memory_access().keys()
    }

    pub fn variable_and_types(&self) -> HashMap<RegisterName, Type> {
        self.memory_access()
            .iter()
            .map(|(variable, info)| {
                let data_type = self.content[info.alloca.clone()]
                    .as_alloca()
                    .alloc_type
                    .clone();
                (variable.clone(), data_type)
            })
            .collect()
    }

    fn memory_access(&self) -> &HashMap<RegisterName, MemoryAccessInfo> {
        self.memory_access.get_or_init(|| {
            let mut memory_access = HashMap::new();
            for (index, statement) in self.content.iter().function_definition_index_enumerate() {
                if matches!(statement, IRStatement::Alloca(_)) {
                    memory_access
                        .entry(statement.generate_register().unwrap().0)
                        .or_insert_with(|| MemoryAccessInfo {
                            alloca: index,
                            store: Vec::new(),
                            load: Vec::new(),
                            store_group_by_basic_block: OnceCell::new(),
                            load_group_by_basic_block: OnceCell::new(),
                        })
                        .alloca = index.clone();
                } else if matches!(statement, IRStatement::Store(_)) {
                    if let Quantity::RegisterName(local) = &statement.as_store().target {
                        memory_access
                            .entry(local.clone())
                            .or_insert_with(|| MemoryAccessInfo {
                                // it's ok to use `index` as the index here, because it will definitly be updated later
                                alloca: index.clone(),
                                store: Vec::new(),
                                load: Vec::new(),
                                store_group_by_basic_block: OnceCell::new(),
                                load_group_by_basic_block: OnceCell::new(),
                            })
                            .store
                            .push(index);
                    }
                } else if matches!(statement, IRStatement::Load(_)) {
                    if let Quantity::RegisterName(local) = &statement.as_load().from {
                        memory_access
                            .entry(local.clone())
                            .or_insert_with(|| MemoryAccessInfo {
                                // it's ok to use `index` as the index here, because it will definitly be updated later
                                alloca: index.clone(),
                                store: Vec::new(),
                                load: Vec::new(),
                                store_group_by_basic_block: OnceCell::new(),
                                load_group_by_basic_block: OnceCell::new(),
                            })
                            .load
                            .push(index);
                    }
                }
            }
            memory_access
        })
    }
}
