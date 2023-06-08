use crate::ir::{self, FunctionDefinition};

use self::register_usage::RegisterUsageAnalyzer;
pub use self::{
    control_flow::{BindedControlFlowGraph, ControlFlowGraph, Loop, LoopContent},
    memory_usage::{BindedMemoryUsage, MemoryUsage},
    register_usage::{BindedRegisterUsage, BindedRegisterUsageAnalyzer},
};
use super::action::Action;

mod control_flow;
mod memory_usage;
pub mod register_usage;

pub trait IsAnalyzer<'item, 'bind: 'item> {
    type Binded;
    fn on_action(&'item mut self, action: &Action);
    fn bind(&'item self, content: &'bind ir::FunctionDefinition) -> Self::Binded;
}

/// [`Analyzer`] is for gather information about a [`FunctionDefinition`].
#[derive(Default, Debug)]
pub struct Analyzer {
    pub register_usage: RegisterUsageAnalyzer,
    pub memory_usage: MemoryUsage,
    pub control_flow_graph: ControlFlowGraph,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            register_usage: RegisterUsageAnalyzer::new(),
            memory_usage: MemoryUsage::new(),
            control_flow_graph: ControlFlowGraph::new(),
        }
    }
}
pub struct BindedAnalyzer<'item, 'bind: 'item> {
    pub bind_on: &'bind FunctionDefinition,
    item: &'item Analyzer,
}

impl<'item, 'bind: 'item> BindedAnalyzer<'item, 'bind> {
    pub fn register_usage(&self) -> BindedRegisterUsageAnalyzer {
        self.item.register_usage.bind(self.bind_on)
    }
    pub fn memory_usage(&self) -> BindedMemoryUsage {
        self.item.memory_usage.bind(self.bind_on)
    }
    pub fn control_flow_graph(&self) -> BindedControlFlowGraph {
        self.item.control_flow_graph.bind(self.bind_on)
    }
}

impl<'item, 'bind: 'item> IsAnalyzer<'item, 'bind> for Analyzer {
    fn on_action(&mut self, action: &Action) {
        self.register_usage.on_action(action);
        self.memory_usage.on_action(action);
        self.control_flow_graph.on_action(action);
    }

    type Binded = BindedAnalyzer<'item, 'bind>;

    fn bind(&'item self, content: &'bind ir::FunctionDefinition) -> Self::Binded {
        BindedAnalyzer {
            bind_on: content,
            item: self,
        }
    }
}
