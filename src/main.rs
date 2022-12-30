#![feature(once_cell)]
#![feature(let_chains)]
use clap::Parser;
use ezio::prelude::*;
use ir::optimize::optimize;
use std::{io::Write, path::PathBuf, str::FromStr};

use crate::ir::optimize::pass::Pass;

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
    // let args = Args::parse();
    let code = r#"fn main() -> () {
    let gpio_address: Address = 0x80002000;
    while 1 {
        let current_value: u32 = load_u32(gpio_address);
        if current_value == 0 {
            store_u32(gpio_address, 1);
        } else {
            store_u32(gpio_address, 0);
        }
    }
}"#;
    let from_source = ast::from_source(&code).unwrap();
    let ast = from_source.1;
    let ir = ir::from_ast(&ast);
    for ir in &ir {
        println!("{}", ir);
    }
    let passes = vec![
        Pass::from_str("RemoveOnlyOnceStore").unwrap(),
        Pass::from_str("RemoveLoadDirectlyAfterStore").unwrap(),
        Pass::from_str("RemoveUnusedRegister").unwrap(),
        Pass::from_str("MemoryToRegister").unwrap(),
        Pass::from_str("RemoveUnusedRegister").unwrap(),
    ];
    let ir = optimize(ir, passes);
    for ir in &ir {
        println!("{}", ir);
    }
    // if let Some(emit_ir_path) = args.emit_ir_path {
    //     let mut w = file::writer(emit_ir_path);
    //     for ir in ir.iter() {
    //         writeln!(w, "{}", ir).unwrap();
    //     }
    // }
    let code = backend::riscv::emit_code(&ir);
    println!("{}", code);
    // file::write(args.output, &code);
}
