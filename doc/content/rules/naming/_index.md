+++
title = "Naming"
description = "Naming rules for Come."
date = 2023-07-16T04:21:20.130Z
updated = 2023-07-16T04:21:20.130Z
template = "docs/section.html"
sort_by = "weight"
weight = 1
draft = false
+++

Except for the rules mentioned on [rust naming guides](https://rust-lang.github.io/api-guidelines/naming.html),
there are some additional rules for Come.

## Don't use undocumented abbreviations

We don't use abbreviations unless they are documented in the glossary.

Here is the glossary:

| Abbreviation | Origin | In Module |
|--------------|--------|-----------|
|     ast      | Abstract Syntax Tree |    (All)      |
|     asm      | ASseMbly language       |   (All)        |
|    ir        | Intermediate Representation       |     (All)      |
|    bb        | Basic Block       |     `come::ir`      |
