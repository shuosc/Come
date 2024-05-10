use std::{fs::File, path::PathBuf};

use clap::{Parser, ValueEnum};
use come::{
    ast,
    backend::{riscv, wasm},
    ir::{self, optimize},
};
use ezio::file;
use shadow_rs::shadow;
use std::io::Write;
shadow!(build);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, ValueEnum)]
enum Target {
    /// riscv backend
    RISCV,
    /// web assembly backend
    WASM,
}

/// Come language compiler.
#[derive(Parser, Debug)]
#[command(version, long_version = build::CLAP_LONG_VERSION, about, long_about = None)]
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

    #[arg(short = 't', long, value_enum)]
    target: Target,
}

fn main() {
    let args = Args::parse();
    let code = file::read(args.input);
    let ast = ast::from_source(&code).unwrap().1;
    let ir = ir::from_ast(&ast);
    let ir = optimize::optimize(ir, args.optimize);
    if let Some(emit_ir_path) = args.emit_ir_path {
        let mut w = file::writer(emit_ir_path);
        for ir in ir.iter() {
            writeln!(w, "{ir}").unwrap();
        }
    }
    match args.target {
        Target::RISCV => {
            let code = riscv::from_ir::emit_asm(&ir);
            file::write(args.output, &code);
        }
        Target::WASM => {
            let module = wasm::compile(&ir);
            let mut output_file = File::create(args.output).unwrap();
            output_file.write_all(module.as_slice()).unwrap();
        }
    }
}
