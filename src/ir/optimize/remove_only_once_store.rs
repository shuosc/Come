use crate::ir::{
    function::HasRegister,
    statement::{ContentStatement, StatementRef},
};

use super::{Pass, StatementIndex};

pub struct RemoveOnlyOnceStore;

impl Pass for RemoveOnlyOnceStore {
    fn run<'a>(&self, optimizer: &mut super::Optimizer) {
        let mut to_remove = Vec::new();
        // (store_index, load_indexes)
        let mut to_edit = Vec::new();
        for alloca_index in optimizer.allocas().iter() {
            let alloca = optimizer.index(alloca_index);
            let alloca = if let StatementRef::Content(ContentStatement::Alloca(alloca)) = alloca {
                alloca
            } else {
                unreachable!()
            };
            let alloca_register = alloca.to.clone();
            let mut stores = optimizer.stores(&alloca_register);
            if stores.len() == 1 {
                let loads = optimizer.loads(&alloca_register);
                to_edit.push((stores.pop().unwrap(), loads));
                to_remove.push(alloca_index.clone());
            }
        }
        for (store_index, load_indexes) in to_edit {
            let store = if let StatementRef::Content(ContentStatement::Store(store)) =
                optimizer.index(&store_index)
            {
                store
            } else {
                unreachable!()
            };
            let store_value = store.source.clone();
            for load_index in load_indexes {
                let load = if let StatementRef::Content(ContentStatement::Load(load)) =
                    optimizer.index(&load_index)
                {
                    load
                } else {
                    unreachable!()
                };
                let to = load.to.clone();
                for (bb_index, bb) in &mut optimizer.ir.content.iter_mut().enumerate() {
                    for statement_index in 0..bb.len() {
                        if load_index
                            != (StatementIndex {
                                block_index: bb_index,
                                statement_index,
                            })
                        {
                            let mut statement = bb.index_mut(statement_index);
                            statement.on_register_change(&to, &store_value);
                        }
                    }
                }
                to_remove.push(load_index);
            }
            to_remove.push(store_index);
        }
        to_remove.sort();
        for to_remove_index in to_remove.iter().rev() {
            optimizer.remove_statement(to_remove_index);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        ir::{
            function::{basic_block::BasicBlock, GenerateRegister, UseRegister},
            optimize::Optimizer,
            statement::{
                calculate::binary::BinaryOperation, Alloca, BinaryCalculate, Jump, Load, Ret, Store,
            },
            FunctionDefinition, RegisterName,
        },
        utility::data_type::{self, Type},
    };

    use super::RemoveOnlyOnceStore;

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
                        Alloca {
                            to: RegisterName("0".to_string()),
                            alloc_type: data_type::I32.clone(),
                        }
                        .into(),
                        Store {
                            data_type: data_type::I32.clone(),
                            source: 42.into(),
                            target: RegisterName("0".to_string()).into(),
                        }
                        .into(),
                        Alloca {
                            to: RegisterName("1".to_string()),
                            alloc_type: data_type::I32.clone(),
                        }
                        .into(),
                        Store {
                            data_type: data_type::I32.clone(),
                            source: 42.into(),
                            target: RegisterName("1".to_string()).into(),
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
                    content: vec![
                        Load {
                            to: RegisterName("2".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("0".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("2".to_string()).into(),
                            operand2: RegisterName("2".to_string()).into(),
                            to: RegisterName("3".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("4".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("4".to_string()).into(),
                            operand2: RegisterName("2".to_string()).into(),
                            to: RegisterName("5".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                    ],
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
                    content: vec![
                        Store {
                            data_type: data_type::I32.clone(),
                            source: 43.into(),
                            target: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        Load {
                            to: RegisterName("6".to_string()),
                            data_type: data_type::I32.clone(),
                            from: RegisterName("1".to_string()).into(),
                        }
                        .into(),
                        BinaryCalculate {
                            operation: BinaryOperation::Add,
                            operand1: RegisterName("6".to_string()).into(),
                            operand2: RegisterName("6".to_string()).into(),
                            to: RegisterName("7".to_string()),
                            data_type: data_type::I32.clone(),
                        }
                        .into(),
                    ],
                    terminator: None,
                },
                BasicBlock {
                    name: Some("bb3".to_string()),
                    phis: Vec::new(),
                    content: Vec::new(),
                    terminator: Some(
                        Ret {
                            value: Some(RegisterName("7".to_string()).into()),
                        }
                        .into(),
                    ),
                },
            ],
        };
        let pass = RemoveOnlyOnceStore;
        let mut optimizer = Optimizer::new(function);
        optimizer.add_pass(pass.into());
        let function = optimizer.optimize();
        // %0 and %2 should be optimized out
        let mut registers = HashSet::new();
        for bbs in function.content {
            for bb in bbs.iter() {
                registers.extend(bb.use_register().into_iter());
                if let Some((r, _)) = bb.generated_register() {
                    registers.insert(r);
                }
            }
        }
        assert!(!registers.contains(&RegisterName("0".to_string())));
        assert!(registers.contains(&RegisterName("1".to_string())));
        assert!(!registers.contains(&RegisterName("2".to_string())));
        assert!(registers.contains(&RegisterName("3".to_string())));
        assert!(registers.contains(&RegisterName("4".to_string())));
        assert!(registers.contains(&RegisterName("5".to_string())));
        assert!(registers.contains(&RegisterName("6".to_string())));
        assert!(registers.contains(&RegisterName("7".to_string())));
    }
}
