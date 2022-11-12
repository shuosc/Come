use std::collections::HashMap;

use petgraph::{prelude::*, stable_graph::DefaultIx};

use super::{function::UseRegister, statement::Terminator, RegisterName};

pub struct BasicBlockAnalyzer {}

pub struct FunctionAnalyzer {
    control_flow_diagram: DiGraph<usize, ()>,
    basic_block_index_to_node_index: HashMap<usize, NodeIndex<DefaultIx>>,
}

impl<'a> FunctionAnalyzer {
    pub fn new(ir: &super::FunctionDefinition) -> Self {
        let mut control_flow_diagram = DiGraph::new();
        let mut basic_block_index_to_node_index = HashMap::new();
        for i in 0..ir.content.len() {
            let node_index = control_flow_diagram.add_node(i);
            basic_block_index_to_node_index.insert(i, node_index);
        }
        for (index, bb) in ir.content.iter().enumerate() {
            if let Some(terminator) = &bb.terminator {
                match terminator {
                    Terminator::Branch(branch) => {
                        let success_index = ir
                            .content
                            .iter()
                            .position(|bb| bb.name == Some(branch.success_label.clone()))
                            .unwrap();
                        let success_node_index = basic_block_index_to_node_index[&success_index];
                        let failure_index = ir
                            .content
                            .iter()
                            .position(|bb| bb.name == Some(branch.failure_label.clone()))
                            .unwrap();
                        let failure_node_index = basic_block_index_to_node_index[&failure_index];
                        let current_bb_node_index = basic_block_index_to_node_index[&index];
                        control_flow_diagram.add_edge(
                            current_bb_node_index,
                            success_node_index,
                            (),
                        );
                        control_flow_diagram.add_edge(
                            current_bb_node_index,
                            failure_node_index,
                            (),
                        );
                    }
                    Terminator::Jump(jump) => {
                        let to_index = ir
                            .content
                            .iter()
                            .position(|bb| bb.name == Some(jump.label.clone()))
                            .unwrap();
                        let to_node_index = basic_block_index_to_node_index[&to_index];
                        let current_bb_node_index = basic_block_index_to_node_index[&index];
                        control_flow_diagram.add_edge(current_bb_node_index, to_node_index, ());
                    }
                    Terminator::Ret(_) => {}
                }
            }
        }
        Self {
            control_flow_diagram,
            basic_block_index_to_node_index,
        }
    }

    pub fn register_used_at(
        &self,
        ir: &super::FunctionDefinition,
        register: &RegisterName,
    ) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        for (bb_id, bb) in ir.content.iter().enumerate() {
            for (statement_id, statement) in bb.iter().enumerate() {
                if statement.use_register().contains(register) {
                    result.push((bb_id, statement_id));
                }
            }
        }
        result
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         ast::function_definition,
//         ir::{
//             quantity::Quantity,
//             statement::{branch::BranchType, Branch, Jump},
//             FunctionDefinition,
//         },
//         utility::data_type::Type,
//     };

//     use super::*;

//     #[test]
//     fn test_dorminate() {
//         let block1 = BasicBlock {
//             name: None,
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: Some(
//                 Branch {
//                     branch_type: BranchType::NE,
//                     operand1: 0.into(),
//                     operand2: 0.into(),
//                     success_label: "bb2".to_string(),
//                     failure_label: "bb3".to_string(),
//                 }
//                 .into(),
//             ),
//         };
//         let block2 = BasicBlock {
//             name: Some("bb2".to_string()),
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: Some(Jump {
//                 label: "bb4".to_string(),
//             }.into())
//         };
//         let block3 = BasicBlock {
//             name: Some("bb3".to_string()),
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: Some(Jump {
//                 label: "bb4".to_string(),
//             }.into())
//         };
//         let block4 = BasicBlock {
//             name: Some("bb4".to_string()),
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: Some(Branch {
//                 branch_type: BranchType::NE,
//                 operand1: 0.into(),
//                 operand2: 0.into(),
//                 success_label: "bb5".to_string(),
//                 failure_label: "bb6".to_string(),
//             }.into())
//         };
//         let block5 = BasicBlock {
//             name: Some("bb5".to_string()),
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: Some(Jump {
//                 label: "bb7".to_string(),
//             }.into())
//         };
//         let block6 = BasicBlock {
//             name: Some("bb6".to_string()),
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: Some(Jump {
//                 label: "bb7".to_string(),
//             }.into())
//         };
//         let block7 = BasicBlock {
//             name: Some("bb7".to_string()),
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: Some(Branch {
//                 branch_type: BranchType::NE,
//                 operand1: 0.into(),
//                 operand2: 0.into(),
//                 success_label: "bb4".to_string(),
//                 failure_label: "bb8".to_string(),
//             }.into())
//         };
//         let block8 = BasicBlock {
//             name: Some("bb8".to_string()),
//             phis: Vec::new(),
//             content: Vec::new(),
//             terminator: None
//         };

//         let function_definition = FunctionDefinition {
//             name: "f".to_string(),
//             parameters: Vec::new(),
//             return_type: Type::None,
//             content: vec![block1, block2, block3, block4, block5, block6, block7, block8],
//         };
//         let analyzer = FunctionAnalyzer::new(&function_definition);
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[0]));
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[1]));
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[2]));
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[3]));
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[4]));
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[5]));
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[6]));
//         assert!(analyzer.dorminate(&function_definition.content[0], &function_definition.content[7]));
//         assert!(!analyzer.dorminate(&function_definition.content[1], &function_definition.content[0]));
//         assert!(analyzer.dorminate(&function_definition.content[1], &function_definition.content[1]));
//         assert!(!analyzer.dorminate(&function_definition.content[1], &function_definition.content[2]));
//         assert!(!analyzer.dorminate(&function_definition.content[1], &function_definition.content[3]));
//     }
// }
