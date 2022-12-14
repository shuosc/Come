use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
};

use either::Either;
use itertools::Itertools;

use crate::{
    ir::{
        function::{parameter::Parameter, FunctionDefinitionIndex},
        statement::{IRStatement, IsIRStatement},
        FunctionDefinition, RegisterName,
    },
    utility::data_type,
};

use super::control_flow::ControlFlowGraph;
pub struct RegisterUsage<'a> {
    content: &'a FunctionDefinition,
    // define statementindex or parameter id
    pub define_index: Either<FunctionDefinitionIndex, usize>,
    pub use_indexes: Vec<FunctionDefinitionIndex>,
}

impl<'a> RegisterUsage<'a> {
    pub fn alloca_type(&self) -> data_type::Type {
        self.content[self.define_index.as_ref().unwrap_left().clone()]
            .as_alloca()
            .alloc_type
            .clone()
    }

    pub fn data_type(&self) -> data_type::Type {
        match &self.define_index {
            Either::Left(define_index) => self.content[define_index.clone()]
                .generate_register()
                .unwrap()
                .1
                .clone(),
            Either::Right(parameter_index) => {
                self.content.parameters[*parameter_index].data_type.clone()
            }
        }
    }

    pub fn define_statement(&self) -> &IRStatement {
        &self.content[self.define_index.as_ref().unwrap_left().clone()]
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
}

impl<'a> RegisterUsageAnalyzer<'a> {
    pub fn new(content: &'a FunctionDefinition) -> Self {
        Self {
            content,
            register_usages: OnceCell::new(),
            uses_grouped_by_block: OnceCell::new(),
            define_grouped_by_block: OnceCell::new(),
        }
    }

    pub fn registers(&self) -> Vec<&RegisterName> {
        // we want the result in order, so that we can make unit tests easier
        // Maybe use IndexMap for register_usages in the future
        let mut registers: Vec<_> = self.register_usages().keys().collect();
        registers.sort();
        registers
    }

    pub fn get(&self, register: &RegisterName) -> &RegisterUsage {
        self.register_usages().get(register).unwrap()
    }

    pub fn register_usages(&self) -> &HashMap<RegisterName, RegisterUsage> {
        self.register_usages.get_or_init(|| {
            let mut register_usages = HashMap::new();
            for (index, param) in self.content.parameters.iter().enumerate() {
                register_usages.insert(
                    param.name.clone(),
                    RegisterUsage {
                        content: self.content,
                        define_index: Either::Right(index),
                        use_indexes: Vec::new(),
                    },
                );
            }
            for (index, statement) in self.content.iter().function_definition_index_enumerate() {
                if let Some((define_register_name, _)) = statement.generate_register() {
                    register_usages
                        .entry(define_register_name)
                        .or_insert_with(|| RegisterUsage {
                            content: self.content,
                            define_index: Either::Left(index.clone()),
                            use_indexes: Vec::new(),
                        })
                        .define_index = Either::Left(index.clone());
                }
                for use_register_name in statement.use_register() {
                    register_usages
                        .entry(use_register_name)
                        .or_insert_with(|| RegisterUsage {
                            content: self.content,
                            define_index: Either::Left(index.clone()),
                            use_indexes: Vec::new(),
                        })
                        .use_indexes
                        .push(index.clone());
                }
            }
            register_usages
        })
    }

