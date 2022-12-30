use std::{cell::OnceCell, collections::HashMap};

use crate::{
    ir::{
        function::FunctionDefinitionIndex,
        statement::{Call, IRStatement, IsIRStatement},
        FunctionDefinition, RegisterName,
    },
    utility::data_type,
};

use super::control_flow::ControlFlowGraph;

/// [`RegisterDefinePosition`] is the position where a register is defined.
pub enum RegisterDefinePosition {
    /// The register is defined in the body of the function.
    Body(FunctionDefinitionIndex),
    /// The register is defined as the function's parameter.
    Parameter(usize),
}

impl RegisterDefinePosition {
    /// Get the index of the statement where the register is defined.
    /// Panics if the register is not defined in the function body.
    pub fn unwrap_body(&self) -> &FunctionDefinitionIndex {
        if let Self::Body(index) = self {
            index
        } else {
            panic!("called `RegisterDefinePosition::unwrap_body()` on a `Parameter` value")
        }
    }

    /// Get the index of the statement where the register is defined.
    /// Return `None` if the register is not defined in the function body.
    pub fn body(&self) -> Option<&FunctionDefinitionIndex> {
        if let Self::Body(index) = self {
            Some(index)
        } else {
            None
        }
    }
}

/// [`RegisterUsage`] is about how a register is used.
pub struct RegisterUsage<'a> {
    content: &'a FunctionDefinition,
    /// Where the register is defined.
    pub define_position: RegisterDefinePosition,
    ///  Where the register is used, in order
    pub use_indexes: Vec<FunctionDefinitionIndex>,
}

impl<'a> RegisterUsage<'a> {
    /// If the register is defined by an alloca, return the type of the alloca.
    /// Panics if the register is not defined by an alloca.
    pub fn alloca_type(&self) -> data_type::Type {
        self.content[self.define_position.unwrap_body().clone()]
            .as_alloca()
            .alloc_type
            .clone()
    }

    /// Type of the register.
    pub fn data_type(&self) -> data_type::Type {
        match &self.define_position {
            RegisterDefinePosition::Body(define_index) => {
                self.content[define_index.clone()]
                    .generate_register()
                    .unwrap()
                    .1
            }
            RegisterDefinePosition::Parameter(parameter_index) => self.content.header.parameters
                [*parameter_index]
                .data_type
                .clone(),
        }
    }

