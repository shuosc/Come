+++
title = "Pending Symbol"
description = "desc"
date = 2023-01-23T09:00:47.335Z
updated = 2023-01-23T09:00:47.335Z
draft = false
template = "blog/page.html"

[extra]
lead = "(Reference to) a symbol in other clef files."
+++

Pending symbol is an "unknown" [symbol](@/concepts/symbol.md) in compile time, which is used to represent a symbol which is not defined in this `clef` file. It is used to represent a symbol which should be in another `clef` file.

In a `clef` file, pending symbols in this section's information (includes the offsets of instructions which are using this symbol) are part of the section's metadata. These information are accessed during linking stage to enable calling across different `clef` files.
