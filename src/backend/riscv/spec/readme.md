# RISC-V specs

This folder contains several spec files which record information about how to mapping asm code to binary instructions.

## [registers.spec](./registers.spec)

This file maps the register names to their id.

## [csr.spec](./csr.spec)

This file maps the csr names to their id.

## [instructions.spec](./instructions.spec)

This file maps the instruction names to their binary format.

Each lines in this file has 2 parts,
the first part is the instruction name,
the second part is the binary format template.

## [pseudo_simple.spec](./pseudo_simple.spec)

Simple pseudo instructions.

Each lines in this file has 2 parts,
the first part is the pseudo instruction name,
the second part is the template for expanding this pseudo instruction.
