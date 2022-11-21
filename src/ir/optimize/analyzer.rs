use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    ops::Deref,
    rc::Rc,
};

use bimap::BiMap;
use itertools::Itertools;
use petgraph::{
    algo::dominators::{simple_fast, Dominators},
    prelude::DiGraph,
    visit::GraphBase,
};

use crate::{
    ir::{
        function::FunctionDefinitionIndex,
        quantity::Quantity,
        statement::{IRStatement, IsIRStatement},
        FunctionDefinition, RegisterName,
    },
    utility::{self, data_type},
};

#[derive(Debug, Clone)]
pub struct MemoryAccessInfo {
    pub alloca: FunctionDefinitionIndex,
    // store statements index, in order
    pub store: Vec<FunctionDefinitionIndex>,
    // load statements index, in order
    pub load: Vec<FunctionDefinitionIndex>,
}

impl MemoryAccessInfo {
    // For each store statement, this function will find all load statements in this basic block which
    // load the value it just stored.
    pub fn dorminate_in_basic_block(
        &self,
    ) -> HashMap<FunctionDefinitionIndex, Vec<FunctionDefinitionIndex>> {
        let mut result: HashMap<_, Vec<FunctionDefinitionIndex>> = HashMap::new();
        let mut store_iter = self.store.iter().peekable();
        let mut load_iter = self.load.iter().peekable();
        while let Some(store) = store_iter.next() {
            while let Some(&next_load) = load_iter.peek() && next_load < store {
                load_iter.next();
            }
            let end_index = if let Some(&next_store) = store_iter.peek() {
                let in_same_bb = store.0 == next_store.0;
                if in_same_bb {
                    Some(next_store)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(_end_index) = end_index {
                while let Some(&next_load) = load_iter.peek() && next_load < store {
                    result.entry(store.clone()).or_default().push(load_iter.next().unwrap().clone());
                }
            } else {
                while let Some(&next_load) = load_iter.peek() && next_load.0 == store.0 {
                    result.entry(store.clone()).or_default().push(load_iter.next().unwrap().clone());
                }
            }
        }
        result
    }

    // for any variable, only the last store in each basic block may affect other basic blocks
    pub fn stores_used_by_other_blocks(&self) -> Vec<FunctionDefinitionIndex> {
        self.store
            .iter()
            .group_by(|it| it.0)
            .into_iter()
            .map(|(_, it)| it.into_iter().last().unwrap())
            .cloned()
            .collect_vec()
    }
}

type DefaultNodeId = <DiGraph<usize, ()> as GraphBase>::NodeId;

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub graph: DiGraph<usize, ()>,
    pub dorminators: Dominators<DefaultNodeId>,
    pub frontiers: HashMap<DefaultNodeId, Vec<DefaultNodeId>>,
    pub bb_index_node_index_map: BiMap<usize, DefaultNodeId>,
    pub start_node: DefaultNodeId,
    pub end_node: DefaultNodeId,
    pub bb_name_index_map: HashMap<Option<String>, usize>,
}

impl ControlFlowGraph {
    pub fn new(function_definition: &FunctionDefinition) -> Self {
        let mut graph = DiGraph::<usize, ()>::new();
        let mut bb_index_node_index_map = BiMap::new();
        let start_node = graph.add_node(0);
        let mut bb_name_index_map = HashMap::new();
        let mut first_node = None;
        for (bb_index, bb) in function_definition.content.iter().enumerate() {
            let bb_node = graph.add_node(bb_index + 1);
            if first_node.is_none() {
                first_node = Some(bb_node);
            }
            bb_index_node_index_map.insert(bb_index, bb_node);
            bb_name_index_map.insert(bb.name.clone(), bb_index);
        }
        let end_node = graph.add_node(usize::MAX);
        graph.add_edge(start_node, first_node.unwrap(), ());
        for (bb_index, bb) in function_definition.content.iter().enumerate() {
            if let Some(last_statement) = bb.content.last() {
                let bb_node_index = bb_index_node_index_map.get_by_left(&bb_index).unwrap();
                match last_statement {
                    IRStatement::Branch(branch) => {
                        let success_node_index = *bb_index_node_index_map
                            .get_by_left(
                                bb_name_index_map
                                    .get(&Some(branch.success_label.clone()))
                                    .unwrap(),
                            )
                            .unwrap();
                        graph.add_edge(*bb_node_index, success_node_index, ());
                        let failure_node_index = *bb_name_index_map
                            .get(&Some(branch.failure_label.clone()))
                            .map(|bb_index| bb_index_node_index_map.get_by_left(bb_index).unwrap())
                            .unwrap();
                        graph.add_edge(*bb_node_index, failure_node_index, ());
                    }
                    IRStatement::Jump(jump) => {
                        let to_node_index = *bb_index_node_index_map
                            .get_by_left(bb_name_index_map.get(&Some(jump.label.clone())).unwrap())
                            .unwrap();
                        graph.add_edge(*bb_node_index, to_node_index, ());
                    }
                    IRStatement::Ret(_) => {
                        graph.add_edge(*bb_node_index, end_node, ());
                    }
                    _ => unreachable!(),
                }
            }
        }
        let dorminators = simple_fast(&graph, start_node);
        let frontiers = utility::graph::dominance_frontiers(&dorminators, &graph);
        Self {
            graph,
            dorminators,
            frontiers,
            bb_index_node_index_map,
            start_node,
            end_node,
            bb_name_index_map,
        }
    }

