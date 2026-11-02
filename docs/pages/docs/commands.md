---
title: Commands
description: CLI commands, nesting, the root command, and dispatch.
---

A command is a function whose name starts with `.`. The dot is not part of the name the user types. Commands are how a script takes subcommands on its command line.

```
.greet(name [--loud])
  if $loud
    -> "HELLO $name"
  else
    -> "hello $name"

.math()
  .add(a<int> b<int>)
    -> $a + $b
  .mul(a<int> b<int>)
    -> $a * $b
```

```bash
scrippo app.scrippo greet ada --loud
scrippo app.scrippo math add 2 3
scrippo app.scrippo math --help
```

## Dispatch

Top-level commands are bash functions. After the script's top-level statements run, a `case` on `$1` selects one of them and shifts it off `"$@"`.

- A known command name runs that command.
- An unknown first argument prints `unknown command`, lists the command names, prints usage, and exits 1.
- No argument at all prints usage and exits 1 when the script has commands and has no root command that accepts an empty invocation.

Nested commands repeat the process. `math add` runs `.math`, which sees `add` and runs `.add`. If the parent body is only nested commands, invoking it with no further word prints usage and exits 1. If the parent also has its own statements, that invocation runs those statements instead. An unknown nested name prints `unknown command` and usage when the parent has no parameters to accept the word.

Duplicate command names in the same command list are a compile error, including duplicates brought in by `@import`.

## The root command

`.()` is the root command. It is the behavior of the script itself.

```
#? Root documentation
.(value<int>)
  -> $value
```

```bash
scrippo app.scrippo 42
scrippo app.scrippo --help
```

Rules for `.()`:

- It is defined once, at indent 0.
- Commands cannot be nested inside it.
- With parameters, every invocation that is not some other top-level command goes to `.()`, including an invocation with no arguments.
- With an empty parameter list, `.()` runs only when there are no arguments. Any word is an unknown command unless it names another top-level command.

A package compiled for install cannot contain `.()`. Put executable root commands in scripts (`@info --script`). A package may still export named commands; they become bash functions the importer can call, and they do not replace the importer's own command dispatch. See [Packages](/packages).

## Help

Every command accepts `--help` unless its parameter list declares `--help`. Declared `--help` is a normal bool option and your body decides what it does.

```
.cmd([--help])
  -> "custom $--help"
```

`scrippo app.scrippo cmd --help` then prints `custom true`. Help found later in the argument list, as in `cmd oops --help`, prints usage and skips the type check that would have rejected `oops`.

## Scope and completion

Completion follows the command tree. After `scrippo app.scrippo math`, the candidates are `add` and `mul`. After a command name, the candidates are that command's options. Shell integration and editor completion both read these declarations. See [Shell integration](/shell).

Import a file's commands under a prefix with `@import file --as .name`. The imported commands are then nested under `name` and do not appear as top-level commands of the importer. See [Imports](/imports).

## Project command files

`.scrippo/commands.scrippo` is the conventional command tree for a project. With autoload on, each top-level command becomes a shell alias. A legacy layout uses a file named `.scrippo` in the project root for the same role. See [Project layout](/layout) and [Scripts](/scripts).
