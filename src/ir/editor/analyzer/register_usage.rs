use std::{cell::OnceCell, collections::HashMap};

use crate::{
    ir::{
        self,
        editor::action::Action,
        function::FunctionDefinitionIndex,
        statement::{IRStatement, IsIRStatement},
        FunctionDefinition, RegisterName,
    },
    utility::data_type,
};

use super::{control_flow::BindedControlFlowGraph, IsAnalyzer};

/// [`RegisterDefinePosition`] is the position where a register is defined.
#[derive(Debug)]
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
#[derive(Debug)]
pub struct RegisterUsage {
    /// Where the register is defined.
    pub define_position: RegisterDefinePosition,
    ///  Where the register is used, in order
    pub use_indexes: Vec<FunctionDefinitionIndex>,
}

impl RegisterUsage {
    /// If the register is defined by an alloca, return the type of the alloca.
    /// Panics if the register is not defined by an alloca.
    pub fn alloca_type(&self, content: &ir::FunctionDefinition) -> data_type::Type {
        content[self.define_position.unwrap_body().clone()]
            .as_alloca()
            .alloc_type
            .clone()
    }

    /// Type of the register.
    pub fn data_type(&self, content: &ir::FunctionDefinition) -> data_type::Type {
        match &self.define_position {
            RegisterDefinePosition::Body(define_index) => {
                content[define_index.clone()].generate_register().unwrap().1
            }
            RegisterDefinePosition::Parameter(parameter_index) => content.header.parameters
                [*parameter_index]
                .data_type
                .clone(),
        }
    }

    pub fn side_effect(&self, content: &ir::FunctionDefinition) -> bool {
        if let RegisterDefinePosition::Body(position) = &self.define_position {
            matches!(content[position.clone()], IRStatement::Call(_))
        } else {
            false
        }
    }
}

pub struct BindedRegisterUsage<'item, 'bind: 'item> {
    bind_on: &'bind FunctionDefinition,
    item: &'item RegisterUsage,
}

impl<'item, 'bind: 'item> BindedRegisterUsage<'item, 'bind> {
    pub fn new(bind_on: &'bind FunctionDefinition, item: &'item RegisterUsage) -> Self {
        Self { bind_on, item }
    }

    pub fn alloca_type(&self) -> data_type::Type {
        self.item.alloca_type(self.bind_on)
    }

    pub fn data_type(&self) -> data_type::Type {
        self.item.data_type(self.bind_on)
    }

    pub fn side_effect(&self) -> bool {
        self.item.side_effect(self.bind_on)
    }
    pub fn use_indexes(&self) -> &[FunctionDefinitionIndex] {
        &self.item.use_indexes
    }
    pub fn define_position(&self) -> &RegisterDefinePosition {
        &self.item.define_position
    }
}

impl RegisterUsage {
    pub fn bind<'item, 'bind: 'item>(
        &'item self,
        bind_on: &'bind FunctionDefinition,
    ) -> BindedRegisterUsage<'item, 'bind> {
        BindedRegisterUsage::new(bind_on, self)
    }
}

/// [`RegisterUsageAnalyzer`] is for analyzing how registers are used in a function.
#[derive(Debug, Default)]
pub struct RegisterUsageAnalyzer {
    register_usages: OnceCell<HashMap<RegisterName, RegisterUsage>>,
}

impl RegisterUsageAnalyzer {
    /// Create a new [`RegisterUsageAnalyzer`].
    pub fn new() -> Self {
        Self {
            register_usages: OnceCell::new(),
        }
    }

    /// All registers which are used in the function.
    fn registers(&self, content: &ir::FunctionDefinition) -> Vec<&RegisterName> {
        // we want the result in order, so that we can make unit tests easier
        // Maybe use IndexMap for register_usages in the future
        let mut registers: Vec<_> = self.register_usages(content).keys().collect();
        registers.sort();
        registers
    }

    /// Get the [`RegisterUsage`] of `register`.
    fn get(&self, content: &ir::FunctionDefinition, register: &RegisterName) -> &RegisterUsage {
        self.register_usages(content).get(register).unwrap()
    }

