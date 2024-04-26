use std::collections::HashMap;

use itertools::Itertools;
use wasm_encoder::{Function, Instruction, ValType};

use crate::{
    ir::{
        function::basic_block::BasicBlock,
        quantity::Quantity,
        statement::{
            calculate::{binary, unary},
            IRStatement,
        },
        FunctionHeader, RegisterName,
    },
    utility::data_type::{Integer, Type},
};

use super::control_flow::ControlFlowElement;
fn lower_type(t: &Type) -> ValType {
    match t {
        crate::utility::data_type::Type::Integer(Integer { signed, width }) => {
            match (signed, width) {
                (true, 32) => ValType::I32,
                (true, 64) => ValType::I64,
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}

pub fn lower_function_type(header: &FunctionHeader) -> (Vec<ValType>, Vec<ValType>) {
    let parameter_types = header
        .parameters
        .iter()
        .map(|p| &p.data_type)
        .map(lower_type)
        .collect();
    let return_type = if header.return_type == Type::None {
        vec![]
    } else {
        vec![lower_type(&header.return_type)]
    };
    (parameter_types, return_type)
}

fn put_value_onto_stack(
    value: &Quantity,
    register_name_id_map: &HashMap<RegisterName, u32>,
    result: &mut Function,
    data_type: ValType,
) {
    match value {
        crate::ir::quantity::Quantity::RegisterName(register) => {
            let register_id = register_name_id_map[register];
            result.instruction(&Instruction::LocalGet(register_id));
        }
        crate::ir::quantity::Quantity::NumberLiteral(n) => {
            match data_type {
                ValType::I32 => result.instruction(&Instruction::I32Const(*n as i32)),
                ValType::I64 => result.instruction(&Instruction::I64Const(*n)),
                _ => unimplemented!(),
            };
        }
        crate::ir::quantity::Quantity::GlobalVariableName(_) => unimplemented!(),
    }
}

fn lower_unary_calculate(
    result: &mut Function,
    register_name_id_map: &HashMap<RegisterName, u32>,
    unary_calculate: &unary::UnaryCalculate,
) {
    let data_type = lower_type(&unary_calculate.data_type);
    match data_type {
        ValType::I32 => result.instruction(&Instruction::I32Const(0)),
        ValType::I64 => result.instruction(&Instruction::I64Const(0)),
        _ => unimplemented!(),
    };
    put_value_onto_stack(
        &unary_calculate.operand,
        register_name_id_map,
        result,
        data_type,
    );
    match &unary_calculate.operation {
        unary::UnaryOperation::Neg => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Sub),
            ValType::I64 => result.instruction(&Instruction::I64Sub),
            _ => unimplemented!(),
        },
        unary::UnaryOperation::Not => unimplemented!(),
    };
    let result_register_id = register_name_id_map[&unary_calculate.to];
    result.instruction(&Instruction::LocalSet(result_register_id));
}

fn lower_binary_calculate(
    result: &mut Function,
    register_name_id_map: &HashMap<RegisterName, u32>,
    binary_calculate: &binary::BinaryCalculate,
) {
    let data_type = lower_type(&binary_calculate.data_type);
    put_value_onto_stack(
        &binary_calculate.operand1,
        register_name_id_map,
        result,
        data_type,
    );
    put_value_onto_stack(
        &binary_calculate.operand2,
        register_name_id_map,
        result,
        data_type,
    );
    match &binary_calculate.operation {
        binary::BinaryOperation::Add => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Add),
            ValType::I64 => result.instruction(&Instruction::I64Add),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::LessThan => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32LtS),
            ValType::I64 => result.instruction(&Instruction::I64LtS),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::LessOrEqualThan => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32LeS),
            ValType::I64 => result.instruction(&Instruction::I64LeS),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::GreaterThan => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32GtS),
            ValType::I64 => result.instruction(&Instruction::I64GtS),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::GreaterOrEqualThan => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32GeS),
            ValType::I64 => result.instruction(&Instruction::I64GeS),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::Equal => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Eq),
            ValType::I64 => result.instruction(&Instruction::I64Eq),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::NotEqual => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Ne),
            ValType::I64 => result.instruction(&Instruction::I64Ne),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::Sub => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Sub),
            ValType::I64 => result.instruction(&Instruction::I64Sub),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::Or => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Or),
            ValType::I64 => result.instruction(&Instruction::I64Or),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::Xor => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Xor),
            ValType::I64 => result.instruction(&Instruction::I64Xor),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::And => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32And),
            ValType::I64 => result.instruction(&Instruction::I64And),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::LogicalShiftLeft => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32Shl),
            ValType::I64 => result.instruction(&Instruction::I64Shl),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::LogicalShiftRight => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32ShrU),
            ValType::I64 => result.instruction(&Instruction::I64ShrU),
            _ => unimplemented!(),
        },
        binary::BinaryOperation::AthematicShiftRight => match data_type {
            ValType::I32 => result.instruction(&Instruction::I32ShrS),
            ValType::I64 => result.instruction(&Instruction::I64ShrS),
            _ => unimplemented!(),
        },
    };
    let result_register_id = register_name_id_map[&binary_calculate.to];
    result.instruction(&Instruction::LocalSet(result_register_id));
}

fn lower_statement(
    result: &mut Function,
    register_name_id_map: &HashMap<RegisterName, u32>,
    statement: &IRStatement,
) {
    match statement {
        IRStatement::UnaryCalculate(unary_calculate) => {
            lower_unary_calculate(result, register_name_id_map, unary_calculate)
        }
        IRStatement::BinaryCalculate(binary_calculate) => {
            lower_binary_calculate(result, register_name_id_map, binary_calculate)
        }
        IRStatement::Branch(_) => todo!(),
        IRStatement::Jump(_) => todo!(),
        IRStatement::Ret(_) => todo!(),

        IRStatement::Phi(_) => unimplemented!(),
        IRStatement::Alloca(_) => unimplemented!(),
        IRStatement::Call(_) => unimplemented!(),
        IRStatement::Load(_) => unimplemented!(),
        IRStatement::Store(_) => unimplemented!(),
        IRStatement::LoadField(_) => unimplemented!(),
        IRStatement::SetField(_) => unimplemented!(),
    }
}

fn lower_basic_block(result: &mut Function, block: &BasicBlock) {
    for statement in &block.content {}
}

fn lower_control_flow_element(
    result: &mut Function,
    body: &[BasicBlock],
    element: &ControlFlowElement,
) {
    match element {
        ControlFlowElement::Block { content } => todo!(),
        ControlFlowElement::If {
            condition,
            on_success,
            on_failure,
        } => todo!(),
        ControlFlowElement::Loop { content } => todo!(),
        ControlFlowElement::BasicBlock { id } => todo!(),
    }
}

pub fn lower_function_body(body: &[BasicBlock], control_flow: &[ControlFlowElement]) -> Function {
    let locals = body
        .iter()
        .flat_map(|block| block.created_registers())
        .collect_vec();
    let mut function = Function::new_with_locals_types(locals.iter().map(|(_, t)| lower_type(t)));
    for control_flow_element in control_flow {}
    function
}