    pub fn side_effect(&self) -> bool {
        if let RegisterDefinePosition::Body(position) = &self.define_position {
            if matches!(self.content[position.clone()], IRStatement::Call(_)) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// [`RegisterUsageAnalyzer`] is for analyzing how registers are used in a function.
pub struct RegisterUsageAnalyzer<'a> {
    content: &'a FunctionDefinition,
    register_usages: OnceCell<HashMap<RegisterName, RegisterUsage<'a>>>,
}

impl<'a> RegisterUsageAnalyzer<'a> {
    /// Create a new [`RegisterUsageAnalyzer`].
    pub fn new(content: &'a FunctionDefinition) -> Self {
        Self {
            content,
            register_usages: OnceCell::new(),
        }
    }

    /// All registers which are used in the function.
    pub fn registers(&self) -> Vec<&RegisterName> {
        // we want the result in order, so that we can make unit tests easier
        // Maybe use IndexMap for register_usages in the future
        let mut registers: Vec<_> = self.register_usages().keys().collect();
        registers.sort();
        registers
    }

    /// Get the [`RegisterUsage`] of `register`.
    pub fn get(&self, register: &RegisterName) -> &RegisterUsage {
        self.register_usages().get(register).unwrap()
    }

    /// Get all [`RegisterUsage`]s.    
    pub fn register_usages(&self) -> &HashMap<RegisterName, RegisterUsage> {
        self.register_usages.get_or_init(|| {
            let mut register_usages = HashMap::new();
            for (index, param) in self.content.header.parameters.iter().enumerate() {
                register_usages.insert(
                    param.name.clone(),
                    RegisterUsage {
                        content: self.content,
                        define_position: RegisterDefinePosition::Parameter(index),
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
                            define_position: RegisterDefinePosition::Body(index.clone()),
                            use_indexes: Vec::new(),
                        })
                        .define_position = RegisterDefinePosition::Body(index.clone());
                }
                for use_register_name in statement.use_register() {
                    register_usages
                        .entry(use_register_name)
                        .or_insert_with(|| RegisterUsage {
                            content: self.content,
                            define_position: RegisterDefinePosition::Body(index.clone()),
                            use_indexes: Vec::new(),
                        })
                        .use_indexes
                        .push(index.clone());
                }
            }
            register_usages
        })
    }

    /// Get the blocks `register` is active in.
    pub fn register_active_blocks(
        &self,
        register: &RegisterName,
        control_flow_graph: &ControlFlowGraph,
    ) -> Vec<usize> {
        let register_usages = &self.register_usages().get(register).unwrap();
        let mut use_blocks = register_usages
            .use_indexes
            .iter()
            .map(|it| it.0)
            .collect::<Vec<_>>();
        use_blocks.sort();
        use_blocks.dedup();
        let mut result = Vec::new();
        if let Some(define_block) = register_usages.define_position.body().map(|it| it.0) {
            if use_blocks.len() == 1 && use_blocks[0] == define_block {
                return vec![define_block];
            }
            result.push(define_block);

            for use_block in use_blocks {
                result.extend(
                    control_flow_graph
                        .may_pass_blocks(define_block, use_block)
                        .iter(),
                );
            }
        } else {
            for use_block in use_blocks {
                result.extend(control_flow_graph.may_pass_blocks(0, use_block).iter());
            }
        }
        result.sort();
        result.dedup();
        result
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::{
        ir::{
            self,
            function::{basic_block::BasicBlock, test_util::*},
            statement::Ret,
            FunctionDefinition,
        },
        utility::data_type::Type,
    };

    #[test]
    fn register_active_blocks() {
        let function_definition = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: Type::None,
            },
            content: vec![
                BasicBlock {
                    name: Some("bb0".to_string()),
                    content: vec![
                        binop_constant("m"),
                        binop_constant("n"),
                        binop_constant("u1"),
                        binop("i0", "m", "m"),
                        binop("j0", "n", "n"),
                        binop("a0", "u1", "u1"),
                        binop_constant("r"),
                        jump("bb1"),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        phi("i_bb1", "bb1", "i0", "bb4", "i2"),
                        phi("a_bb1", "bb1", "a0", "bb4", "a1"),
                        binop("i1", "i_bb1", "i_bb1"),
                        binop("j1", "j0", "j0"),
                        branch("bb2", "bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![
                        binop("u2", "a_bb1", "a_bb1"),
                        binop("a1", "u2", "i1"),
                        jump("bb3"),
                    ],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![
                        binop_constant("u3"),
                        binop("i2", "u3", "j1"),
                        branch("bb1", "bb4"),
                    ],
                },
                BasicBlock {
                    name: Some("bb4".to_string()),
                    content: vec![Ret {
                        value: Some(RegisterName("r".to_string()).into()),
                    }
                    .into()],
                },
            ],
        };
        let control_flow_graph = ControlFlowGraph::new(&function_definition);
        let analyzer = RegisterUsageAnalyzer::new(&function_definition);
        assert_eq!(
            analyzer.register_active_blocks(&RegisterName("m".to_string()), &control_flow_graph),
            vec![0],
        );
        assert_eq!(
            analyzer
                .register_active_blocks(&RegisterName("i_bb1".to_string()), &control_flow_graph),
            vec![1],
        );
        assert_eq!(
            analyzer.register_active_blocks(&RegisterName("i0".to_string()), &control_flow_graph),
            vec![0, 1]
        );
        assert_eq!(
            analyzer.register_active_blocks(&RegisterName("i2".to_string()), &control_flow_graph),
            vec![1, 3],
        );
        assert_eq!(
            analyzer.register_active_blocks(&RegisterName("a1".to_string()), &control_flow_graph),
            vec![1, 2, 3],
        );
        assert_eq!(
            analyzer.register_active_blocks(&RegisterName("j1".to_string()), &control_flow_graph),
            vec![1, 2, 3],
        );
        assert_eq!(
            analyzer.register_active_blocks(&RegisterName("r".to_string()), &control_flow_graph),
            vec![0, 1, 2, 3, 4]
        );
    }
}
