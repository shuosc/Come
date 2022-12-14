use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
};

use itertools::Itertools;

use crate::{
    ir::{
        function::FunctionDefinitionIndex,
        statement::{IRStatement, IsIRStatement},
        FunctionDefinition, RegisterName,
    },
    utility::data_type,
};

use super::control_flow::ControlFlowGraph;
pub struct RegisterUsage<'a> {
    content: &'a FunctionDefinition,
    pub define_index: FunctionDefinitionIndex,
    pub use_indexes: Vec<FunctionDefinitionIndex>,
}

impl<'a> RegisterUsage<'a> {
    pub fn alloca_type(&self) -> data_type::Type {
        self.content[self.define_index.clone()]
            .as_alloca()
            .alloc_type
            .clone()
    }

    pub fn data_type(&self) -> data_type::Type {
        self.content[self.define_index.clone()]
            .generate_register()
            .unwrap()
            .1
            .clone()
    }

    pub fn define_statement(&self) -> &IRStatement {
        &self.content[self.define_index.clone()]
    }

    pub fn use_statements(&self) -> impl Iterator<Item = &IRStatement> {
        self.use_indexes
            .iter()
            .map(|index| &self.content[index.clone()])
    }

    pub fn uses_grouped_by_block(&self) -> BTreeMap<usize, BTreeSet<usize>> {
        self.use_indexes
            .iter()
            .group_by(|it| it.0)
            .into_iter()
            .map(|(bb_index, group)| (bb_index, group.into_iter().map(|it| it.1).collect()))
            .collect()
    }
}

pub struct RegisterUsageAnalyzer<'a> {
    content: &'a FunctionDefinition,
    register_usages: OnceCell<HashMap<RegisterName, RegisterUsage<'a>>>,
    uses_grouped_by_block: OnceCell<HashMap<usize, HashSet<RegisterName>>>,
    define_grouped_by_block: OnceCell<HashMap<usize, HashSet<RegisterName>>>,
    active_info: OnceCell<RegisterActiveInfo>,
}

impl<'a> RegisterUsageAnalyzer<'a> {
    pub fn new(content: &'a FunctionDefinition) -> Self {
        Self {
            content,
            register_usages: OnceCell::new(),
            uses_grouped_by_block: OnceCell::new(),
            define_grouped_by_block: OnceCell::new(),
            active_info: OnceCell::new(),
        }
    }

    pub fn registers(&self) -> Vec<&RegisterName> {
        self.register_usages().keys().collect()
    }

    pub fn get(&self, register: &RegisterName) -> &RegisterUsage {
        self.register_usages().get(register).unwrap()
    }

    pub fn register_usages(&self) -> &HashMap<RegisterName, RegisterUsage> {
        self.register_usages.get_or_init(|| {
            let mut register_usages = HashMap::new();
            for (index, statement) in self.content.iter().function_definition_index_enumerate() {
                if let Some((define_register_name, _)) = statement.generate_register() {
                    register_usages
                        .entry(define_register_name)
                        .or_insert_with(|| RegisterUsage {
                            content: self.content,
                            define_index: index.clone(),
                            use_indexes: Vec::new(),
                        })
                        .define_index = index.clone();
                }
                for use_register_name in statement.use_register() {
                    register_usages
                        .entry(use_register_name)
                        .or_insert_with(|| RegisterUsage {
                            content: self.content,
                            define_index: index.clone(),
                            use_indexes: Vec::new(),
                        })
                        .use_indexes
                        .push(index.clone());
                }
            }
            register_usages
        })
    }

    pub fn define_grouped_by_block(&self, block_id: usize) -> &HashSet<RegisterName> {
        self.define_grouped_by_block
            .get_or_init(|| {
                let mut define_grouped_by_block: HashMap<usize, HashSet<RegisterName>> =
                    HashMap::new();
                if define_grouped_by_block.is_empty() {
                    for (register_name, usage) in self.register_usages() {
                        let define_in_block = usage.define_index.0;
                        define_grouped_by_block
                            .entry(define_in_block)
                            .or_default()
                            .insert(register_name.clone());
                    }
                }
                define_grouped_by_block
            })
            .get(&block_id)
            .unwrap()
    }

    pub fn registers_used_in_block(&self, block_id: usize) -> &HashSet<RegisterName> {
        self.uses_grouped_by_block
            .get_or_init(|| {
                let mut uses_grouped_by_block: HashMap<usize, HashSet<RegisterName>> =
                    HashMap::new();
                if uses_grouped_by_block.is_empty() {
                    for (register_name, usage) in self.register_usages() {
                        for use_in_block in usage.use_indexes.iter().map(|it| it.0) {
                            uses_grouped_by_block
                                .entry(use_in_block)
                                .or_default()
                                .insert(register_name.clone());
                        }
                    }
                }
                uses_grouped_by_block
            })
            .get(&block_id)
            .unwrap()
    }

    pub fn active_info(&self, control_flow_graph: &ControlFlowGraph) -> &RegisterActiveInfo {
        self.active_info.get_or_init(|| {
            let mut current_active_entrance = HashMap::new();
            let mut current_active_exit = HashMap::new();
            for (basic_block_id, _) in self.content.content.iter().enumerate() {
                current_active_entrance.insert(
                    basic_block_id,
                    self.registers_used_in_block(basic_block_id).clone(),
                );
                current_active_exit.insert(basic_block_id, HashSet::new());
            }
            let mut changed = true;
            while changed {
                for (basic_block_id, _) in self.content.content.iter().enumerate() {
                    let to_blocks = control_flow_graph.to_blocks(basic_block_id);
                    let current_active_exit_for_block = to_blocks
                        .into_iter()
                        .map(|it| current_active_entrance[&it].clone())
                        .fold(
                            HashSet::new(),
                            |mut a: HashSet<RegisterName>, b: HashSet<RegisterName>| {
                                a.extend(b);
                                a
                            },
                        );
                    let from_out = current_active_exit_for_block
                        .difference(self.define_grouped_by_block(basic_block_id))
                        .cloned()
                        .collect();
                    let current_active_entrance_for_block: HashSet<_> = self
                        .registers_used_in_block(basic_block_id)
                        .union(&from_out)
                        .cloned()
                        .collect();

                    current_active_exit.insert(basic_block_id, current_active_exit_for_block);
                    let current_active_entrance_for_block_len =
                        current_active_entrance_for_block.len();
                    let origin_entrance = current_active_entrance
                        .insert(basic_block_id, current_active_entrance_for_block)
                        .unwrap();
                    changed = origin_entrance.len() != current_active_entrance_for_block_len;
                }
            }
            RegisterActiveInfo {
                active_on_entrance: current_active_entrance,
                active_on_exit: current_active_exit,
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct RegisterActiveInfo {
    active_on_entrance: HashMap<usize, HashSet<RegisterName>>,
    active_on_exit: HashMap<usize, HashSet<RegisterName>>,
}

impl RegisterActiveInfo {
    pub fn register_active_blocks(&self, register: &RegisterName) -> HashSet<usize> {
        let mut result = HashSet::new();
        for (basic_block_index, active_registers) in self.active_on_exit.iter() {
            if active_registers.contains(register) {
                result.insert(*basic_block_index);
            }
        }
        result
    }
}
