+++
title = "Distance (between symbols/instructions)"
description = "desc"
date = 2023-01-27T09:10:37.493Z
updated = 2023-01-27T09:10:37.493Z
draft = false
template = "blog/page.html"

[extra]
lead = "Distance (between symbols/instructions) is the distance between two symbols/instructions, can be negative."
+++

For example:

```asm
a:
  addi x1, x0, 1
b:
  addi x2, x0, 2
```

The distance from `a` to `b` is 4 bytes.

The distance from `b` to `a` is -4 bytes.

Disambiguation:

- [address](@/concepts/address.md)
- [offset](@/concepts/offset.md)
