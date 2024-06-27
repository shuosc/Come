use super::IsPass;
use crate::ir::{editor, editor::analyzer::register_usage::RegisterDefinePosition};
use serde::{Deserialize, Serialize};
/// This pass will remove the register which are defined but not used.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct RemoveUnusedRegister;

impl IsPass for RemoveUnusedRegister {
    fn run(&self, editor: &mut editor::Editor) {
        let mut to_remove = Vec::new();
        for usage in editor
            .binded_analyzer()
            .register_usage()
            .register_usages()
            .values()
        {
            if !usage.side_effect() && usage.use_indexes().is_empty() {
                if let RegisterDefinePosition::Body(define_index) = &usage.define_position() {
                    to_remove.push(define_index.clone());
                }
            }
        }
        editor.remove_statements(to_remove);
    }
    fn need(&self) -> Vec<super::Pass> {
        Vec::new()
    }
    fn invalidate(&self) -> Vec<super::Pass> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::borrow_interior_mutable_const)]

    use super::*;
    use crate::{
        ir::{
            self,
            function::basic_block::BasicBlock,
            statement::{
                branch::BranchType, calculate::binary::BinaryOperation, BinaryCalculate, Branch,
                Jump, Load, Ret,
            },
            FunctionDefinition, RegisterName,
        },
        utility::data_type::{self, Type},
    };

    #[test]
    fn run() {
        let function = FunctionDefinition {
            header: ir::FunctionHeader {
                name: "f".to_string(),
                parameters: Vec::new(),
                return_type: Type::None,
            },
            content: vec![
                BasicBlock {
                    name: None,
                    content: vec![
                        Load {
                            to: RegisterName("0".to_string()),
                            data_type: data_type::I32.clone(),
                            from: 0.into(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("1".to_string()),
                            data_type: data_type::I32.clone(),
                            from: 4.into(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("3".to_string()),
                            data_type: data_type::I32.clone(),
                            from: 4.into(),
                        }
                        .into(),
                        Jump {
                            label: "bb1".to_string(),
                        }
                        .into(),
                    ],
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    content: vec![
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("0".to_string()).into(),
                            operand2: RegisterName("1".to_string()).into(),
                            to: RegisterName("2".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                        Jump {
                            label: "bb2".to_string(),
                        }
                        .into(),
                    ],
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    content: vec![Branch {
                        branch_type: BranchType::NE,
                        operand1: RegisterName("2".to_string()).into(),
                        operand2: 0.into(),
                        success_label: "bb1".to_string(),
                        failure_label: "bb3".to_string(),
                    }
                    .into()],
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    content: vec![Ret {
                        value: Some(RegisterName("2".to_string()).into()),
                    }
                    .into()],
                },
            ],
        };
        let mut editor = editor::Editor::new(function);
        RemoveUnusedRegister.run(&mut editor);
        assert_eq!(editor.content[0].content.len(), 3);
    }
}
