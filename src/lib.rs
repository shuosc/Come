#![feature(lazy_cell)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(hash_extract_if)]
#![feature(exact_size_is_empty)]
#![feature(assert_matches)]
/// Definitions of AST nodes and their parser.
pub mod ast;
/// Functions for generating assembly and binary code from ir.
pub mod backend;
/// Definitions of binary (linkable or executable) format.
pub mod binary_format;
/// Definitions of IR nodes and their parser, and ir generator functions for generating ir from ast.
pub mod ir;
/// Utilities shared among modules.
pub mod utility;
