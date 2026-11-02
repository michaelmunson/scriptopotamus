---
title: Usage text
description: Documentation comments, --help, @usage, and how usage is formatted.
---

Usage is generated from the command tree, the parameter grammar, and `#?` comments. You do not write a separate help string for the shape of the command. You write the prose.

## Documentation comments

`#?` immediately above a function or command documents that entry. `#?` above a parameter, or on its line, documents that parameter. The key is the parameter's first token. Several comments for the same key are joined with `; `.

```scrippo
#? Resolve a path
.get-path(
  path #? Path to resolve
  [--verbose] #? Print details
)
  -> $path
```

Comments above the closing `)` of a multiline header stay attached to the command, not to a parameter.

## What the user sees

`scrippo script.scrippo --help` and `scrippo script.scrippo cmd --help` print usage and exit 0. A function's own `--help` prints its usage and returns.

A missing command, a missing argument, a bad type, or an exclusive-option clash prints the error on stderr, then usage, then exits 1.

The layout:

- The first line is `Usage:`.
- The command path is next, in green on a terminal, followed by ` # ` and the command's `#?` text.
- Required arguments are `{name}`. Optional parameters are `[name]`.
- Options show every alias: `[-ls / --list]`.
- A type that is not the default (`str` for arguments, `bool` for options) is printed after the name: `{count}<int>`.
- Repeatable parameters keep `...`.
- A group's later parameters are indented under the first.
- An exclusive set is introduced by `one of:`, and each alternative is a bullet.

Colors are used when stdout is a terminal and `NO_COLOR` is unset. Set `NO_COLOR` to any value, including empty, for plain text.

## @usage

`@usage` prints the usage for the command or function that contains it.

```scrippo
.hello(name)
  .world(@inherit)
    @usage 0
```

| Form | Behavior |
| --- | --- |
| `@usage` | Print usage and exit 1. |
| `@usage 0` | Print usage and exit with that status. The code is an integer. |
| `@usage --no-exit` | Print usage and continue. |
| `@usage 7 --no-exit` | The code is ignored when `--no-exit` is set, because the process does not exit. |

Any other argument is a compile error: `@usage: expected [code<int=1>] [--no-exit]`.

`@usage` always describes the enclosing function or command, which already includes inherited parameters and nested commands. It is the way a command shows help and then stops, including help that should exit 0.

## Declaring --help

A parameter named `--help` disables the automatic help action for that function. Your body receives a bool. The script's top-level automatic help is disabled the same way when the root command `.()` declares `--help`.

Automatic help on a script with no root command still runs when `$1` is `--help`, before command dispatch.
