+++
title = "Impl Bind Pattern"
description = "desc"
date = 2023-04-17T17:44:34.976Z
updated = 2023-04-17T17:44:34.976Z
draft = false
template = "blog/page.html"

[extra]
lead = "The Impl Bind Pattern allows us to create a new type that represents another type, but binds a parameter to all the methods in this implementation."


+++

For example:

```rust
pub struct ControlFlowGraph(/*...*/);

impl ControlFlowGraph {
    fn dominance_frontier(
        &self,
        content: &ir::FunctionDefinition,
        bb_index: usize,
    ) -> &[usize];

    fn basic_block_index_by_name(&self, content: &ir::FunctionDefinition, name: &str) -> usize;
    fn basic_block_name_by_index(
        &self,
        content: &ir::FunctionDefinition,
        index: usize,
    ) -> &str;
    fn may_pass_blocks(
        &self,
        content: &ir::FunctionDefinition,
        from: usize,
        to: usize,
    ) -> Ref<Vec<usize>>;
}

pub struct BindedControlFlowGraph<'item, 'bind: 'item> {
    bind_on: &'bind FunctionDefinition,
    item: &'item ControlFlowGraph,
}

impl<'item, 'bind: 'item> BindedControlFlowGraph<'item, 'bind> {
    pub fn dominance_frontier(&self, bb_index: usize) -> &[usize] {
        self.item.dominance_frontier(self.bind_on, bb_index)
    }
    pub fn basic_block_index_by_name(&self, name: &str) -> usize {
        self.item.basic_block_index_by_name(self.bind_on, name)
    }
    pub fn basic_block_name_by_index(&self, index: usize) -> &str {
        self.item.basic_block_name_by_index(self.bind_on, index)
    }
    pub fn may_pass_blocks(&self, from: usize, to: usize) -> Ref<Vec<usize>> {
        self.item.may_pass_blocks(self.bind_on, from, to)
    }
}
```

## Motivation

Use the example above, `ControlFlowGraph` is used to analyze the control flow of a function. It uses interior mutability to cache information about the control flow. We use `on_action` to update or invalidate the cache and use the other methods to query the information.

By letting the `BindedControlFlowGraph` store a reference to `FunctionDefinition` and using it instead of `ControlFlowGraph` in other places, we prevent the `FunctionDefinition` from being edited when the `BindedControlFlowGraph` is in scope and save a parameter passing when using.

