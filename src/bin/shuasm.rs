use std::{fs::File, io::Write, path::PathBuf};

use bincode::Options;
use clap::Parser;
use come::backend::riscv::emit_clef;
use ezio::file;
use shadow_rs::shadow;
shadow!(build);

/// SHUOSC assembler.
#[derive(Parser, Debug)]
#[command(version, long_version = build::CLAP_LONG_VERSION, about, long_about = None)]
struct Args {
    /// Input file path.
    #[arg(short, long)]
    input: PathBuf,

    /// Output file path.
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let asm_code = file::read(args.input);
    let clef = emit_clef(&asm_code);
    let dumper = bincode::DefaultOptions::new().with_fixint_encoding();
    let file_content = dumper.serialize(&clef).unwrap();
    let mut output_file = File::create(args.output).unwrap();
    output_file.write_all(&file_content).unwrap();
}
