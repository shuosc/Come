use std::fmt::Display;

use bitvec::vec::BitVec;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Symbol {
    pub name: String,
    pub offset: u32,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: 0x{:x}", self.name, self.offset)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingSymbol {
    pub name: String,
    pub pending_instruction_offsets: Vec<u32>,
}

impl Display for PendingSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  {}:", self.name)?;
        writeln!(
            f,
            "{}",
            self.pending_instruction_offsets
                .iter()
                .map(|it| format!("0x{it:08x}"))
                .chunks(4)
                .into_iter()
                .map(|mut it| it.join(", "))
                .map(|it| format!("    {it}"))
                .join("\n")
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LinkableOrLoadable {
    Linkable,
    Loadable(u32),
}

impl Display for LinkableOrLoadable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkableOrLoadable::Linkable => write!(f, "linkable"),
            LinkableOrLoadable::Loadable(address) => {
                write!(f, "should be loaded into address 0x{address:x}")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SectionMeta {
    pub name: String,
    pub linkable_or_loadable: LinkableOrLoadable,
    pub symbols: Vec<Symbol>,
    pub pending_symbols: Vec<PendingSymbol>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Architecture {
    RiscV,
    Arm,
    X86,
}

impl Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::RiscV => write!(f, "riscv"),
            Architecture::Arm => write!(f, "arm"),
            Architecture::X86 => write!(f, "x86"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Os {
    BareMetal,
}

impl Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bare metal")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Section {
    pub meta: SectionMeta,
    pub content: BitVec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clef {
    pub architecture: Architecture,
    pub os: Os,
    pub sections: Vec<Section>,
}

impl Clef {
    pub fn new(architecture: Architecture, os: Os) -> Self {
        Self {
            architecture,
            os,
            sections: Vec::new(),
        }
    }
}