    pub fn dorminate_frontier(&self, bb_index: usize) -> Vec<usize> {
        let node = self.bb_index_node_index_map.get_by_left(&bb_index).unwrap();
        self.frontiers
            .get(node)
            .unwrap()
            .iter()
            .map(|node| *self.bb_index_node_index_map.get_by_right(node).unwrap())
            .collect()
    }
}

#[derive(Debug)]
pub struct Analyzer {
    pub content: Rc<RefCell<FunctionDefinition>>,
    pub definition_index: HashMap<RegisterName, FunctionDefinitionIndex>,
    pub use_indexes: HashMap<RegisterName, Vec<FunctionDefinitionIndex>>,
    pub memory_access: HashMap<RegisterName, MemoryAccessInfo>,
    control_flow_graph: RefCell<Option<ControlFlowGraph>>,
}

impl Analyzer {
    pub fn new(content: Rc<RefCell<FunctionDefinition>>) -> Self {
        Self {
            content,
            definition_index: HashMap::new(),
            use_indexes: HashMap::new(),
            memory_access: HashMap::new(),
            control_flow_graph: RefCell::new(None),
        }
    }

    pub fn on_statement_remove(&mut self, _index: &FunctionDefinitionIndex) {
        // todo: we should profile which is better: clear all the cache or update (at least part of) the cache
        self.definition_index.clear();
        self.use_indexes.clear();
        self.memory_access.clear();
    }

    pub fn on_statement_insert(&mut self) {
        // todo: we should profile which is better: clear all the cache or update (at least part of) the cache
        self.definition_index.clear();
        self.use_indexes.clear();
        self.memory_access.clear();
    }

    pub fn memory_access_info(&mut self) -> &HashMap<RegisterName, MemoryAccessInfo> {
        if self.memory_access.is_empty() {
            let content = self.content.borrow();
            for (index, statement) in content.iter().function_definition_index_enumerate() {
                if matches!(statement, IRStatement::Alloca(_)) {
                    self.memory_access
                        .entry(statement.generate_register().unwrap().0)
                        .or_insert_with(|| MemoryAccessInfo {
                            alloca: index,
                            store: Vec::new(),
                            load: Vec::new(),
                        })
                        .alloca = index.clone();
                } else if matches!(statement, IRStatement::Store(_)) {
                    if let Quantity::RegisterName(local) = &statement.as_store().target {
                        self.memory_access
                            .entry(local.clone())
                            .or_insert_with(|| MemoryAccessInfo {
                                // it's ok to use `index` as the index here, because it will definitly be updated later
                                alloca: index.clone(),
                                store: Vec::new(),
                                load: Vec::new(),
                            })
                            .store
                            .push(index);
                    }
                } else if matches!(statement, IRStatement::Load(_)) {
                    if let Quantity::RegisterName(local) = &statement.as_load().from {
                        self.memory_access
                            .entry(local.clone())
                            .or_insert_with(|| MemoryAccessInfo {
                                // it's ok to use `index` as the index here, because it will definitly be updated later
                                alloca: index.clone(),
                                store: Vec::new(),
                                load: Vec::new(),
                            })
                            .load
                            .push(index);
                    }
                }
            }
        }
        &self.memory_access
    }

    pub fn alloca_register_type(&mut self, name: &RegisterName) -> data_type::Type {
        let alloca_index = self.memory_access_info().get(name).unwrap().alloca.clone();
        let content = Rc::as_ref(&self.content).borrow();
        let alloca_statement = content[alloca_index].as_alloca();
        alloca_statement.alloc_type.clone()
    }

    pub fn use_indexes(&mut self, register: &RegisterName) -> &Vec<FunctionDefinitionIndex> {
        if self.use_indexes.is_empty() {
            for (index, statement) in self
                .content
                .borrow()
                .iter()
                .function_definition_index_enumerate()
            {
                let use_registers = statement.use_register();
                for register in use_registers {
                    self.use_indexes
                        .entry(register)
                        .or_insert_with(Vec::new)
                        .push(index.clone());
                }
            }
        }
        self.use_indexes.get(register).unwrap()
    }

    pub fn control_flow_graph(&self) -> Ref<ControlFlowGraph> {
        let mut control_flow_graph = self.control_flow_graph.borrow_mut();
        if control_flow_graph.is_none() {
            let function_definition = self.content.borrow();
            *control_flow_graph = Some(ControlFlowGraph::new(function_definition.deref()));
        }
        drop(control_flow_graph);
        Ref::map(self.control_flow_graph.borrow(), |it| it.as_ref().unwrap())
    }

    pub fn basic_block_index(&self, name: &Option<String>) -> usize {
        *self
            .control_flow_graph()
            .bb_name_index_map
            .get(name)
            .unwrap()
    }
}
