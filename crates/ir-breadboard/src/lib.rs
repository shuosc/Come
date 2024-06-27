use std::str::FromStr;

use come::ir::{
    self,
    analyzer::{self, control_flow::structural::FoldedCFG, ControlFlowGraph, IsAnalyzer},
    optimize::{optimize as optimize_ir, pass::Pass},
    IR,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(code: &str) -> JsValue {
    let (_, parsed_ir) = ir::parse(code).unwrap();
    let result = parsed_ir.as_function_definition();
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn optimize(code: &str, pass: &str) -> String {
    let ir_code = ir::parse(code).unwrap().1;
    let pass = Pass::from_str(pass).unwrap();
    let result = optimize_ir(vec![ir_code], vec![pass])
        .into_iter()
        .next()
        .unwrap();
    format!("{result}")
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Edge {
    pub from: u32,
    pub to: u32,
    pub back: bool,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Debug)]
pub struct CFGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<Edge>,
}

#[wasm_bindgen]
pub fn dump_control_flow_graph(code: &str) -> CFGraph {
    let ir_code = ir::parse(code).unwrap().1;
    if let IR::FunctionDefinition(f) = ir_code {
        let cfg = analyzer::ControlFlowGraph::new();
        let cfg = cfg.bind(&f);
        let backedges = cfg.back_edges();
        let mut result = CFGraph::default();
        let g = cfg.graph();
        for n in g.node_indices() {
            if n.index() == g.node_count() - 1 {
                result.nodes.push("_dummy_end".to_string());
            } else {
                result
                    .nodes
                    .push(cfg.basic_block_name_by_index(n.index()).to_string());
            }
        }
        for e in g.edge_indices() {
            let (from, to) = g.edge_endpoints(e).unwrap();
            let is_backedge = backedges.contains(&(from.index() as _, to.index() as _));
            result.edges.push(Edge {
                from: from.index() as _,
                to: to.index() as _,
                back: is_backedge,
            });
        }
        result
    } else {
        panic!("faq")
    }
}

#[wasm_bindgen]
pub fn structural_analysis(code: &str) -> JsValue {
    let ir_code = ir::parse(code).unwrap().1;
    let f = ir_code.as_function_definition();
    let cfg = ControlFlowGraph::new();
    let cfg = cfg.bind(f);
    let folded = FoldedCFG::from_control_flow_graph(&cfg);
    let result = folded.structural_analysis(&cfg);
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[test]
fn test_optimize() {
    dbg!(optimize(
        r"fn main() -> () {
      %0 = add i32 1, 2
      ret
    }",
        "FixIrreducible"
    ));
}

#[test]
fn test_dump_cfg() {
    dbg!(dump_control_flow_graph(
        r"fn test_condition(i32 %a, i32 %b) -> i32 {
          test_condition_entry:
            %a_0_addr = alloca i32
            store i32 %a, address %a_0_addr
            %b_0_addr = alloca i32
            store i32 %b, address %b_0_addr
            %result_0_addr = alloca i32
            store i32 0, address %result_0_addr
            %i_0_addr = alloca i32
            %0 = load i32 %a_0_addr
            store i32 %0, address %i_0_addr
            j loop_0_condition
          loop_0_condition:
            %2 = load i32 %i_0_addr
            %3 = load i32 %b_0_addr
            %1 = slt i32 %2, %3
            bne %1, 0, loop_0_success, loop_0_fail
          loop_0_success:
            %5 = load i32 %result_0_addr
            %6 = load i32 %i_0_addr
            %4 = add i32 %5, %6
            store i32 %4, address %result_0_addr
            %8 = load i32 %i_0_addr
            %7 = add i32 %8, 1
            store i32 %7, address %i_0_addr
            j loop_0_condition
          loop_0_fail:
            %9 = load i32 %result_0_addr
            ret %9
        }"
    ));
}

#[test]
fn test_structural_analysis() {
    let code = r"fn test_condition(i32 %a, i32 %b) -> i32 {
          test_condition_entry:
            %a_0_addr = alloca i32
            store i32 %a, address %a_0_addr
            %b_0_addr = alloca i32
            store i32 %b, address %b_0_addr
            %1 = load i32 %a_0_addr
            %2 = load i32 %b_0_addr
            %0 = slt i32 %1, %2
            bne %0, 0, if_0_success, if_0_fail
          if_0_success:
            %3 = load i32 %a_0_addr
            ret %3
          if_0_fail:
            %4 = load i32 %b_0_addr
            ret %4
        }";
    let ir_code = ir::parse(code).unwrap().1;
    let f = ir_code.as_function_definition();
    let cfg = ControlFlowGraph::new();
    let cfg = cfg.bind(f);
    let folded = FoldedCFG::from_control_flow_graph(&cfg);
    let result = folded.structural_analysis(&cfg);
    dbg!(result);
}
