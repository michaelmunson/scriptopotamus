---
title: Parameters
description: Arguments, options, aliases, groups, exclusive sets, defaults, repeats, and rest.
---

Functions and commands share one parameter grammar. The text inside the parentheses is the grammar. It may span lines. `#` comments and `#?` documentation comments are allowed inside it.

```scrippo
.serve(
  path
  [port<int=8080>]
  [--bind<str="0.0.0.0">]
)
  -> "$path $port $bind"
```

## Arguments and options

A word that starts with `-` is an option. Any other word is a positional argument. An option with no type and no default is `bool`. An argument with no type is `str`.

| Form | Meaning |
| --- | --- |
| `path` | Required string argument. |
| `count<int>` | Required int argument. |
| `[path]` | Optional string argument. |
| `path=notes` | Argument defaulting to `notes`. |
| `path<str="notes">` | Typed argument with a default. |
| `--loud` | Required bool option. |
| `[--loud]` | Optional bool option. Absent means `false`. |
| `-v<str>` | Required string option. The caller writes `-v value`. |
| `[--mode<str {a b c}>]` | Optional string option limited to those words. |
| `--items<str>...` | Repeatable string option. Values collect into an array until the next flag. |
| `files...` | Repeatable argument. The remaining positionals are the array. |
| `*rest` | Everything after the fixed positionals, kept verbatim. |

Brackets make a parameter optional. A default also supplies a value when the caller omits it. Repeatable parameters (`...`) are optional in the sense that zero occurrences are allowed, and they bind an array.

Bool options are `true` when present. Their default is `false` unless you write one, as in `[--loud<bool=true>]`.

## Names in the body

The variable for an option is the first flag with leading dashes removed and `-` turned into `_`.

| Declaration | Variable |
| --- | --- |
| `--loud` | `$loud` |
| `--dry-run` | `$dry_run` |
| `-v<str>` | `$v` |
| `-s / --select` | `$s`, and `$select` as a nameref to `$s` |

`$--loud` and `$-v` are rewritten to those variable names, so `$--loud` and `$loud` are the same read. Later flags in an alias list become bash namerefs, which is why both spellings work.

## Aliases

`/` joins flags that are the same option. It applies only when both sides are flags.

```scrippo
.files([-ls / --list | -rm / --remove])
  if $list
    -> "listing"
  if $remove
    -> "removing"
```

`-ls`, `--list` set the same bool. The variable is `ls` (the first flag), and `list` is a nameref. `|` is exclusive and is not an alias. A `/` with one side missing is a compile error.

## Groups

`[` and `]` group parameters. A group of one marks that parameter optional. A larger group is optional as a whole: its members are optional, and required parameters nested where the grammar demands them are checked only when the group is used.

Usage indents the group's later parameters under its first option. That is how `scrippo --help` shows `--tag` under `--install`.

## Exclusive sets

`|` separates alternatives. The caller may supply one of them.

```scrippo
.ask(
  [
    --select<str>...
    [--multi]
  ] | [
    --confirm
  ]
)
  -> "ok"
```

If every alternative is optional, the caller may supply none. If none of them is optional, one of them is required and the error is `one of ... is required`. Supplying two alternatives fails with `X and Y are mutually exclusive`. Nested exclusive sets are checked on their own, so `[[-a | -b] | [-c]]` rejects both `-a -b` and `-a -c`.

## Rest

`*name` is an array. It must be the last parameter you care about. After every fixed positional is filled, the remaining words are appended as written, options included. `--` is kept as a word.

```scrippo
.cmd(file [-v] *rest)
  -> "$v|$file|${rest[*]}"
```

`cmd -v a -v --x -- b` binds `v=true`, `file=a`, and `rest` as `-v`, `--x`, `--`, `b`. Words before the positional are still parsed as options. Unknown options before the positional is filled go into the rest array when a rest parameter exists. Without `*rest`, an unknown option is an error.

`--` ends option parsing. The words after it are positionals. With `*rest`, `--` itself is one of the rest words.

## Checks

Missing required parameters, extra positionals, unknown options, mutual exclusion, and type mismatches print the function or command name, the message, and usage, then exit 1. Empty optional values skip the type pattern. Each element of a repeatable or list parameter is checked.

`--help` is recognized before user options and returns from the function after printing usage, unless `--help` is in the parameter list.
