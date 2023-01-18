#![feature(once_cell)]
#![feature(let_chains)]
/// Definitions of AST nodes and their parser.
pub mod ast;
/// Functions for generating assembly code from ir.
pub mod backend;
pub mod binary;
/// Definitions of IR nodes and their parser, and ir generator functions for generating ir from ast.
pub mod ir;
/// Utilities shared among modules.
pub mod utility;
