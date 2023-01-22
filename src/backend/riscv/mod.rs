/// Functions for generating asm from IR
pub mod from_ir;
/// Instruction information parser
pub mod instruction;
/// Section name information and parser
mod section;

use self::section::parse_section;
use crate::{
    binary_format::clef::{self, Architecture, Clef, Os, PendingSymbol, SectionMeta, Symbol},
    utility::parsing,
};
use bitvec::prelude::*;
use section::Section;
use std::{collections::HashMap, sync::OnceLock};

/// Directive in an asm file.
#[derive(Debug, PartialEq, Eq, Clone)]
enum Directive {
    /// Marks a global symbol to be exported.
    Global(String),
    /// Marks the beginning of a section.
    Section(Section),
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

/// A line in an asm file.
#[derive(Debug, PartialEq, Eq, Clone)]
enum Line {
    /// A tag.
    Tag(String),
    /// An instruction.
    Instruction(instruction::Unparsed),
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
    Line::Instruction(instruction::Unparsed {
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
            Line::Instruction(instruction::Unparsed { name, params }) => match name.as_str() {
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
                        result.push(Line::Instruction(instruction::Unparsed {
                            name: "mv".to_string(),
                            params: vec![params[0].clone(), "zero".to_string()],
                        }))
                    } else if higher == 0 {
                        result.push(Line::Instruction(instruction::Unparsed {
                            name: "addi".to_string(),
                            params: vec![params[0].clone(), "x0".to_string(), format!("{lower}")],
                        }));
                    } else {
                        result.push(Line::Instruction(instruction::Unparsed {
                            name: "lui".to_string(),
                            params: vec![params[0].clone(), format!("0x{higher:x}")],
                        }));
                        if lower != 0 {
                            result.push(Line::Instruction(instruction::Unparsed {
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
                _ => result.push(Line::Instruction(instruction::Unparsed {
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
        if let Line::Instruction(instruction::Unparsed { name, params }) = line {
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

pub fn decide_instruction_symbol(
    pending_symbol: &PendingSymbol,
    symbol: &Symbol,
    content: &mut BitVec<u32>,
) {
    for offset in &pending_symbol.pending_instruction_offsets {
        let (_rest, mut instruction) =
            instruction::parse_bin_with_pending(&content[(*offset * 8) as usize..], pending_symbol)
                .unwrap();
        instruction.fill_symbol(*offset, symbol);
        let bin = instruction.binary(*offset as _);
        content[(*offset * 8) as usize..(*offset * 8) as usize + bin.len()]
            .copy_from_bitslice(&bin);
    }
}

/// Emit clef file from an asm file.
pub fn emit_clef(asm_code: &str) -> Clef {
    let mut result = Clef::new(Architecture::RiscV, Os::BareMetal);
    let preprocessed = preprocess(asm_code);
    let replace_complex_pseudo_done = replace_complex_pseudo(&preprocessed);
    let replace_simple_pseudo_done = replace_simple_pseudo(&replace_complex_pseudo_done);
    let mut section_map: HashMap<String, clef::Section> = HashMap::new();
    let mut current_section_name = String::new();
    let mut current_offset = 0;
    for line in replace_simple_pseudo_done {
        match line {
            Line::Tag(tag) => {
                let current_section = section_map.get_mut(&current_section_name).unwrap();
                if let Some(symbol) = current_section
                    .meta
                    .symbols
                    .iter_mut()
                    .find(|it| it.name == tag)
                {
                    symbol.offset = current_offset
                }
            }
            Line::Instruction(unparsed) => {
                let parsed = instruction::Parsed::from(unparsed);
                let binary = parsed.binary(current_offset as _);
                let current_section = section_map.get_mut(&current_section_name).unwrap();
                current_section.content.extend_from_bitslice(&binary);
                current_offset += 4;
            }
            Line::Directive(directive) => match directive {
                Directive::Global(symbol_name) => {
                    let current_section = section_map.get_mut(&current_section_name).unwrap();
                    current_section.meta.symbols.push(Symbol {
                        name: symbol_name,
                        offset: 0,
                    });
                }
                Directive::Section(section) => {
                    current_section_name = format!("{section}");
                    section_map
                        .entry(current_section_name.clone())
                        .or_insert(clef::Section {
                            meta: SectionMeta {
                                name: current_section_name.clone(),
                                symbols: vec![],
                                pending_symbols: vec![],
                                linkable: true,
                                loadable: None,
                            },
                            content: BitVec::new(),
                        });
                }
            },
        }
    }
    result.sections = section_map.into_values().collect();
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
                Line::Instruction(instruction::Unparsed {
                    name: "addi".to_string(),
                    params: vec!["t0".to_string(), "t1".to_string(), "1".to_string()]
                }),
                Line::Instruction(instruction::Unparsed {
                    name: "li".to_string(),
                    params: vec!["t2".to_string(), "0x998".to_string()]
                }),
                Line::Instruction(instruction::Unparsed {
                    name: "not".to_string(),
                    params: vec!["t3".to_string(), "t4".to_string()]
                }),
                Line::Instruction(instruction::Unparsed {
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
            Line::Instruction(instruction::Unparsed {
                name: "addi".to_string(),
                params: vec!["t0".to_string(), "t1".to_string(), "1".to_string()],
            }),
            Line::Instruction(instruction::Unparsed {
                name: "li".to_string(),
                params: vec!["t2".to_string(), "0x998".to_string()],
            }),
            Line::Instruction(instruction::Unparsed {
                name: "not".to_string(),
                params: vec!["t3".to_string(), "t4".to_string()],
            }),
            Line::Instruction(instruction::Unparsed {
                name: "lb".to_string(),
                params: vec!["t5".to_string(), "4".to_string(), "t6".to_string()],
            }),
        ];
        let result = replace_complex_pseudo(&lines);
        assert_eq!(
            result,
            vec![
                Line::Tag("label".to_string()),
                Line::Instruction(instruction::Unparsed {
                    name: "addi".to_string(),
                    params: vec!["t0".to_string(), "t1".to_string(), "1".to_string()]
                }),
                Line::Instruction(instruction::Unparsed {
                    name: "lui".to_string(),
                    params: vec!["t2".to_string(), "0x1".to_string()]
                }),
                Line::Instruction(instruction::Unparsed {
                    name: "addi".to_string(),
                    params: vec!["t2".to_string(), "t2".to_string(), "-1640".to_string()]
                }),
                Line::Instruction(instruction::Unparsed {
                    name: "not".to_string(),
                    params: vec!["t3".to_string(), "t4".to_string()]
                }),
                Line::Instruction(instruction::Unparsed {
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
            Line::Instruction(instruction::Unparsed {
                name: "mv".to_string(),
                params: vec!["t0".to_string(), "t1".to_string()],
            }),
        ];
        let result = replace_simple_pseudo(&lines);
        assert_eq!(
            result,
            vec![
                Line::Tag("label".to_string()),
                Line::Instruction(instruction::Unparsed {
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
        assert_eq!(result.sections[0].meta.symbols[0].offset, 0);
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
