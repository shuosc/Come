use pass::{IsPass, Pass};

use super::analyzer::Analyzer;

/// Actions and action batch to edit ir function.
mod action;
/// Optimizing passes to be executed on a function.
mod pass;

/// [`Optimizer`] can manage passes and optimize the ir function.
#[derive(Default)]
pub struct Optimizor {
    passes: Vec<Pass>,
}

impl Optimizor {
    /// Add a [`Pass`] to [`Optimizer`].
    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }

    /// Run all passes on the ir function.
    pub fn optimize(self, mut ir: super::FunctionDefinition) -> super::FunctionDefinition {
        // todo: make it a special `formalize` function
        //       except fill name for the first block,
        //       we should also fill the jump statement for blocks
        //       which don't have a terminator
        let mut current_control_flow_graph = None;
        for pass in self.passes {
            let analyzer = if let Some(current_control_flow_graph) = current_control_flow_graph {
                Analyzer::reuse_control_flow_graph(&ir, current_control_flow_graph)
            } else {
                Analyzer::new(&ir)
            };
            let edit_action_batch = pass.run(&analyzer);
            let variable_and_types = analyzer.memory_usage.memory_access_variables_and_types();
            current_control_flow_graph = Some(analyzer.free());
            ir = edit_action_batch.execute(ir, &variable_and_types);
        }
        ir
    }
}

#[cfg(test)]
mod test_util {
    use crate::ir::FunctionDefinition;

    use super::*;

    pub fn execute_pass(mut ir: FunctionDefinition, pass: Pass) -> FunctionDefinition {
        // todo: make it a special `formalize` function
        //       except fill name for the first block,
        //       we should also fill the jump statement for blocks
        //       which don't have a terminator
        if ir.content[0].name.is_none() {
            ir.content[0].name = Some(format!("{}_entry", ir.name));
        }

        let analyzer = Analyzer::new(&ir);
        let edit_action_batch = pass.run(&analyzer);
        let variable_and_types = analyzer.memory_usage.memory_access_variables_and_types();
        edit_action_batch.execute(ir, &variable_and_types)
    }
}
