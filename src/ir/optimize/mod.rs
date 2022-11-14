mod analyzer;
mod editor;
mod pass;
use editor::IRFunctionEditor;
use std::mem;

use pass::{IsPass, Pass};

pub struct Optimizor {
    passes: Vec<Pass>,
}

impl Optimizor {
    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }

    pub fn optimize(mut self, ir: super::FunctionDefinition) -> super::FunctionDefinition {
        let mut editor = IRFunctionEditor::new(ir);
        let passes = mem::take(&mut self.passes);
        for pass in passes {
            pass.run(&mut editor);
        }
        editor.done()
    }
}
