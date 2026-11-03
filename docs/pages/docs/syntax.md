---
title: Files and syntax
description: Indentation, comments, names, shebangs, and how a line becomes a statement.
---

A scrippo source file is a sequence of statements. A statement is one logical line, plus the lines indented under it. The compiler skips blank lines and lines whose first non-whitespace character is `#`. Quoted strings and parenthesized [list and dict values](/values) can continue across lines until their closing delimiter.

## Names

Function, command, variable, and macro names are ASCII. A variable or function starts with a letter or `_`, then letters, digits, or `_`. Command names may also contain `-`. Package and script names used by the CLI start with a letter or `_` and may contain `-`.

```scrippo
count<int>=1
.bump-version()
  -> $count
```

## Indentation

The body of a function, command, `if`, `for`, `while`, `case` arm, or macro is every following statement indented deeper than the header. The block ends when a line returns to the header's indent or less. Spaces and tabs both count. One tab is one indent unit, the same as one space, so a file should pick one and keep it. Examples in this manual use two spaces.

```scrippo
if $ready
  -> "yes"
-> "after the if"
```

There is no `fi`, `done`, or `esac` in scrippo. A line that is exactly `fi` or `esac` is a compile error. End the block by outdenting.

`else` and `elif` sit at the same indent as their `if`:

```scrippo
if $n -lt 0
  -> "negative"
elif $n -eq 0
  -> "zero"
else
  -> "positive"
```

## Comments

`#` starts a comment that runs to the end of the line, except inside quotes. `#?` is a documentation comment. Documentation comments immediately above a command, function, parameter, or option are copied into usage text. Ordinary `#` comments are discarded and never appear in the bash output.

```scrippo
# ignored by the compiler
#? Shown in --help
.deploy(
  target #? Host to update
)
  -> $target
```

`#?` may sit on the same line as a parameter, after the declaration. See [Usage text](/usage).

## Shebang

A shebang is a comment, because it starts with `#`:

```scrippo
#! /usr/bin/env scrippo
```

The compiler ignores it. The operating system uses it when you execute the file directly. Autoload also treats a file whose first line contains `scrippo` after `#!` as a script, even when the filename does not end in `.scrippo`. The Neovim and VS Code extensions recognize `.scrippo`, `.scriptopotamus`, and a scrippo shebang.

## One line, several statements

`;` separates statements on one line. Quotes hide semicolons. This form works for `if`, `elif`, `else`, `for`, and `while`:

```scrippo
if $flag; -> "yes"; else -> "no"
for i in a b c; -> $i
```

A `case` header is not split this way. Its arms use the `pattern)` form described in [Control flow](/control-flow).

## What a line can be

The compiler classifies each statement in this order:

1. `@info`, which must be at indent 0. See [Metadata](/metadata).
2. An inline `;` sequence, unless the line is a `case` or a function header.
3. `if`, `for`, `while`, `case`.
4. `->`, which prints. See [Values and output](/values).
5. A `@` macro definition, macro call, or `@import`.
6. A function or command: a name or `.name` followed by `(`.
7. An assignment (`name=...` or `name<type>=...`) or a declaration (`name<type>`).
8. Anything else is bash, copied through with `$` references rewritten. See [Bash](/bash).

A function header may span lines until its parentheses balance. Parameters may be laid out one per line. A header may end with `->` and an expression, which becomes the whole body:

```scrippo
double(n<int>) -> $n * 2
```
