use std::collections::{HashMap, VecDeque};

use enum_dispatch::enum_dispatch;
use itertools::Itertools;

use crate::{
    ir::{
        function::FunctionDefinitionIndex,
        quantity::Quantity,
        statement::{phi::PhiSource, IRStatement, IsIRStatement, Phi},
        FunctionDefinition, RegisterName,
    },
    utility::data_type::{self, Type},
};
mod insert_statement;
mod remove_statement;
mod rename_local;
pub use insert_statement::InsertStatement;
pub use remove_statement::RemoveStatement;
pub use rename_local::RenameLocal;
// /// A phi entry to be inserted to the function definition.
// #[derive(Debug, PartialEq, Eq)]
// pub struct PhiEntry {
//     /// The block index of the phi entry should be inserted.
//     pub block: usize,
//     /// The variable name of the phi entry.
//     pub variable_name: String,
//     /// The block index the control flow comes from.
//     pub source: usize,
//     /// The quantity of the phi entry.
//     pub value: Quantity,
// }

// /// [`EditActionBatch`] represents a batch of edit actions created by a [`super::Pass`].
// #[derive(Debug, Default)]
// pub struct EditActionBatch {
//     /// These statements needs to be removed.
//     pub remove: Vec<FunctionDefinitionIndex>,
//     /// These [`PhiEntry`]s needs to be inserted.
//     pub insert_phis: Vec<PhiEntry>,
//     /// These registers should be replaced.
//     pub replace_register: Vec<(RegisterName, Quantity)>,
// }

// impl EditActionBatch {
//     /// Add a remove action to the batch.
//     pub fn remove(&mut self, index: impl Into<FunctionDefinitionIndex>) {
//         self.remove.push(index.into());
//     }

//     /// Add an insert phi action to the batch.
//     pub fn insert_phi(
//         &mut self,
//         to_basic_block_index: usize,
//         variable_name: String,
//         value_from_basic_block: usize,
//         value: Quantity,
//     ) {
//         self.insert_phis.push(PhiEntry {
//             block: to_basic_block_index,
//             variable_name,
//             source: value_from_basic_block,
//             value,
//         });
//     }

//     /// Add a replace register action to the batch.
//     pub fn replace(&mut self, from: RegisterName, to: Quantity) {
//         self.replace_register.push((from, to));
//     }

//     /// Merge two [`EditActionBatch`]s.
//     pub fn merge(mut self, other: Self) -> Self {
//         let Self {
//             remove,
//             insert_phis,
//             replace_register,
//         } = other;
//         self.remove.extend(remove);
//         self.insert_phis.extend(insert_phis);
//         self.replace_register.extend(replace_register);
//         self
//     }

//     fn push_front_statement(
//         function: &mut FunctionDefinition,
//         basic_block_index: usize,
//         statement: impl Into<IRStatement>,
//     ) {
//         function[basic_block_index]
//             .content
//             .insert(0, statement.into());
//     }

//     fn remove_statement(function: &mut FunctionDefinition, index: &FunctionDefinitionIndex) {
//         function.content[index.0].remove(index.1);
//     }

//     fn replace_register(
//         function: &mut FunctionDefinition,
//         register: &RegisterName,
//         value: Quantity,
//     ) {
//         function.iter_mut().for_each(|statement| {
//             statement.on_register_change(register, value.clone());
//         });
//     }

//     fn generate_phi_node(
//         function: &FunctionDefinition,
//         variable_name: &str,
//         variable_type: data_type::Type,
//         to_be_putted_block_index: usize,
//         from: impl Iterator<Item = (String, Quantity)>,
//     ) -> Phi {
//         let sources = from.map(|(from, value)| PhiSource {
//             name: value,
//             block: from,
//         });
//         let to_be_putted_block_name = function[to_be_putted_block_index].name.as_ref().unwrap();
//         let mut from = sources.collect_vec();
//         from.sort();
//         from.dedup();
//         Phi {
//             to: RegisterName(format!("{variable_name}_{to_be_putted_block_name}")),
//             data_type: variable_type,
//             from,
//         }
//     }

//     pub fn execute(
//         self,
//         mut function: FunctionDefinition,
//         variable_and_types: &HashMap<RegisterName, Type>,
//     ) -> FunctionDefinition {
//         let EditActionBatch {
//             mut remove,
//             replace_register,
//             mut insert_phis,
//         } = self;
//         // Remove statements in reverse order.
//         remove.sort();
//         remove.dedup();
//         for index_to_remove in remove.iter().rev() {
//             Self::remove_statement(&mut function, index_to_remove);
//         }
//         // First we group phi entries by (variable name, block index), so that wen can generate phi nodes.
//         // Then insert the phi nodes into the function.
//         insert_phis.sort_by(|a, b| (&a.variable_name, a.block).cmp(&(&b.variable_name, b.block)));
//         insert_phis
//             .into_iter()
//             .group_by(|it| (it.variable_name.clone(), it.block))
//             .into_iter()
//             .for_each(|((variable_name, block), group)| {
//                 let source =
//                     group.map(|entry| (function[entry.source].name.clone().unwrap(), entry.value));
//                 let phi_node = Self::generate_phi_node(
//                     &function,
//                     &variable_name,
//                     variable_and_types
//                         .get(&RegisterName(variable_name.clone()))
//                         .unwrap()
//                         .clone(),
//                     block,
//                     source,
//                 );
//                 Self::push_front_statement(&mut function, block, phi_node);
//             });
//         // Replace registers.
//         for (register_name, value) in replace_register {
//             Self::replace_register(&mut function, &register_name, value);
//         }
//         function
//     }
// }

#[enum_dispatch]
pub trait IsAction {
    fn perform(self, ir: &mut FunctionDefinition);
    fn affect_others<'a>(&self, others: impl Iterator<Item = &'a mut Action>);
}

#[enum_dispatch(IsAction)]
pub enum Action {
    InsertStatement,
    RemoveStatement,
    RenameLocal,
}

#[derive(Default)]
pub struct Actions(VecDeque<Action>);

impl Actions {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn push(&mut self, action: impl Into<Action>) {
        self.0.push_back(action.into());
    }
    pub fn merge(mut self, mut other: Self) -> Self {
        self.0.append(&mut other.0);
        self
    }
    pub fn perform(self, ir: &mut FunctionDefinition) {
        let mut actions = self.0;
        while let Some(action) = actions.pop_front() {
            action.affect_others(actions.iter_mut());
            action.perform(ir);
        }
    }
}

impl FromIterator<Action> for Actions {
    fn from_iter<T: IntoIterator<Item = Action>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
