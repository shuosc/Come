use crate::ir::FunctionDefinition;

use self::{
    control_flow::ControlFlowGraph, memory_usage::MemoryUsageAnalyzer,
    register_usage::RegisterUsageAnalyzer,
};

pub mod control_flow;
pub mod memory_usage;
pub mod register_usage;
pub struct Analyzer<'a> {
    pub content: &'a FunctionDefinition,
    pub register_usage: RegisterUsageAnalyzer<'a>,
    pub memory_usage: MemoryUsageAnalyzer<'a>,
    pub control_flow_graph: ControlFlowGraph,
}

impl<'a> Analyzer<'a> {
    pub fn new(content: &'a FunctionDefinition) -> Self {
        Self {
            content,
            register_usage: RegisterUsageAnalyzer::new(content),
            memory_usage: MemoryUsageAnalyzer::new(content),
            control_flow_graph: ControlFlowGraph::new(content),
        }
    }

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

    pub fn free(self) -> ControlFlowGraph {
        self.control_flow_graph
    }
}
