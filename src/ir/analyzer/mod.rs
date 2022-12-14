use crate::ir::FunctionDefinition;

use self::{
    control_flow::ControlFlowGraph, memory_usage::MemoryUsageAnalyzer,
    register_usage::RegisterUsageAnalyzer,
};

/// Contains control flow graph and related infomation of a function.
pub mod control_flow;
/// Contains memory usage analyzer of a function.
pub mod memory_usage;
/// Contains register usage analyzer of a function.
pub mod register_usage;

/// [`Analyzer`] is for gather infomation about a [`FunctionDefinition`].
pub struct Analyzer<'a> {
    pub content: &'a FunctionDefinition,
    pub register_usage: RegisterUsageAnalyzer<'a>,
    pub memory_usage: MemoryUsageAnalyzer<'a>,
    pub control_flow_graph: ControlFlowGraph,
}

impl<'a> Analyzer<'a> {
    /// Create a [`Analyzer`] from a [`FunctionDefinition`].
    pub fn new(content: &'a FunctionDefinition) -> Self {
        Self {
            content,
            register_usage: RegisterUsageAnalyzer::new(content),
            memory_usage: MemoryUsageAnalyzer::new(content),
            control_flow_graph: ControlFlowGraph::new(content),
        }
    }

    /// Create a [`Analyzer`] from a [`FunctionDefinition`], and reuse the [`ControlFlowGraph`] instead of construct it.
    pub fn reuse_control_flow_graph(
        content: &'a FunctionDefinition,
        control_flow_graph: ControlFlowGraph,
    ) -> Self {
        Self {
            content,
            register_usage: RegisterUsageAnalyzer::new(content),
            memory_usage: MemoryUsageAnalyzer::new(content),
            control_flow_graph,
        }
    }

    /// Free the [`Analyzer`] and return the [`ControlFlowGraph`] for possible reusings.
    pub fn free(self) -> ControlFlowGraph {
        self.control_flow_graph
    }
}
