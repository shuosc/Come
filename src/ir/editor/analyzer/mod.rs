pub use self::{
    control_flow::ControlFlowGraph, memory_usage::MemoryUsage,
    register_usage::RegisterUsageAnalyzer as RegisterUsage,
};
use super::action::Action;

mod control_flow;
mod memory_usage;
pub mod register_usage;

pub trait IsAnalyzer {
    fn on_action(&mut self, action: &Action);
}

/// [`Analyzer`] is for gather infomation about a [`FunctionDefinition`].
#[derive(Default, Debug)]
pub struct Analyzer {
    pub register_usage: RegisterUsage,
    pub memory_usage: MemoryUsage,
    pub control_flow_graph: ControlFlowGraph,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            register_usage: RegisterUsage::new(),
            memory_usage: MemoryUsage::new(),
            control_flow_graph: ControlFlowGraph::new(),
        }
    }
}

impl IsAnalyzer for Analyzer {
    fn on_action(&mut self, action: &Action) {
        self.register_usage.on_action(action);
        self.memory_usage.on_action(action);
        self.control_flow_graph.on_action(action);
    }
}
