/// Functions for generating asm from IR
pub mod from_ir;
/// Section name information and parser
mod section;
/// Instruction information parser
pub mod simple_instruction;

use self::{section::parse_section, simple_instruction::SimpleInstruction};
use crate::{
    binary_format::clef::{Architecture, Clef, Os, PendingSymbol, Section, SectionMeta, Symbol},
    utility::parsing,
};
use bitvec::prelude::*;
use itertools::Itertools;
use section::SectionName;
use std::{collections::HashMap, sync::OnceLock};

/// Directive in an asm file.
#[derive(Debug, PartialEq, Eq, Clone)]
enum Directive {
    /// Marks a global symbol to be exported.
    Global(String),
    /// Marks the beginning of a section.
    Section(SectionName),
}

fn parse_directive(line: &str) -> Directive {
    let mut parts = line
        .split(' ')
        .map(|it| it.trim())
        .filter(|it| !it.is_empty());
    let first_part = parts.next().unwrap();
    match first_part {
        ".globl" | ".global" => Directive::Global(parts.next().unwrap().to_string()),
        ".section" => Directive::Section(parse_section(parts.next().unwrap()).unwrap().1),
        section_name => Directive::Section(parse_section(section_name).unwrap().1),
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnparsedInstruction {
    name: String,
    params: Vec<String>,
}

/// A line in an asm file.
#[derive(Debug, PartialEq, Eq, Clone)]
enum Line {
    /// A tag.
    Tag(String),
    /// An instruction.
    Instruction(UnparsedInstruction),
    /// A directive.
    Directive(Directive),
}

fn instruction_line(line: &str) -> Line {
    let (name, params) = line.split_once(' ').unwrap_or((line, ""));
    let params = params
        .replace('(', ",")
        .replace(')', " ")
        .split(',')
        .map(|it| it.trim().to_string())
        .collect();
    Line::Instruction(UnparsedInstruction {
        name: name.to_string(),
        params,
    })
}

fn preprocess(code: &str) -> Vec<Line> {
    let mut result = Vec::new();
    for line in code.lines().map(|it| it.trim()).filter(|it| !it.is_empty()) {
        if line.ends_with(':') {
            result.push(Line::Tag(line.trim_end_matches(':').to_string()));
        } else if line.starts_with('.') {
            result.push(Line::Directive(parse_directive(line)));
        } else {
            result.push(instruction_line(line));
        }
    }
    result
}

fn replace_complex_pseudo(preprocessed: &[Line]) -> Vec<Line> {
    let mut result = Vec::new();
    for line in preprocessed {
        match line {
            t @ Line::Tag(_tag) => result.push(t.clone()),
            Line::Instruction(UnparsedInstruction { name, params }) => match name.as_str() {
                "li" => {
                    let param: i64 = parsing::integer(&params[1]).unwrap().1;
                    let lower = param & 0xfff;
                    let lower_is_negative = lower > 0x7ff;
                    let higher = (if lower_is_negative {
                        // lower is, in fact, a negative number when used in addi
                        (param >> 12) + 1
                    } else {
                        param >> 12
                    }) & 0xffffffff;
                    let lower = param - (higher << 12);
                    if higher == 0 && lower == 0 {
                        result.push(Line::Instruction(UnparsedInstruction {
                            name: "mv".to_string(),
                            params: vec![params[0].clone(), "zero".to_string()],
                        }))
                    } else if higher == 0 {
                        result.push(Line::Instruction(UnparsedInstruction {
                            name: "addi".to_string(),
                            params: vec![params[0].clone(), "x0".to_string(), format!("{lower}")],
                        }));
                    } else {
                        result.push(Line::Instruction(UnparsedInstruction {
                            name: "lui".to_string(),
                            params: vec![params[0].clone(), format!("0x{higher:x}")],
                        }));
                        if lower != 0 {
                            result.push(Line::Instruction(UnparsedInstruction {
                                name: "addi".to_string(),
                                params: vec![
                                    params[0].clone(),
                                    params[0].clone(),
                                    format!("{lower}"),
                                ],
                            }));
                        }
                    }
                }
                _ => result.push(Line::Instruction(UnparsedInstruction {
                    name: name.to_string(),
                    params: params.clone(),
                })),
            },
            d @ Line::Directive(_directive) => result.push(d.clone()),
        }
    }
    result
}

struct SimplePseudoTemplate {
    template: &'static str,
}

fn replace_simple_pseudo(complex_replaced: &[Line]) -> Vec<Line> {
    static PSEUDO_SIMPLE_INSTRUCTIONS: OnceLock<HashMap<&'static str, SimplePseudoTemplate>> =
        OnceLock::new();
    let pseudo_simple_instructions = PSEUDO_SIMPLE_INSTRUCTIONS.get_or_init(|| {
        let mut pseudo_simple_instructions = HashMap::new();
        let templates_str = include_str!("./spec/pseudo_simple.spec");
        let templates = templates_str
            .split('\n')
            .map(|it| it.trim())
            .filter(|it| !it.is_empty());
        for template in templates {
            let (name, template) = template.split_once(' ').unwrap();
            pseudo_simple_instructions.insert(
                name,
                SimplePseudoTemplate {
                    template: template.trim(),
                },
            );
        }
        pseudo_simple_instructions
    });
    let mut result = Vec::new();
    for line in complex_replaced {
        if let Line::Instruction(UnparsedInstruction { name, params }) = line {
            if let Some(SimplePseudoTemplate { template }) =
                pseudo_simple_instructions.get(name.as_str())
            {
                let mut replaced = template.to_string();
                for (i, param) in params.iter().enumerate() {
                    let param_pattern = format!("{{{{params[{i}]}}}}");
                    replaced = replaced.replace(&param_pattern, param);
                }
                result.push(instruction_line(&replaced));
            } else {
                result.push(line.clone());
            }
        } else {
            result.push(line.clone());
        }
    }
    result
}

// todo: test?
fn parse_single_section(
    simple_replaced: impl IntoIterator<Item = Line>,
) -> (Vec<SimpleInstruction>, Vec<Symbol>, Vec<PendingSymbol>) {
    let mut current_offset_bytes = 0u32;
    let mut simple_instructions = Vec::new();
    let mut all_symbols = HashMap::new();
    let mut exported_symbols = Vec::new();
    let mut pending_symbols = HashMap::new();
    for line in simple_replaced.into_iter() {
        match line {
            Line::Tag(tag) => {
                all_symbols.insert(tag, current_offset_bytes);
            }
            Line::Instruction(unparsed) => {
                let mut instruction: SimpleInstruction = unparsed.clone().try_into().unwrap();
                instruction.set_offset_bytes(current_offset_bytes);
                current_offset_bytes += (instruction.bit_count() / 8) as u32;
                simple_instructions.push(instruction);
            }
            Line::Directive(Directive::Global(symbol_name)) => {
                exported_symbols.push(symbol_name.clone());
            }
            Line::Directive(Directive::Section(_)) => {
                unreachable!("Please separate sections before calling to_instructions");
            }
        }
    }
    for (index, instruction) in simple_instructions.iter().enumerate() {
        if let Some(pending_symbol) = instruction.pending_symbol() {
            pending_symbols
                .entry(pending_symbol.name.clone())
                .or_insert(Vec::new())
                .push(index);
        }
    }
    pending_symbols.drain_filter(|name, indexes| {
        if let Some(symbol_offset_bytes) = all_symbols.get(name) {
            for index in indexes {
                simple_instructions[*index].decide_symbol(&Symbol {
                    name: name.clone(),
                    offset_bytes: *symbol_offset_bytes,
                });
            }
            true
        } else {
            false
        }
    });
    let exported_symbols = exported_symbols
        .into_iter()
        .map(|name| Symbol {
            offset_bytes: all_symbols[&name],
            name,
        })
        .collect();
    let pending_symbols = pending_symbols
        .into_iter()
        .map(|(name, offset_bytes)| PendingSymbol {
            name,
            pending_instructions_offset_bytes: offset_bytes
                .into_iter()
                .map(|index| simple_instructions[index].offset_bytes())
                .collect(),
        })
        .collect();
    (simple_instructions, exported_symbols, pending_symbols)
}

pub fn fill_pending_symbol(
    symbols: &[Symbol],
    pending_symbols: &[PendingSymbol],
    content: &mut BitVec<u32>,
) -> Vec<PendingSymbol> {
    let mut remaining_pending_symbols = Vec::new();
    for pending_symbol in pending_symbols {
        if let Some(corresponding_symbol) = symbols.iter().find(|it| it.name == pending_symbol.name)
        {
            for pending_instruction_offset_bytes in
                &pending_symbol.pending_instructions_offset_bytes
            {
                let pending_instruction_offset_bits = pending_instruction_offset_bytes * 8;
                let (_rest, mut instruction) = simple_instruction::parse_binary(
                    (
                        &content[pending_instruction_offset_bits as usize..],
                        pending_instruction_offset_bits as usize,
                    ),
                    pending_symbols,
                )
                .unwrap();
                instruction.decide_symbol(corresponding_symbol);
                let binary_form = instruction.render();
                content[pending_instruction_offset_bits as usize
                    ..pending_instruction_offset_bits as usize + instruction.bit_count()]
                    .copy_from_bitslice(&binary_form);
            }
        } else {
            remaining_pending_symbols.push(pending_symbol.clone());
        }
    }
    remaining_pending_symbols
}

// pub fn fill_pending_symbol<'a>(
//     pending_symbols: &'a HashMap<u32, &'a PendingSymbol>,
//     symbol_offsets: &'a HashMap<String, u32>,
//     pending_symbol: &'a PendingSymbol,
//     symbol: &Symbol,
//     content: &mut BitVec<u32>,
// ) {
//     for offset_bytes in &pending_symbol.pending_instruction_offset_bytes {
//         let offset_bits = *offset_bytes as usize * 8;
//         let current_content = (&content[offset_bits..], offset_bits);
//         let (_rest, mut instruction) =
//             instruction::parse_binary(current_content, pending_symbols).unwrap();
//         instruction.fill_symbol(*offset_bytes, symbol);
//         let bin = instruction.binary(*offset_bytes as _, symbol_offsets);
//         content[offset_bits..offset_bits + bin.len()].copy_from_bitslice(&bin);
//     }
// }

// pub fn fill_pending_symbols(meta: &mut SectionMeta, content: &mut BitVec<u32>) {
//     let pending_symbols = meta.offset_pending_symbol_map();
//     let symbol_offsets = meta.symbol_offsets();
//     let mut new_pending_symbols = Vec::new();
//     for current_pending_symbol in meta.pending_symbols.iter() {
//         if let Some(symbol) = meta
//             .symbols
//             .iter()
//             .find(|it| it.name == current_pending_symbol.name)
//         {
//             fill_pending_symbol(
//                 &pending_symbols,
//                 &symbol_offsets,
//                 current_pending_symbol,
//                 symbol,
//                 content,
//             );
//         } else {
//             new_pending_symbols.push(current_pending_symbol.clone());
//         }
//     }
//     meta.pending_symbols = new_pending_symbols;
// }

// fn get_offsets(replace_simple_pseudo_done: &[Line]) -> HashMap<String, u32> {
//     let mut result = HashMap::new();
//     let mut current_offset_bytes = 0;
//     for line in replace_simple_pseudo_done {
//         match line {
//             Line::Tag(tag) => {
//                 result.insert(tag.clone(), current_offset_bytes);
//             }
//             Line::Instruction(_instruction) => {
//                 // todo: handle none 4 byte instructions
//                 current_offset_bytes += 4;
//             }
//             Line::Directive(Directive::Section(_)) => {
//                 current_offset_bytes = 0;
//             }
//             Line::Directive(_) => {
//                 // todo: ("handle .data type directives")
//             }
//         }
//     }
//     result
// }

// Emit clef file from an asm file.
pub fn emit_clef(asm_code: &str) -> Clef {
    let mut result = Clef::new(Architecture::RiscV, Os::BareMetal);
    let preprocessed = preprocess(asm_code);
    let replace_complex_pseudo_done = replace_complex_pseudo(&preprocessed);
    let replace_simple_pseudo_done = replace_simple_pseudo(&replace_complex_pseudo_done);
    let mut line_iter = replace_simple_pseudo_done.into_iter();
    while !line_iter.is_empty() {
        let first_line = line_iter.next().unwrap();
        let current_section = if let Line::Directive(Directive::Section(section)) = first_line {
            section
        } else {
            panic!("First line must be a section directive");
        };
        let this_section_lines =
            line_iter.take_while_ref(|it| !matches!(it, Line::Directive(Directive::Section(_))));
        let (instructions, symbols, pending_symbols) =
            parse_single_section(this_section_lines.into_iter());
        result.sections.push(Section {
            meta: SectionMeta {
                name: format!("{current_section}"),
                linkable: true,
                loadable: None,
                symbols,
                pending_symbols,
            },
            content: instructions.into_iter().map(|it| it.render()).fold(
                BitVec::new(),
                |mut acc, instruction| {
                    acc.extend_from_bitslice(&instruction);
                    acc
                },
            ),
        })
    }
    result
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;

    use super::*;
    #[test]
    fn test_preprocess() {
        let code = r#"
            label:
                addi t0, t1, 1
                li t2, 0x998
                not t3, t4
                lb t5, 4(t6)
        "#;
        let preprocessed = preprocess(code);
        assert_eq!(
            preprocessed,
            vec![
                Line::Tag("label".to_string()),
                Line::Instruction(UnparsedInstruction {
                    name: "addi".to_string(),
                    params: vec!["t0".to_string(), "t1".to_string(), "1".to_string()]
                }),
                Line::Instruction(UnparsedInstruction {
                    name: "li".to_string(),
                    params: vec!["t2".to_string(), "0x998".to_string()]
                }),
                Line::Instruction(UnparsedInstruction {
                    name: "not".to_string(),
                    params: vec!["t3".to_string(), "t4".to_string()]
                }),
                Line::Instruction(UnparsedInstruction {
                    name: "lb".to_string(),
                    params: vec!["t5".to_string(), "4".to_string(), "t6".to_string()]
                }),
            ]
        );
    }

    #[test]
    fn test_replace_complex_pseudo() {
        let lines = vec![
            Line::Tag("label".to_string()),
            Line::Instruction(UnparsedInstruction {
                name: "addi".to_string(),
                params: vec!["t0".to_string(), "t1".to_string(), "1".to_string()],
            }),
            Line::Instruction(UnparsedInstruction {
                name: "li".to_string(),
                params: vec!["t2".to_string(), "0x998".to_string()],
            }),
            Line::Instruction(UnparsedInstruction {
                name: "not".to_string(),
                params: vec!["t3".to_string(), "t4".to_string()],
            }),
            Line::Instruction(UnparsedInstruction {
                name: "lb".to_string(),
                params: vec!["t5".to_string(), "4".to_string(), "t6".to_string()],
            }),
        ];
        let result = replace_complex_pseudo(&lines);
        assert_eq!(
            result,
            vec![
                Line::Tag("label".to_string()),
                Line::Instruction(UnparsedInstruction {
                    name: "addi".to_string(),
                    params: vec!["t0".to_string(), "t1".to_string(), "1".to_string()]
                }),
                Line::Instruction(UnparsedInstruction {
                    name: "lui".to_string(),
                    params: vec!["t2".to_string(), "0x1".to_string()]
                }),
                Line::Instruction(UnparsedInstruction {
                    name: "addi".to_string(),
                    params: vec!["t2".to_string(), "t2".to_string(), "-1640".to_string()]
                }),
                Line::Instruction(UnparsedInstruction {
                    name: "not".to_string(),
                    params: vec!["t3".to_string(), "t4".to_string()]
                }),
                Line::Instruction(UnparsedInstruction {
                    name: "lb".to_string(),
                    params: vec!["t5".to_string(), "4".to_string(), "t6".to_string()]
                }),
            ]
        );
    }

    #[test]
    fn test_replace_simple_pseudo() {
        let lines = vec![
            Line::Tag("label".to_string()),
            Line::Instruction(UnparsedInstruction {
                name: "mv".to_string(),
                params: vec!["t0".to_string(), "t1".to_string()],
            }),
        ];
        let result = replace_simple_pseudo(&lines);
        assert_eq!(
            result,
            vec![
                Line::Tag("label".to_string()),
                Line::Instruction(UnparsedInstruction {
                    name: "addi".to_string(),
                    params: vec!["t0".to_string(), "t1".to_string(), "0".to_string()]
                }),
            ]
        );
    }

    #[test]
    fn test_emit_clef() {
        let code = r#"
.section .text
.global main
main:
main_entry:
    addi t0, t0, 1
    mv t0, t1
    li t2, 0x998
    not t3, t4
    sw t5, 4(t6)"#;
        let result = emit_clef(code);
        assert_eq!(result.sections[0].meta.name, ".text");
        assert_eq!(result.sections[0].meta.symbols[0].name, "main");
        assert_eq!(result.sections[0].meta.symbols[0].offset_bytes, 0);
        assert_eq!(result.sections[0].content[0..32].load_le::<u32>(), 0x128293);
        assert_eq!(
            result.sections[0].content[32..32 * 2].load_le::<u32>(),
            0x30293
        );
        assert_eq!(
            result.sections[0].content[32 * 2..32 * 3].load_le::<u32>(),
            0x13b7
        );
        assert_eq!(
            result.sections[0].content[32 * 3..32 * 4].load_le::<u32>(),
            0x99838393
        );
        assert_eq!(
            result.sections[0].content[32 * 4..32 * 5].load_le::<u32>(),
            0xfffece13
        );
        assert_eq!(
            result.sections[0].content[32 * 5..32 * 6].load_le::<u32>(),
            0x1efa223
        );
    }
}
