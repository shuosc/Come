use std::{fs::File, path::PathBuf};

use bincode::Options;
use clap::Parser;
use come::{backend::riscv::instruction, binary_format::clef::Clef};

use shadow_rs::shadow;
shadow!(build);

/// SHUOSC assembler.
#[derive(Parser, Debug)]
#[command(version, long_version = build::CLAP_LONG_VERSION, about, long_about = None)]
struct Args {
    /// Input file path.
    #[arg(short, long)]
    input: PathBuf,
}

fn main() {
    let args = Args::parse();
    let clef_file = File::open(args.input).unwrap();
    let loader = bincode::DefaultOptions::new().with_fixint_encoding();
    let clef: Clef = loader.deserialize_from(&clef_file).unwrap();
    println!("architecture: {}", clef.architecture);
    println!("os: {}", clef.os);
    for section in clef.sections {
        println!("section: {}", section.meta.name);
        println!("linkable: {}", section.meta.linkable);
        println!(
            "loadable: {}",
            if let Some(address) = section.meta.loadable {
                format!("should be loaded to {address}")
            } else {
                "no".to_string()
            }
        );
        println!("symbols:");
        for symbol in section.meta.symbols {
            println!("  {symbol}");
        }
        println!("pending symbols:");
        for pending_symbol in &section.meta.pending_symbols {
            println!("{pending_symbol}");
        }
        println!("content:",);
        let instructions =
            instruction::parse_whole_binary(&section.content, &section.meta.pending_symbols);
        for instruction in instructions {
            println!("  {instruction}");
        }
    }
}
