---
title: Macros
description: Compile-time constructs built into the scrippo compiler.
---

A macro is a compile-time construct. The compiler resolves it before type checking, and it never becomes a bash function. Every macro is built into the compiler and written with a leading `@`.

You cannot define your own macros yet. A definition such as `@name()` followed by an indented body is a compile error, and a call to any `@name` that is not listed below fails with `unknown macro`. When a snippet needs reuse, write a [function](/functions).

## Built-in macros

| Form | Role |
| --- | --- |
| `@import` | Compile-time inclusion. [Imports](/imports). |
| `@info` | File metadata. [Metadata](/metadata). |
| `@usage` | Print usage. [Usage text](/usage). |
| `@inherit` | Copy the parent's parameters. [Inherit](/inherit). |

`@info` is only legal at the top of a file.

## Order of resolution

1. `@info` is read and removed.
2. `@import` is loaded, recursively, and removed.
3. `@inherit` in a parameter list is replaced by the parent parameter list. Any other unknown `@name` is a compile error.
4. The program is type-checked and emitted. `@usage` is emitted as a call to the generated usage function.
