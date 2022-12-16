#![feature(once_cell)]
#![feature(let_chains)]
use clap::Parser;
use ezio::prelude::*;
use ir::optimize::optimize;
use std::{io::Write, path::PathBuf};

/// Definitions of AST nodes and their parser.
mod ast;
/// Functions for generating assembly code from ir.
mod backend;
/// Definitions of IR nodes and their parser, and ir generator functions for generating ir from ast.
mod ir;
/// Utilities shared among modules.
mod utility;

/// Come language compiler.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file path.
    #[arg(short, long)]
    input: PathBuf,

    /// Output file path.
    #[arg(short, long)]
    output: PathBuf,

    /// IR file path, won't generate ir file if empty.
    #[arg(short = None, long = "emit-ir")]
    emit_ir_path: Option<PathBuf>,

    #[arg(short = 'O', long, value_delimiter = ',')]
    optimize: Vec<ir::optimize::pass::Pass>,
}

fn main() {
    let args = Args::parse();
    let code = file::read(args.input);
    let ast = ast::from_source(&code).unwrap().1;
    let ir = ir::from_ast(&ast);
    let ir = optimize(ir, args.optimize);
    if let Some(emit_ir_path) = args.emit_ir_path {
        let mut w = file::writer(emit_ir_path);
        for ir in ir.iter() {
            writeln!(w, "{}", ir).unwrap();
        }
    }
    let code = backend::riscv::emit_code(&ir);
    file::write(args.output, &code);
}
