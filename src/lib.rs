#![feature(once_cell)]
#![feature(let_chains)]
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
