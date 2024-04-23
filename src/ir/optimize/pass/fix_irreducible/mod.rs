use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
};

use itertools::Itertools;

use crate::{
    ir::{
        analyzer::BindedControlFlowGraph,
        editor::Editor,
        function::basic_block::BasicBlock,
        optimize::pass::{remove_unused_register::RemoveUnusedRegister, TopologicalSort},
        quantity::Quantity,
        statement::{branch::BranchType, phi::PhiSource, Branch, IRStatement, Jump, Phi},
        FunctionDefinition, RegisterName,
    },
    utility::data_type::{Integer, Type},
};

use super::{IsPass, Pass};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct FixIrreducible;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum IntoSccEdgeSource {
    /// Jump or branch into one single block
    /// Can be jump directly into the scc
    /// Or branch into the scc in **one single** condition
    One(usize),
    /// Branch into two blocks in same Scc
    Two {
        source_block_index: usize,
        on_success: usize,
        on_failure: usize,
    },
}

impl IntoSccEdgeSource {
    fn source(&self) -> usize {
        match self {
            IntoSccEdgeSource::One(source_block_index) => *source_block_index,
            IntoSccEdgeSource::Two {
                source_block_index, ..
            } => *source_block_index,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum FixOtherBlockPlan {
    DirectReplace { block: usize, origin_target: String },
    ExtractCondition { block: usize, inverse: bool },
}

#[derive(Debug, Clone)]
struct EditPlan {
    scc_id: String,
    phis: Vec<Phi>,
    branches: Vec<Branch>,
    fix_other_block_plan: HashSet<FixOtherBlockPlan>,
}

fn phi_target(origin_target: &str, scc_id: &str) -> RegisterName {
    RegisterName(format!("_should_goto_scc_{}_{}", scc_id, origin_target))
}

fn extracted_condition(in_block: &str, scc_id: &str) -> RegisterName {
    RegisterName(format!(
        "_extracted_branch_condition_scc_{}_at_{}",
        scc_id, in_block
    ))
}

fn guard_block_name(origin_target: &str, scc_id: &str) -> String {
    format!("_guard_block_scc_{}_for_{}", scc_id, origin_target)
}

fn generate_edit_plan(
    origin_target_to_source_map: &HashMap<usize, Vec<IntoSccEdgeSource>>,
    control_flow_graph: &BindedControlFlowGraph,
) -> EditPlan {
    let origin_targets = origin_target_to_source_map
        .keys()
        .copied()
        .sorted()
        .collect_vec();
    let scc_id = origin_targets.iter().join("_");
    // We order all the source blocks by their index
    let all_sources: BTreeSet<usize> = origin_target_to_source_map
        .values()
        .flatten()
        .map(|it| it.source())
        .collect();
    // origin target's index -> the phi node which represents whether we should jump to this target
    let mut phis = HashMap::new();
    // first, we construct all the Phi statements
    for &origin_target in &origin_targets {
        // todo: we construct an invalid Phi node (all value = 0) here and fill the content back later
        //   MAYBE try avoid this by using a `PhiBuilder`.
        let origin_target_name = control_flow_graph.basic_block_name_by_index(origin_target);
        let phi = Phi {
            to: phi_target(origin_target_name, &scc_id),
            data_type: Type::Integer(Integer {
                signed: false,
                width: 1,
            }),
            // all phi nodes are interested in all the sources
            // we keep them all 0, means "should not branch to that target" for now
            from: all_sources
                .iter()
                .map(|it| PhiSource {
                    value: Quantity::NumberLiteral(0),
                    block: control_flow_graph
                        .basic_block_name_by_index(*it)
                        .to_string(),
                })
                .collect(),
        };
        phis.insert(origin_target, phi);
    }
    // then we consider all sources, and:
    //   - Generate fix other block plan
    //   - Fill the Phi node
    let mut fix_other_block_plan = HashSet::new();
    for (origin_target, sources) in origin_target_to_source_map {
        for source in sources.iter() {
            let source_block_name = control_flow_graph.basic_block_name_by_index(source.source());
            // get the `value` field for the Phi node, and generate fix other block plan
            let value = match source {
                IntoSccEdgeSource::One(source_block_index) => {
                    // in this case, we can directly replace the jump/branch target of the last statement
                    // of the source block
                    fix_other_block_plan.insert(FixOtherBlockPlan::DirectReplace {
                        block: *source_block_index,
                        origin_target: control_flow_graph
                            .basic_block_name_by_index(*origin_target)
                            .to_string(),
                    });
                    // and we should jump to the origin target whatever happens
                    Quantity::NumberLiteral(1)
                }
                IntoSccEdgeSource::Two {
                    on_success,
                    on_failure,
                    source_block_index,
                } => {
                    // in this case, we need to extract the source block's branch condition

                    // Whether we need to inverse the condition depends on whether the order of the
                    // [on_success, on_failure] is the same as the order of the branch targets.
                    // For making these easier, we keep the branch target in ascending order
                    // thus, if on_success >= on_failure, we need to inverse the condition
                    fix_other_block_plan.insert(FixOtherBlockPlan::ExtractCondition {
                        block: *source_block_index,
                        inverse: on_success >= on_failure,
                    });
                    // and we should jump to the left target if the extracted condition is true
                    Quantity::RegisterName(extracted_condition(source_block_name, &scc_id))
                }
            };
            // update the Phi node
            phis.get_mut(origin_target).unwrap().from[all_sources
                .iter()
                .position(|&it| it == source.source())
                .unwrap()] = PhiSource {
                value,
                block: source_block_name.to_string(),
            };
        }
    }
    // we don't need origin target's index part of the phis anymore
    let mut phis = phis
        .into_iter()
        .sorted_by(|(a, _), (b, _)| a.cmp(b))
        .map(|it| it.1)
        .collect_vec();
    // last, we generate all the branches

    // the last branch to is special, we don't need to generate a guard block for it
    // so we save it here for later use
    let last_branch_to = *origin_targets.last().unwrap();
    // Generate all the branches, include the last one
    let mut branches = phis
        .iter()
        .zip(origin_targets.into_iter())
        .tuple_windows()
        .map(
            |((depend_on_phi_result, target_block_index), (_, next_target_block_index))| Branch {
                branch_type: BranchType::NE,
                operand1: Quantity::RegisterName(depend_on_phi_result.to.clone()),
                operand2: Quantity::NumberLiteral(0),
                success_label: control_flow_graph
                    .basic_block_name_by_index(target_block_index)
                    .to_string(),
                failure_label: guard_block_name(
                    control_flow_graph.basic_block_name_by_index(next_target_block_index),
                    &scc_id,
                ),
            },
        )
        .collect_vec();
    // we don't generate a guard block for the last branch,
    // instead, we directly jump to the last target
    // so the last branch's failure label needs to be updated
    branches.last_mut().unwrap().failure_label = control_flow_graph
        .basic_block_name_by_index(last_branch_to)
        .to_string();
    // and the last phi is also useless
    phis.pop();
    EditPlan {
        phis,
        branches,
        fix_other_block_plan,
        scc_id,
    }
}

fn fix_other_block(
    function: &mut FunctionDefinition,
    first_guard_block_name: &str,
    scc_id: &str,
    plan: FixOtherBlockPlan,
) {
    match plan {
        FixOtherBlockPlan::DirectReplace {
            block,
            origin_target,
        } => {
            let block = &mut function.content[block];
            let last_stmt = block.content.last_mut().unwrap();
            match last_stmt {
                IRStatement::Jump(jump) => {
                    jump.label = first_guard_block_name.to_string();
                }
                IRStatement::Branch(branch) => {
                    if branch.success_label == origin_target {
                        branch.success_label = first_guard_block_name.to_string();
                    }
                    if branch.failure_label == origin_target {
                        branch.failure_label = first_guard_block_name.to_string();
                    }
                }
                _ => unreachable!(),
            }
        }
        FixOtherBlockPlan::ExtractCondition { block, inverse } => {
            let block = &mut function.content[block];
            let block_name = block.name.clone().unwrap();
            let last_stmt = block.content.pop().unwrap();
            if let IRStatement::Branch(branch) = last_stmt {
                let extracted_register_name = extracted_condition(&block_name, scc_id);
                let mut condition = branch.extract_condition(extracted_register_name);
                if inverse {
                    condition.operation = condition.operation.inverse().unwrap();
                }
                block.content.push(IRStatement::BinaryCalculate(condition));
                block.content.push(IRStatement::Jump(Jump {
                    label: first_guard_block_name.to_string(),
                }));
            } else {
                unreachable!()
            }
        }
    }
}

fn execute_edit_plan(function: &mut FunctionDefinition, plan: EditPlan) {
    // first, we create all necessary blocks
    let mut guard_blocks = plan
        .branches
        .iter()
        .map(|it| {
            let name = guard_block_name(&it.success_label, &plan.scc_id);
            BasicBlock::new(name)
        })
        .collect_vec();
    // insert all the phis into the first guard block
    guard_blocks[0].content = plan.phis.into_iter().map(IRStatement::Phi).collect();
    // insert all the branches into the guard blocks
    for (guard_block, branch) in guard_blocks.iter_mut().zip(plan.branches) {
        guard_block.content.push(IRStatement::Branch(branch));
    }
    // edit the original function
    let first_guard_block_name = guard_blocks[0].name.clone().unwrap();
    for fix_plan in plan.fix_other_block_plan {
        fix_other_block(function, &first_guard_block_name, &plan.scc_id, fix_plan);
    }
    function.content.extend(guard_blocks.into_iter());
}

fn generate_origin_target_to_source_map(
    function_definition: &FunctionDefinition,
    mut edges_into_entry_nodes: Vec<(usize, usize)>,
) -> HashMap<usize, Vec<IntoSccEdgeSource>> {
    // first we want to know which "into scc edge sources" should be IntoSccEdgeSource::Two
    // to do this, we edges_into_entry_nodes sort by first source,
    // if there are two consequent edges_into_entry_node with same .0,
    // they belong to the IntoSccEdgeSource::Two category
    edges_into_entry_nodes.sort();
    let mut two_nodes = Vec::new();
    let mut one_nodes = Vec::new();
    while !edges_into_entry_nodes.is_empty() {
        let last = edges_into_entry_nodes.pop().unwrap();
        if let Some(last_but_one) = edges_into_entry_nodes.pop() {
            if last_but_one.0 == last.0 {
                two_nodes.push((last, last_but_one));
            } else {
                one_nodes.push(last);
                edges_into_entry_nodes.push(last_but_one);
            }
        } else {
            one_nodes.push(last);
        }
    }
    let mut result: HashMap<usize, Vec<IntoSccEdgeSource>> = HashMap::new();
    for ((in_block, target1), (_, target2)) in two_nodes {
        let target1_is_success = &function_definition[in_block]
            .content
            .last()
            .unwrap()
            .as_branch()
            .success_label
            == function_definition[target1].name.as_ref().unwrap();
        let on_success = if target1_is_success { target1 } else { target2 };
        let on_failure = if target1_is_success { target2 } else { target1 };
        let into_scc_edge_source = IntoSccEdgeSource::Two {
            source_block_index: in_block,
            on_success,
            on_failure,
        };
        result
            .entry(target1)
            .or_default()
            .push(into_scc_edge_source.clone());
        result
            .entry(target2)
            .or_default()
            .push(into_scc_edge_source);
    }
    for (in_block, target) in one_nodes {
        result
            .entry(target)
            .or_default()
            .push(IntoSccEdgeSource::One(in_block));
    }
    result
}

impl IsPass for FixIrreducible {
    fn run(&self, editor: &mut Editor) {
        while let Some(irreducible_scc) = editor
            .binded_analyzer()
            .control_flow_graph()
            .top_level_scc()
            .first_irreducible_sub_scc()
        {
            let analyzer = editor.binded_analyzer();
            let graph = analyzer.control_flow_graph();
            let edges_into_entry_nodes = irreducible_scc.edges_into_entry_nodes();
            let origin_target_to_source_map =
                generate_origin_target_to_source_map(&editor.content, edges_into_entry_nodes);
            let edit_plan = generate_edit_plan(&origin_target_to_source_map, &graph);
            drop(graph);
            drop(irreducible_scc);
            // fixme: don't use direct_edit for performance's sake!
            editor.direct_edit(move |f| {
                execute_edit_plan(f, edit_plan);
            });
        }
    }

    /// Which passes this pass requires to be executed before it.
    fn need(&self) -> Vec<Pass> {
        Vec::new()
    }

    /// Which passes this pass will invalidate.
    fn invalidate(&self) -> Vec<Pass> {
        vec![TopologicalSort.into(), RemoveUnusedRegister.into()]
    }
}

#[cfg(test)]
mod tests;
