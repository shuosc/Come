+++
title = "Pseudo Instruction"
description = "desc"
date = 2023-01-27T11:19:43.597Z
updated = 2023-01-27T11:19:43.597Z
draft = false
template = "blog/page.html"

[extra]
lead = "Pseudo instruction is an asm instruction which cannot be rendered directly, but needs to be translated into one or several [simple instructions](@/concepts/simple_instruction.md) first."
+++

Simple pseudo instructions' translating can be done by a template which map a pseudo instruction to one or several simple instructions directly.

Complex pseudo instructions' translating may be depending on its params, and cannot be done with directly template mapping. For example, `li` instruction can be translated into a single `addi` instruction with a zero register as the first param when the immediate value has only 12 bits, but needs to be translated into two `lui` and `addi` instructions if the immediate value has more than than 12 bits.
