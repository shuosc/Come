use std::collections::HashSet;

use self::pass::IsPass;

use super::{editor::Editor, IR};

/// Optimizing passes to be executed on a function.
pub mod pass;
use pass::Pass;
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
    pub fn optimize(mut self, ir: super::FunctionDefinition) -> super::FunctionDefinition {
        let mut editor = Editor::new(ir);
        let mut executed = HashSet::new();
        self.passes.reverse();
        while !self.passes.is_empty() {
            let next_pass = self.passes.last().unwrap();
            let mut first_updated = false;
            for require in next_pass.need() {
                if !executed.contains(&require) {
                    self.passes.push(require);
                    first_updated = true;
                }
            }
            if first_updated {
                continue;
            }
            let pass = self.passes.pop().unwrap();
            pass.run(&mut editor);
            executed.insert(pass);
        }
        editor.content
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
