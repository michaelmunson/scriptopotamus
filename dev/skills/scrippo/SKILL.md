---
name: scrippo
description: >-
  Write Scriptopotamus (.scrippo) scripts, packages, commands, and imports that
  compile to bash. Use when creating or editing .scrippo files, .scrippo/lib
  packages, .scrippo/commands.scrippo, @import usage, the standard library
  (prt, input, config…), or scrippo/scrippolib CLI packaging. Routes to nested
  skills for syntax, commands, stdlib, packages, and tooling.
---

# Writing Scrippo

Scrippo compiles to bash. Prefer scrippo syntax over raw bash unless a construct has no scrippo form yet.

Reference docs live in [docs/pages/docs](../../../docs/pages/docs/) and runnable samples in [docs/examples](../../../docs/examples/). Read them, and the golden fixtures in `tests/fixtures/`, before inventing syntax.

## Nested skills

Read the one(s) that match the task. Each is self-contained; load more than one when the task spans areas.

| Skill | Read it when… |
|-------|---------------|
| [syntax](syntax/SKILL.md) | writing any script body: variables and types, lists/dicts, math, `->`, `if`/`for`/`while`/`case`, functions, raw bash |
| [commands](commands/SKILL.md) | the script takes arguments or subcommands: `.cmd()` trees, `.()` root, parameter grammar, `#?` usage, `@usage`, `@inherit`, `@hook exec`/`exit` |
| [stdlib](stdlib/SKILL.md) | printing styled output, prompting, failing with `throw`, `list`/`dict`/`str` helpers, `here`, `config` (JSON) |
| [packages](packages/SKILL.md) | `@import` rules, `.scrippo/lib` layout, `@info`, `scrippolib` (new/install/update), install hooks, publishing, registries |
| [tooling](tooling/SKILL.md) | compiling and checking (`-c`, `--noexec`), debugging errors, `scrippo --configure`, autoload/aliases, shell hook, env vars |

Typical combinations:

* new CLI script → syntax + commands (+ stdlib)
* project `commands.scrippo` → commands + packages (layout) + tooling (autoload)
* reusable helper library → syntax + packages
* "why does this not compile" → tooling, then the construct's skill

## House style

* match surrounding file indentation (examples use 2 spaces; tabs are fine)
* no `#` comments unless the user asks or a non-obvious invariant needs one; `#?` usage docs on commands and parameters are not comments
* prefer typed declarations (`name<type>=…`) over bare bash assigns
* use `->` instead of `echo` for output
* end blocks by outdenting, never `fi` / `done` / `esac`
* keep bodies short; extract helpers or packages when a script grows
* use the standard library (`prt`, `input`, `throw`) before hand-rolling bash

## Cheat sheet

```scrippo
#! /usr/bin/env scrippo
@import scrippo/std

PI<const float>=3.14
items<list>=(a b c)

#? Greet someone
.greet(name [--loud])
  if $loud
    prt -s bold -s green "HELLO $name"
  else
    -> "hello $name"

.math()
  .add(a<int> b<int>)
    -> $a + $b
```

```bash
scrippo --noexec app.scrippo           # compile + bash -n
scrippo app.scrippo greet ada --loud   # run
scrippolib new package mylib           # start a package (needs scrippolib init)
```

## Rules that bite

* inside a body use the bare option name (`$select`, `$dry_run`); `--dry-run` becomes `dry_run`
* `-> $a + $b` is math only with spaces around the operator; it needs `bc`
* macros are built into the compiler (`@import @info @usage @inherit @hook`); users cannot define their own, so reuse code with a function
* a package cannot contain `.()`; root commands belong in scripts
* unscoped `@import` of a script pastes its top-level statements into the importer
* `scrippo` runs and compiles; `scrippolib` creates and manages packages and scripts

## Default workflow

1. pick the nested skill(s) above
2. use `scrippolib new …` instead of hand-creating package/script files
3. write the code; keep it in scrippo syntax
4. `scrippo --noexec <file>`, then run the command you changed
5. when the compiler or language itself changes, follow [AGENTS.md](../../../AGENTS.md): update `docs/pages`, the VS Code extension, and these skills in the same change
