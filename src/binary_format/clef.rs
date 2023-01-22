use std::{fmt::Display, mem};

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
    pub offset: u32,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: 0x{:x}", self.name, self.offset)
    }
}

/// An unknown symbol in compile time, which address/offset is waiting to be determined for linking.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
        // we presume the entry offset of all sections is 0
        // thus, if we link a loadable section with a non-loadable section,
        // we keep the loadable section at the front
        if other.meta.loadable.is_some() {
            (other, self) = (self, other);
        }
        // "chain" other to self
        // add offsets to symbols in other
        let self_bytes = self.content.len() as u32 / 8;
        other.meta.symbols.iter_mut().for_each(|symbol| {
            symbol.offset += self_bytes;
        });
        // add offsets to pending instructions in other
        other
            .meta
            .pending_symbols
            .iter_mut()
            .for_each(|pending_symbol| {
                pending_symbol
                    .pending_instruction_offsets
                    .iter_mut()
                    .for_each(|offset| {
                        *offset += self_bytes;
                    });
            });
        // merge content
        self.content.extend_from_bitslice(&other.content);
        // fill pending symbols with known symbols in other
        self.meta.pending_symbols = update_pending_symbols_in_content(
            self.meta.pending_symbols,
            &other.meta.symbols,
            &mut self.content,
            architecture,
        );
        // fill pending symbols with known symbols in self
        other.meta.pending_symbols = update_pending_symbols_in_content(
            other.meta.pending_symbols,
            &self.meta.symbols,
            &mut self.content,
            architecture,
        );
        // merge symbols and pending_symbols
        self.meta.symbols.extend(other.meta.symbols);
        self.meta.pending_symbols.extend(other.meta.pending_symbols);
        self
    }
}

fn update_pending_symbol_in_content(
    pending_symbol: &PendingSymbol,
    symbol: &Symbol,
    content: &mut BitVec<u32>,
    architecture: Architecture,
) {
    match architecture {
        Architecture::RiscV => {
            backend::riscv::decide_instruction_symbol(pending_symbol, symbol, content)
        }
        Architecture::Arm => todo!(),
        Architecture::X86 => todo!(),
    }
}

fn update_pending_symbols_in_content(
    pending_symbols: Vec<PendingSymbol>,
    symbols: &[Symbol],
    content: &mut BitVec<u32>,
    architecture: Architecture,
) -> Vec<PendingSymbol> {
    pending_symbols
        .into_iter()
        .filter_map(|pending_symbol| {
            if let Some(symbol) = symbols
                .iter()
                .find(|symbol| symbol.name == pending_symbol.name)
            {
                update_pending_symbol_in_content(&pending_symbol, symbol, content, architecture);
                None
            } else {
                Some(pending_symbol)
            }
        })
        .collect()
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
    use std::collections::BTreeMap;

    use crate::backend::riscv::instruction::{self, instruction};

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
                        offset: 0,
                    },
                    Symbol {
                        name: "f".to_string(),
                        offset: 4,
                    },
                ],
                pending_symbols: vec![PendingSymbol {
                    name: "test1".to_string(),
                    pending_instruction_offsets: vec![0],
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
                        offset: 0,
                    },
                    Symbol {
                        name: "test1".to_string(),
                        offset: 8,
                    },
                ],
                pending_symbols: vec![PendingSymbol {
                    name: "f".to_string(),
                    pending_instruction_offsets: vec![8],
                }],
                linkable: true,
            },
            content: section2_content,
        };
        clef1.sections.push(section1);
        clef2.sections.push(section2);
        let clef = clef1.merge(clef2);
        let mut offset_pending_symbol_map = BTreeMap::new();
        for pending_symbol in &clef.sections[0].meta.pending_symbols {
            for pending_instruction_offset in &pending_symbol.pending_instruction_offsets {
                offset_pending_symbol_map.insert(*pending_instruction_offset, pending_symbol);
            }
        }
        let mut content: &BitSlice<u32> = &clef.sections[0].content;
        let mut offset = 0usize;
        let mut expected = vec![
            instruction!(jal, x0, 16),
            instruction!(addi, x1, x1, 2),
            instruction!(addi, x1, x1, 2),
            instruction!(addi, x2, x2, 3),
            instruction!(jal, x0, -12),
        ];
        expected.reverse();
        while !content.is_empty() {
            let hex = content[0..32].load_le::<u32>();
            let (rest, result) = if let Some((offset_bytes, pending_symbol)) =
                offset_pending_symbol_map.pop_first()
            {
                if offset_bytes * 8 == offset as _ {
                    instruction::parse_bin_with_pending(content, pending_symbol).unwrap()
                } else {
                    offset_pending_symbol_map.insert(offset_bytes, pending_symbol);
                    instruction::parse_bin(content).unwrap()
                }
            } else {
                instruction::parse_bin(content).unwrap()
            };
            let expected_instruction = expected.pop().unwrap();
            assert_eq!(expected_instruction, result);
            content = rest;
            offset += 32;
        }
        // assert_eq!(clef.sections.len(), 1);
        // assert_eq!(clef.sections[0].meta.symbols.len(), 2);
        // assert_eq!(clef.sections[0].meta.symbols[0].offset, 0);
        // assert_eq!(clef.sections[0].meta.symbols[1].offset, 32);
    }
}
