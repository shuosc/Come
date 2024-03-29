use std::{collections::HashMap, fmt::Display, mem};

use bitvec::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::backend;

/// A symbol in clef file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Symbol {
    /// Name of the symbol.
    pub name: String,
    /// Offset of the definition of the symbol in the content part of clef.
    pub offset_bytes: u32,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: 0x{:08x}", self.name, self.offset_bytes)
    }
}

/// An unknown symbol in compile time, which address/offset is waiting to be determined for linking.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingSymbol {
    /// Name of the symbol.
    pub name: String,
    /// These instructions' nth param are waiting for the address of this symbol.
    pub pending_instructions_offset_bytes: Vec<u32>,
}

impl Display for PendingSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  {}:", self.name)?;
        writeln!(
            f,
            "{}",
            self.pending_instructions_offset_bytes
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

impl PendingSymbol {
    pub fn used_by_instruction_at_offset(&self, offset_bytes: u32) -> bool {
        self.pending_instructions_offset_bytes
            .iter()
            .any(|&it| it == offset_bytes)
    }
}

/// Metadata of a section.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SectionMeta {
    /// Name of the section.
    pub name: String,
    /// Whether this section is linkable.
    pub linkable: bool,
    /// Target load address of this section.
    pub loadable: Option<u32>,
    /// Symbols defined in this section.
    pub symbols: Vec<Symbol>,
    /// Symbols used in this section, but not defined in this section.
    pub pending_symbols: Vec<PendingSymbol>,
}

impl SectionMeta {
    pub fn symbol_offsets(&self) -> HashMap<String, u32> {
        self.symbols
            .iter()
            .map(|it| (it.name.clone(), it.offset_bytes))
            .collect()
    }
}

/// Target architecture of the binary.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Os {
    BareMetal,
}

impl Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bare metal")
    }
}

/// A section in clef file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Section {
    /// Metadata of this section.
    pub meta: SectionMeta,
    /// Content of this section.
    pub content: BitVec<u32>,
}

impl Section {
    fn merge(mut self, mut other: Self, architecture: Architecture) -> Self {
        assert!(self.meta.name == other.meta.name);
        // we presume the "entry" offset of all sections is 0
        // thus, if we link a loadable section with a non-loadable section,
        // we keep the loadable section at the front
        if other.meta.loadable.is_some() {
            (other, self) = (self, other);
        }
        // "chain" other to self
        // add offsets to symbols in `other`
        let self_bytes = self.content.len() as u32 / 8;
        other.meta.symbols.iter_mut().for_each(|symbol| {
            symbol.offset_bytes += self_bytes;
        });
        // add offsets to pending instructions in `other`
        other
            .meta
            .pending_symbols
            .iter_mut()
            .for_each(|pending_symbol| {
                pending_symbol
                    .pending_instructions_offset_bytes
                    .iter_mut()
                    .for_each(|offset| {
                        *offset += self_bytes;
                    });
            });
        // merge content
        self.content.extend_from_bitslice(&other.content);
        // merge symbols and pending_symbols
        self.meta.symbols.extend(other.meta.symbols);
        self.meta.pending_symbols.extend(other.meta.pending_symbols);
        resolve_pending_symbols(&mut self.meta, &mut self.content, architecture);
        self
    }
}

fn resolve_pending_symbols(
    meta: &mut SectionMeta,
    content: &mut BitVec<u32>,
    architecture: Architecture,
) {
    match architecture {
        Architecture::RiscV => {
            let remaining_pending = backend::riscv::resolve_pending_symbol(
                &meta.symbols,
                &meta.pending_symbols,
                content,
            );
            meta.pending_symbols = remaining_pending;
        }
        Architecture::Arm => todo!(),
        Architecture::X86 => todo!(),
    }
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

    pub fn merge(mut self, mut other: Self) -> Self {
        assert!(self.architecture == other.architecture);
        assert!(self.os == other.os);
        for other_section in mem::take(&mut other.sections) {
            let mut found = false;
            let mut result_sections = Vec::new();
            for self_section in mem::take(&mut self.sections) {
                if self_section.meta.name == other_section.meta.name {
                    found = true;
                    result_sections.push(Section::merge(
                        self_section,
                        other_section.clone(),
                        self.architecture,
                    ));
                    break;
                }
            }
            self.sections = result_sections;
            if !found {
                self.sections.push(other_section);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::riscv::simple_instruction::{self, instruction};

    use super::*;

    #[test]
    fn test_merge() {
        let mut clef1 = Clef::new(Architecture::RiscV, Os::BareMetal);
        let mut clef2 = Clef::new(Architecture::RiscV, Os::BareMetal);
        let section1_content = [0x0000006fu32, 0x00208093].as_bits::<Lsb0>().to_bitvec();
        let section1 = Section {
            meta: SectionMeta {
                name: "text".to_string(),
                loadable: Some(0),
                symbols: vec![
                    Symbol {
                        name: "main".to_string(),
                        offset_bytes: 0,
                    },
                    Symbol {
                        name: "f".to_string(),
                        offset_bytes: 4,
                    },
                ],
                pending_symbols: vec![PendingSymbol {
                    name: "test1".to_string(),
                    pending_instructions_offset_bytes: vec![0],
                }],
                linkable: true,
            },
            content: section1_content,
        };
        let section2_content = [0x00208093u32, 0x00310113, 0xff9ff06f]
            .as_bits::<Lsb0>()
            .to_bitvec();
        let section2 = Section {
            meta: SectionMeta {
                name: "text".to_string(),
                loadable: None,
                symbols: vec![
                    Symbol {
                        name: "dumb".to_string(),
                        offset_bytes: 0,
                    },
                    Symbol {
                        name: "test1".to_string(),
                        offset_bytes: 8,
                    },
                ],
                pending_symbols: vec![PendingSymbol {
                    name: "f".to_string(),
                    pending_instructions_offset_bytes: vec![8],
                }],
                linkable: true,
            },
            content: section2_content,
        };
        clef1.sections.push(section1);
        clef2.sections.push(section2);
        let clef = clef1.merge(clef2);
        let expected = vec![
            instruction!(jal, x0, 16),
            instruction!(addi, x1, x1, 2),
            instruction!(addi, x1, x1, 2),
            instruction!(addi, x2, x2, 3),
            instruction!(jal, x0, -12),
        ];
        let instructions = simple_instruction::parse_whole_binary(
            &clef.sections[0].content,
            &clef.sections[0].meta.pending_symbols,
        );
        for (instruction, expected) in instructions.into_iter().zip(expected) {
            assert_eq!(expected, instruction);
        }
        assert_eq!(clef.sections.len(), 1);
        assert_eq!(clef.sections[0].meta.name, "text");
        assert!(clef.sections[0].meta.pending_symbols.is_empty());
        assert_eq!(clef.sections[0].meta.symbols.len(), 4);
    }
}
