use std::collections::{HashSet, VecDeque};

use pass::{IsPass, Pass};

use super::{analyzer::Analyzer, function::formalize, IR};

/// Actions and action batch to edit ir function.
mod action;
/// Optimizing passes to be executed on a function.
pub mod pass;

/// [`FunctionOptimizer`] can manage passes and optimize the ir function.
#[derive(Default)]
pub struct FunctionOptimizer {
    passes: Vec<Pass>,
}

impl FunctionOptimizer {
    pub fn from_passes(passes: Vec<Pass>) -> Self {
        Self { passes }
    }

    /// Add a [`Pass`] to [`FunctionOptimizer`].
    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }

    /// Run all passes on the ir function.
    pub fn optimize(self, mut ir: super::FunctionDefinition) -> super::FunctionDefinition {
        ir = formalize(ir);
        let mut executed = HashSet::new();
        let mut current_control_flow_graph = None;
        let mut current_passes: VecDeque<_> = self.passes.iter().cloned().collect();
        while !current_passes.is_empty() {
            let next_pass = current_passes.front().unwrap();
            let mut first_updated = false;
            for require in next_pass.need() {
                if !executed.contains(&require) {
                    current_passes.push_front(require);
                    first_updated = true;
                }
            }
            if first_updated {
                continue;
            }
            let analyzer = if let Some(current_control_flow_graph) = current_control_flow_graph {
                Analyzer::reuse_control_flow_graph(&ir, current_control_flow_graph)
            } else {
                Analyzer::new(&ir)
            };
            let pass = current_passes.pop_front().unwrap();
            let edit_action_batch = pass.run(&analyzer);
            current_control_flow_graph = Some(analyzer.free());
            edit_action_batch.perform(&mut ir);
            executed.insert(pass);
        }
        ir
    }
}

pub fn optimize(ir: Vec<IR>, passes: Vec<Pass>) -> Vec<IR> {
    let mut result = Vec::new();
    for ir in ir {
        match ir {
            IR::FunctionDefinition(function_definition) => {
                let function_optimizer = FunctionOptimizer::from_passes(passes.clone());
                let optimized_function_definition =
                    function_optimizer.optimize(function_definition);
                result.push(IR::FunctionDefinition(optimized_function_definition));
            }
            ir => result.push(ir),
        }
    }
    result
}

#[cfg(test)]
mod test_util {
    use crate::ir::{function::formalize, FunctionDefinition};

    use super::*;

    pub fn execute_pass(ir: FunctionDefinition, pass: Pass) -> FunctionDefinition {
        let mut ir = formalize(ir);
        let analyzer = Analyzer::new(&ir);
        let edit_action_batch = pass.run(&analyzer);
        edit_action_batch.perform(&mut ir);
        ir
    }
}
