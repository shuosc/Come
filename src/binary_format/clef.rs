use std::fmt::Display;

use bitvec::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// A symbol in clef file
#[derive(Serialize, Deserialize, Debug)]
pub struct Symbol {
    /// Name of the symbol.
    pub name: String,
    /// Offset of the definition of the symbol in the content part of clef.
    pub offset: u32,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: 0x{:x}", self.name, self.offset)
    }
}

/// An unknown symbol in compile time, which address/offset is waiting to be determined for linking.
#[derive(Serialize, Deserialize, Debug)]
pub struct PendingSymbol {
    /// Name of the symbol.
    pub name: String,
    /// These instructions are waiting for the address of this symbol.
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

/// Whether a section is linkable or loadable.
#[derive(Serialize, Deserialize, Debug)]
pub enum LinkableOrLoadable {
    /// This section is linkable with other sections with same name.
    Linkable,
    /// This section should be load into an address.
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

/// Metadata of a section.
#[derive(Serialize, Deserialize, Debug)]
pub struct SectionMeta {
    /// Name of the section.
    pub name: String,
    /// Whether this section is linkable or loadable.
    pub linkable_or_loadable: LinkableOrLoadable,
    /// Symbols defined in this section.
    pub symbols: Vec<Symbol>,
    /// Symbols used in this section, but not defined in this section.
    pub pending_symbols: Vec<PendingSymbol>,
}

/// Target architecture of the binary.
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

/// Target operating system of the binary.
#[derive(Serialize, Deserialize, Debug)]
pub enum Os {
    BareMetal,
}

impl Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bare metal")
    }
}

/// A section in clef file.
#[derive(Serialize, Deserialize, Debug)]
pub struct Section {
    /// Metadata of this section.
    pub meta: SectionMeta,
    /// Content of this section.
    pub content: BitVec<u32>,
}

/// A clef file.
#[derive(Serialize, Deserialize, Debug)]
pub struct Clef {
    /// Target architecture of the binary.
    pub architecture: Architecture,
    /// Target operating system of the binary.
    pub os: Os,
    /// Sections in the binary.
    pub sections: Vec<Section>,
}

impl Clef {
    /// Create a new clef file.
    pub fn new(architecture: Architecture, os: Os) -> Self {
        Self {
            architecture,
            os,
            sections: Vec::new(),
        }
    }
}