    /// Get all [`RegisterUsage`]s.    
    fn register_usages(
        &self,
        content: &ir::FunctionDefinition,
    ) -> &HashMap<RegisterName, RegisterUsage> {
        self.register_usages.get_or_init(|| {
            let mut register_usages = HashMap::new();
            for (index, param) in content.header.parameters.iter().enumerate() {
                register_usages.insert(
                    param.name.clone(),
                    RegisterUsage {
                        define_position: RegisterDefinePosition::Parameter(index),
                        use_indexes: Vec::new(),
                    },
                );
            }
            for (index, statement) in content.iter().function_definition_index_enumerate() {
                if let Some((define_register_name, _)) = statement.generate_register() {
                    register_usages
                        .entry(define_register_name)
                        .or_insert_with(|| RegisterUsage {
                            define_position: RegisterDefinePosition::Body(index.clone()),
                            use_indexes: Vec::new(),
                        })
                        .define_position = RegisterDefinePosition::Body(index.clone());
                }
                for use_register_name in statement.use_register() {
                    register_usages
                        .entry(use_register_name)
                        .or_insert_with(|| RegisterUsage {
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
    fn register_active_blocks(
        &self,
        content: &ir::FunctionDefinition,
        register: &RegisterName,
        control_flow_graph: &BindedControlFlowGraph,
    ) -> Vec<usize> {
        let register_usages = &self.register_usages(content).get(register).unwrap();
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

pub struct BindedRegisterUsageAnalyzer<'item, 'bind: 'item> {
    bind_on: &'bind FunctionDefinition,
    item: &'item RegisterUsageAnalyzer,
}

impl<'item, 'bind: 'item> BindedRegisterUsageAnalyzer<'item, 'bind> {
    pub fn registers(&self) -> Vec<&RegisterName> {
        self.item.registers(self.bind_on)
    }

    pub fn get(&self, register: &RegisterName) -> BindedRegisterUsage<'item, 'bind> {
        BindedRegisterUsage::new(self.bind_on, self.item.get(self.bind_on, register))
    }

    pub fn register_usages(&self) -> HashMap<RegisterName, BindedRegisterUsage<'item, 'bind>> {
        self.item
            .register_usages(self.bind_on)
            .iter()
            .map(|(name, usage)| (name.clone(), BindedRegisterUsage::new(self.bind_on, usage)))
            .collect()
    }

    pub fn register_active_blocks(
        &self,
        register: &RegisterName,
        control_flow_graph: &BindedControlFlowGraph,
    ) -> Vec<usize> {
        self.item
            .register_active_blocks(self.bind_on, register, control_flow_graph)
    }
}

impl<'item, 'bind: 'item> IsAnalyzer<'item, 'bind> for RegisterUsageAnalyzer {
    fn on_action(&mut self, _action: &Action) {
        self.register_usages.take();
    }

    type Binded = BindedRegisterUsageAnalyzer<'item, 'bind>;

    fn bind(&'item self, content: &'bind ir::FunctionDefinition) -> Self::Binded {
        BindedRegisterUsageAnalyzer {
            bind_on: content,
            item: self,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::{
        ir::{
            self,
            analyzer::ControlFlowGraph,
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
        let control_flow_graph = ControlFlowGraph::new();
        let control_flow_graph = control_flow_graph.bind(&function_definition);
        let analyzer = RegisterUsageAnalyzer::new();
        assert_eq!(
            analyzer.register_active_blocks(
                &function_definition,
                &RegisterName("m".to_string()),
                &control_flow_graph
            ),
            vec![0],
        );
        assert_eq!(
            analyzer.register_active_blocks(
                &function_definition,
                &RegisterName("i_bb1".to_string()),
                &control_flow_graph
            ),
            vec![1],
        );
        assert_eq!(
            analyzer.register_active_blocks(
                &function_definition,
                &RegisterName("i0".to_string()),
                &control_flow_graph
            ),
            vec![0, 1]
        );
        assert_eq!(
            analyzer.register_active_blocks(
                &function_definition,
                &RegisterName("i2".to_string()),
                &control_flow_graph
            ),
            vec![1, 3],
        );
        assert_eq!(
            analyzer.register_active_blocks(
                &function_definition,
                &RegisterName("a1".to_string()),
                &control_flow_graph
            ),
            vec![1, 2, 3],
        );
        assert_eq!(
            analyzer.register_active_blocks(
                &function_definition,
                &RegisterName("j1".to_string()),
                &control_flow_graph
            ),
            vec![1, 2, 3],
        );
        assert_eq!(
            analyzer.register_active_blocks(
                &function_definition,
                &RegisterName("r".to_string()),
                &control_flow_graph
            ),
            vec![0, 1, 2, 3, 4]
        );
    }
}
