use crate::ir::function::GenerateRegister;

use super::Pass;

pub struct RemoveUnusedRegister;

impl Pass for RemoveUnusedRegister {
    fn run<'a>(&self, optimizer: &mut super::Optimizer) {
        let mut to_remove = Vec::new();
        for (block_index, block) in optimizer.ir.content.iter().enumerate() {
            for (statement_index, statement) in block.iter().enumerate() {
                let generated_register = statement.generated_register();
                if let Some((register, _)) = generated_register {
                    let used_place = optimizer.register_used_at(&register);
                    if used_place.is_empty() {
                        to_remove.push((block_index, statement_index));
                    }
                }
            }
        }
        for (block_id, statement_id) in to_remove.iter().rev() {
            optimizer.ir.content[*block_id].remove(*statement_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{
            function::basic_block::BasicBlock,
            optimize::Optimizer,
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
            name: "f".to_string(),
            parameters: Vec::new(),
            return_type: Type::None,
            content: vec![
                BasicBlock {
                    name: None,
                    phis: Vec::new(),
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
                    ],
                    terminator: Some(
                        Jump {
                            label: "bb1".to_string(),
                        }
                        .into(),
                    ),
                },
                BasicBlock {
                    name: Some("bb1".to_string()),
                    phis: Vec::new(),
                    content: vec![BinaryCalculate {
                        operation: BinaryOperation::Add,
                        operand1: RegisterName("0".to_string()).into(),
                        operand2: RegisterName("1".to_string()).into(),
                        to: RegisterName("2".to_string()),
                        data_type: data_type::I32.clone(),
                    }
                    .into()],
                    terminator: Some(
                        Jump {
                            label: "bb2".to_string(),
                        }
                        .into(),
                    ),
                },
                BasicBlock {
                    name: Some("bb2".to_string()),
                    phis: Vec::new(),
                    content: Vec::new(),
                    terminator: Some(
                        Branch {
                            branch_type: BranchType::NE,
                            operand1: RegisterName("2".to_string()).into(),
                            operand2: 0.into(),
                            success_label: "bb1".to_string(),
                            failure_label: "bb3".to_string(),
                        }
                        .into(),
                    ),
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    phis: Vec::new(),
                    content: Vec::new(),
                    terminator: Some(
                        Ret {
                            value: Some(RegisterName("2".to_string()).into()),
                        }
                        .into(),
                    ),
                },
            ],
        };
        let pass = RemoveUnusedRegister;
        let mut optimizer = Optimizer::new(function);
        optimizer.add_pass(pass.into());
        let function = optimizer.optimize();
        assert_eq!(function.content[0].content.len(), 2);
    }
}