    pub fn registers_defined_in_block(&self, block_id: usize) -> HashSet<RegisterName> {
        self.define_grouped_by_block
            .get_or_init(|| {
                let mut define_grouped_by_block: HashMap<usize, HashSet<RegisterName>> =
                    HashMap::new();
                if define_grouped_by_block.is_empty() {
                    for (register_name, usage) in self.register_usages() {
                        if let Some(define_in_block) = usage.define_index.as_ref().left().map(|it| it.0) {
                            define_grouped_by_block
                                .entry(define_in_block)
                                .or_default()
                                .insert(register_name.clone());
                        }
                    }
                }
                define_grouped_by_block
            })
            .get(&block_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn registers_used_in_block(&self, block_id: usize) -> HashSet<RegisterName> {
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
            .cloned()
            .unwrap_or_default()
    }

    pub fn register_active_blocks(
        &self,
        register: &RegisterName,
        control_flow_graph: &ControlFlowGraph,
    ) -> Vec<usize> {
        let register_usages = &self.register_usages().get(register).unwrap();
        let use_blocks = register_usages
            .use_indexes
            .iter()
            .map(|it| it.0)
            .collect::<Vec<_>>();

        let mut result = Vec::new();
        if let Some(define_block) = register_usages.define_index.as_ref().left().map(|it| it.0) {
            if use_blocks.len() == 1 && use_blocks[0] == define_block {
                return vec![define_block];
            }
            result.push(define_block);

            for use_block in use_blocks {
                result.extend(
                    control_flow_graph
                        .passed_block(define_block, use_block)
                        .into_iter(),
                );
            }
        } else {
            for use_block in use_blocks {
                result.extend(
                    control_flow_graph
                        .passed_block(0, use_block)
                        .into_iter(),
                );
            }
        }
        result
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::ir::{
        function::{basic_block::BasicBlock, test_util::*},
        statement::Ret,
        FunctionDefinition,
    };

    // #[test]
    // fn register_active_blocks() {
    //     let function_definition = FunctionDefinition {
    //         name: "f".to_string(),
    //         parameters: Vec::new(),
    //         return_type: data_type::Type::None,
    //         content: vec![
    //             BasicBlock {
    //                 name: Some("bb0".to_string()),
    //                 content: vec![
    //                     binop_constant("m"),
    //                     binop_constant("n"),
    //                     binop_constant("u1"),
    //                     binop("i0", "m", "m"),
    //                     binop("j0", "n", "n"),
    //                     binop("a0", "u1", "u1"),
    //                     binop_constant("r"),
    //                     jump("bb1"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb1".to_string()),
    //                 content: vec![
    //                     phi("i_bb1", "bb1", "i0", "bb4", "i2"),
    //                     phi("a_bb1", "bb1", "a0", "bb4", "a1"),
    //                     binop("i1", "i_bb1", "i_bb1"),
    //                     binop("j1", "j0", "j0"),
    //                     branch("bb2", "bb3"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb2".to_string()),
    //                 content: vec![
    //                     binop("u2", "a_bb1", "a_bb1"),
    //                     binop("a1", "u2", "i1"),
    //                     jump("bb3"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb3".to_string()),
    //                 content: vec![
    //                     binop_constant("u3"),
    //                     binop("i2", "u3", "j1"),
    //                     branch("bb1", "bb4"),
    //                 ],
    //             },
    //             BasicBlock {
    //                 name: Some("bb4".to_string()),
    //                 content: vec![Ret {
    //                     value: Some(RegisterName("r".to_string()).into()),
    //                 }
    //                 .into()],
    //             },
    //         ],
    //     };
    //     let control_flow_graph = ControlFlowGraph::new(&function_definition);
    //     let analyzer = RegisterUsageAnalyzer::new(&function_definition);
    //     assert_eq!(
    //         analyzer.register_active_blocks(&RegisterName("m".to_string()), &control_flow_graph),
    //         vec![0],
    //     );
    //     assert_eq!(
    //         analyzer
    //             .register_active_blocks(&RegisterName("i_bb1".to_string()), &control_flow_graph),
    //         vec![1],
    //     );
    //     assert_eq!(
    //         analyzer.register_active_blocks(&RegisterName("i0".to_string()), &control_flow_graph),
    //         vec![0, 1]
    //     );
    //     assert_eq!(
    //         analyzer.register_active_blocks(&RegisterName("i2".to_string()), &control_flow_graph),
    //         vec![1, 3],
    //     );
    //     assert_eq!(
    //         analyzer.register_active_blocks(&RegisterName("a1".to_string()), &control_flow_graph),
    //         vec![1, 2, 3],
    //     );
    //     assert_eq!(
    //         analyzer.register_active_blocks(&RegisterName("j1".to_string()), &control_flow_graph),
    //         vec![1, 2, 3],
    //     );
    //     assert_eq!(
    //         analyzer.register_active_blocks(&RegisterName("r".to_string()), &control_flow_graph),
    //         vec![0, 1, 2, 3, 4]
    //     );
    // }
}
