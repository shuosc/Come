use std::{collections::HashMap};

use either::Either;

use crate::ir::{
    self,
    function::HasRegister,
    quantity::{LocalOrGlobal, LocalOrNumberLiteral},
    statements::{branch::BranchType, Jump, Terminator},
    IRStatement,
};
pub struct FunctionCompileContext {
    // local -> Either<reg, stack_offset>
    local_assign: HashMap<ir::Local, Either<String, usize>>,
}

pub fn emit_function_code(function: &ir::FunctionDefinition) -> String {
    let mut ctx = FunctionCompileContext {
        local_assign: HashMap::new(),
    };
    let ir::FunctionDefinition {
        name,
        parameters,
        content,
        ..
    } = function;
    let mut result = format!("{}:\n", name);

    // assign registers
    // params
    for (i, ir::function::Parameter { name, data_type: _ }) in parameters.iter().enumerate() {
        ctx.local_assign
            .insert(name.clone(), Either::Left(format!("a{}", i)));
    }

    // other
    let mut stack_space = 0;
    let mut current_assigned = 0;
    for statement in content.iter().flat_map(|it| &it.content) {
        if let IRStatement::Alloca(ir::statements::Alloca { to, .. }) = statement {
            ctx.local_assign
                .insert(to.clone(), Either::Right(stack_space));
            stack_space += 4;
        } else {
            for logic in statement.get_registers() {
                if !ctx.local_assign.contains_key(&logic) {
                    ctx.local_assign.insert(
                        logic.clone(),
                        Either::Left(format!("t{}", current_assigned)),
                    );
                    current_assigned += 1;
                }
            }
        }
    }

    result.push_str(format!("addi sp, sp, -{}\n", stack_space).as_str());

    for basic_block in content {
        let bb_code = emit_basic_block_code(basic_block, &mut ctx);
        result.push_str(&bb_code);
    }
    result
}

fn emit_basic_block_code(basic_block: &ir::BasicBlock, ctx: &mut FunctionCompileContext) -> String {
    let ir::BasicBlock {
        name,
        phis: _,
        content,
        terminator,
    } = basic_block;
    let mut result = String::new();
    if let Some(name) = name {
        result.push_str(&format!("{}:", name));
    }
    // todo: cannot, though not necessary to handle phi for now
    for statement in content {
        let statement_code = emit_statement_code(statement, ctx);
        result.push_str(&statement_code);
    }
    if let Some(terminator) = terminator {
        let terminator_code = emit_terminator_code(terminator, ctx);
        result.push_str(&terminator_code);
    }
    result
}

fn emit_terminator_code(terminator: &Terminator, ctx: &mut FunctionCompileContext) -> String {
    match terminator {
        Terminator::Ret(ir::statements::Ret { value }) => {
            let mut result = String::new();
            if let Some(LocalOrNumberLiteral::Local(local)) = value {
                let register = ctx.local_assign.get(local).unwrap().clone().unwrap_left();
                result += format!("mv a0, {}\n", register).as_str();
            } else if let Some(LocalOrNumberLiteral::NumberLiteral(n)) = value {
                result += format!("li a0, {}\n", n).as_str();
            }
            result += "ret";
            result
        }
        Terminator::Branch(branch) => emit_branch_code(branch, ctx),
        Terminator::Jump(Jump { label }) => format!("j {}", label),
    }
}

fn emit_branch_code(branch: &ir::statements::Branch, ctx: &mut FunctionCompileContext) -> String {
    let ir::statements::Branch {
        branch_type,
        operand1,
        operand2,
        success_label,
        failure_label,
    } = branch;
    let operand1 = if let LocalOrNumberLiteral::Local(local) = operand1 {
        ctx.local_assign.get(local).unwrap().clone().unwrap_left()
    } else {
        todo!()
    };
    let operand2 = if let LocalOrNumberLiteral::Local(local) = operand2 {
        ctx.local_assign.get(local).unwrap().clone().unwrap_left()
    } else {
        todo!()
    };
    let op = match branch_type {
        BranchType::EQ => "beq",
        BranchType::NE => "bne",
        BranchType::LT => "blt",
        BranchType::GE => "bge",
    };
    format!(
        "{} {}, {}, {}\nj {}\n",
        op, operand1, operand2, success_label, failure_label
    )
}

fn emit_statement_code(statement: &ir::IRStatement, ctx: &mut FunctionCompileContext) -> String {
    match statement {
        IRStatement::Alloca(_) => String::new(),
        IRStatement::UnaryCalculate(unary_calculate) => emit_unary_calculate(unary_calculate, ctx),
        IRStatement::BinaryCalculate(binary_calculate) => {
            emit_binary_calculate(binary_calculate, ctx)
        }
        IRStatement::Load(load) => emit_load(load, ctx),
        IRStatement::Store(store) => emit_store(store, ctx),
        IRStatement::LoadField(_) => todo!(),
        IRStatement::SetField(_) => todo!(),
    }
}

fn emit_store(store: &ir::statements::Store, ctx: &mut FunctionCompileContext) -> String {
    let ir::statements::Store { source, target, .. } = store;
    match (source, target) {
        (LocalOrNumberLiteral::Local(source), LocalOrGlobal::Local(target)) => {
            let from_reg = ctx.local_assign.get(source).unwrap().clone().unwrap_left();
            let to = ctx
                .local_assign
                .get(target)
                .unwrap()
                .clone()
                .unwrap_right();
            format!("sw {}, -{}(sp)\n", from_reg, to)
        }
        (LocalOrNumberLiteral::Local(_), LocalOrGlobal::Global(_)) => unimplemented!(),
        (LocalOrNumberLiteral::NumberLiteral(n), LocalOrGlobal::Local(target)) => {
            let to = ctx
                .local_assign
                .get(target)
                .unwrap()
                .clone()
                .unwrap_right();
            format!("li t6, {}\nsw t6, -{}(sp)\n", n, to)
        }
        (LocalOrNumberLiteral::NumberLiteral(_), LocalOrGlobal::Global(_)) => unimplemented!(),
    }
}

