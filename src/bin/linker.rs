use std::{fs::File, path::PathBuf};

use bincode::Options;
use clap::Parser;
use come::binary_format::clef::{Architecture, Clef, Os};
use shadow_rs::shadow;
shadow!(build);

/// SHUOSC linker.
#[derive(Parser, Debug)]
#[command(version, long_version = build::CLAP_LONG_VERSION, about, long_about = None)]
struct Args {
    /// Input file path.
    #[arg(short, long)]
    input: Vec<PathBuf>,
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let mut result = args
        .input
        .iter()
        .map(File::open)
        .map(Result::unwrap)
        .map(|file| {
            bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .deserialize_from(&file)
                .unwrap()
        })
        .fold(Clef::new(Architecture::RiscV, Os::BareMetal), Clef::merge);
    result
        .sections
        .iter_mut()
        .find(|it| it.meta.name == ".text")
        .map(|it| it.meta.loadable = Some(0x8000_0000))
        .unwrap();
    let mut output_file = File::create(args.output).unwrap();
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .serialize_into(&mut output_file, &result)
        .unwrap();
}
