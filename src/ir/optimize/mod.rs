use std::{cell::RefCell, collections::HashMap, mem};

use enum_dispatch::enum_dispatch;

mod remove_unused_register;
pub use remove_unused_register::RemoveUnusedRegister;

use super::{
    function::{GenerateRegister, UseRegister},
    RegisterName,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StatementIndex {
    pub block_index: usize,
    pub statement_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegisterDefUseInfo {
    def: StatementIndex,
    uses: Vec<StatementIndex>,
}

impl RegisterDefUseInfo {
    pub fn new(def: StatementIndex) -> Self {
        Self {
            def,
            uses: Vec::new(),
        }
    }

    pub fn on_statement_remove(&mut self, index: &StatementIndex) {
        if self.def.block_index == index.block_index
            && index.statement_index > index.statement_index
        {
            self.def.statement_index -= 1;
        }
        for use_index in &mut self.uses {
            if use_index.block_index == index.block_index
                && index.statement_index > index.statement_index
            {
                use_index.statement_index -= 1;
            }
        }
    }
}

pub struct FunctionDefinitionInfo {
    pub register_def_use_info: Option<HashMap<RegisterName, RegisterDefUseInfo>>,
}

impl FunctionDefinitionInfo {
    pub fn new() -> Self {
        Self {
            register_def_use_info: None,
        }
    }

    pub fn on_statement_remove(&mut self, index: &StatementIndex) {
        if let Some(register_def_use_info) = &mut self.register_def_use_info {
            for (_, register_def_use) in register_def_use_info {
                register_def_use.on_statement_remove(index);
            }
        }
    }
}

pub struct Optimizer {
    ir: super::FunctionDefinition,
    passes: Vec<Passes>,
    function_definition_info: RefCell<FunctionDefinitionInfo>,
}

impl Optimizer {
    pub fn new(ir: super::FunctionDefinition) -> Self {
        Self {
            ir,
            passes: Vec::new(),
            function_definition_info: RefCell::new(FunctionDefinitionInfo::new()),
        }
    }

    pub fn add_pass(&mut self, pass: Passes) {
        self.passes.push(pass);
    }

    pub fn remove_statement(&mut self, index: &StatementIndex) {
        self.ir.content[index.block_index].remove(index.statement_index);
    }

    pub fn optimize(mut self) -> super::FunctionDefinition {
        let passes = mem::take(&mut self.passes);
        for pass in passes {
            pass.run(&mut self);
        }
        self.ir
    }

    pub fn register_used_at(&self, register: &RegisterName) -> Vec<StatementIndex> {
        self.function_definition_info
            .borrow_mut()
            .register_def_use_info
            .get_or_insert_with(|| {
                let mut result = HashMap::new();
                let mut unknown_source_register = HashMap::new();
                for (block_index, basic_block) in self.ir.content.iter().enumerate() {
                    for (statement_index, statement) in basic_block.iter().enumerate() {
                        let generated_register = statement.generated_register();
                        if let Some((register, _)) = generated_register {
                            result.insert(
                                register.clone(),
                                RegisterDefUseInfo {
                                    def: StatementIndex {
                                        block_index,
                                        statement_index,
                                    },
                                    uses: Vec::new(),
                                },
                            );
                            if let Some(already_use) = unknown_source_register.remove(&register) {
                                result.get_mut(&register).unwrap().uses = already_use;
                            }
                        }
                        let used_registers = statement.use_register();
                        for register in used_registers {
                            if let Some(register_def_use) = result.get_mut(&register) {
                                register_def_use.uses.push(StatementIndex {
                                    block_index,
                                    statement_index,
                                });
                            } else {
                                unknown_source_register
                                    .entry(register)
                                    .or_insert_with(Vec::new)
                                    .push(StatementIndex {
                                        block_index,
                                        statement_index,
                                    });
                            }
                        }
                    }
                }
                if !unknown_source_register.is_empty() {
                    panic!("unknown source register: {:?}", unknown_source_register);
                }
                result
            })
            .get(register)
            .map(|it| it.uses.clone())
            .unwrap_or_default()
    }
}

#[enum_dispatch]
pub trait Pass {
    fn run<'a>(&self, ir: &mut Optimizer);
}

#[enum_dispatch(Pass)]
pub enum Passes {
    RemoveUnusedRegister,
}