fn emit_load(load: &ir::statements::Load, ctx: &mut FunctionCompileContext) -> String {
    let ir::statements::Load { from, to, .. } = load;
    if let LocalOrGlobal::Local(from) = from {
        // todo: handle right
        if let Either::Right(addr) = ctx.local_assign.get(from).unwrap() {
            format!(
                "lw {}, -{}(sp)\n",
                ctx.local_assign.get(to).unwrap().clone().unwrap_left(),
                addr
            )
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
}

fn emit_unary_calculate(
    statement: &ir::statements::UnaryCalculate,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::statements::UnaryCalculate {
        operation,
        operand,
        to,
        data_type: _,
    } = statement;
    let to_reg = ctx.local_assign.get(to).unwrap().as_ref().left().unwrap();
    match (operation, operand) {
        (ir::statements::calculate::UnaryOperation::Neg, LocalOrNumberLiteral::Local(l)) => {
            format!("neg {}, {}\n", to_reg, l.0)
        }
        (
            ir::statements::calculate::UnaryOperation::Neg,
            LocalOrNumberLiteral::NumberLiteral(n),
        ) => {
            format!("li {}, {}\n", to_reg, -n)
        }
        (ir::statements::calculate::UnaryOperation::Not, LocalOrNumberLiteral::Local(l)) => {
            format!("not {}, {}\n", to_reg, l.0)
        }
        (
            ir::statements::calculate::UnaryOperation::Not,
            LocalOrNumberLiteral::NumberLiteral(n),
        ) => {
            format!("li {}, {}\n", to_reg, !n)
        }
    }
}

fn emit_binary_calculate(
    statement: &ir::statements::BinaryCalculate,
    ctx: &mut FunctionCompileContext,
) -> String {
    let ir::statements::BinaryCalculate {
        operation,
        operand1,
        operand2,
        to,
        data_type: _,
    } = statement;
    let to_reg = ctx.local_assign.get(to).unwrap().as_ref().left().unwrap();
    match (operation, operand1, operand2) {
        (
            ir::statements::calculate::BinaryOperation::Add,
            LocalOrNumberLiteral::Local(lhs),
            LocalOrNumberLiteral::Local(rhs),
        ) => {
            let lhs = ctx.local_assign.get(lhs).unwrap().clone().unwrap_left();
            let rhs = ctx.local_assign.get(rhs).unwrap().clone().unwrap_left();
            format!("add {}, {}, {}\n", to_reg, lhs, rhs)
        }
        (
            ir::statements::calculate::BinaryOperation::Add,
            LocalOrNumberLiteral::Local(lhs),
            LocalOrNumberLiteral::NumberLiteral(n),
        ) => {
            let lhs = ctx.local_assign.get(lhs).unwrap().clone().unwrap_left();
            format!("addi {}, {}, {}\n", to_reg, lhs, n)
        }
        (
            ir::statements::calculate::BinaryOperation::Add,
            LocalOrNumberLiteral::NumberLiteral(n),
            LocalOrNumberLiteral::Local(rhs),
        ) => {
            let rhs = ctx.local_assign.get(rhs).unwrap().clone().unwrap_left();
            format!("addi {}, {}, {}\n", to_reg, rhs, n)
        }
        (
            ir::statements::calculate::BinaryOperation::Add,
            LocalOrNumberLiteral::NumberLiteral(n),
            LocalOrNumberLiteral::NumberLiteral(m),
        ) => {
            format!("li {}, {}\n", to_reg, n + m)
        }
        (
            ir::statements::calculate::BinaryOperation::Sub,
            LocalOrNumberLiteral::Local(lhs),
            LocalOrNumberLiteral::Local(rhs),
        ) => {
            let lhs = ctx.local_assign.get(lhs).unwrap().clone().unwrap_left();
            let rhs = ctx.local_assign.get(rhs).unwrap().clone().unwrap_left();
            format!("sub {}, {}, {}\n", to_reg, lhs, rhs)
        }
        (
            ir::statements::calculate::BinaryOperation::Sub,
            LocalOrNumberLiteral::Local(lhs),
            LocalOrNumberLiteral::NumberLiteral(n),
        ) => {
            let lhs = ctx.local_assign.get(lhs).unwrap().clone().unwrap_left();
            format!("addi {}, {}, {}\n", to_reg, lhs, -n)
        }
        (
            ir::statements::calculate::BinaryOperation::Sub,
            LocalOrNumberLiteral::NumberLiteral(n),
            LocalOrNumberLiteral::Local(rhs),
        ) => {
            // todo: is it correct in edge cases?
            let rhs = ctx.local_assign.get(rhs).unwrap().clone().unwrap_left();
            format!(
                "sub {}, {}, {}\nneg {}, {}\n",
                to_reg, rhs, n, to_reg, to_reg
            )
        }
        (
            ir::statements::calculate::BinaryOperation::Sub,
            LocalOrNumberLiteral::NumberLiteral(n),
            LocalOrNumberLiteral::NumberLiteral(m),
        ) => {
            format!("li {}, {}\n", to_reg, n - m)
        }
        _ => unimplemented!(),
    }
}
