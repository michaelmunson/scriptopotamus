---
name: scrippo-commands
description: >-
  Writing Scriptopotamus command-line interfaces: `.command()` trees, the root
  command, parameter grammar (args, options, defaults, repeats, groups, exclusive
  sets, ref, *rest), `#?` usage docs, @usage, @inherit, and @hook exec/exit. Use
  when a .scrippo script or .scrippo/commands.scrippo takes arguments or
  subcommands.
---

# Scrippo commands and parameters

A command is a function whose name starts with `.`; the dot is not typed by the user. Reference: [docs/pages/docs](../../../../docs/pages/docs/) (`commands.md`, `parameters.md`, `usage.md`, `inherit.md`, `hooks.md`). General syntax is in [syntax](../syntax/SKILL.md).

## Commands

```scrippo
#? Greet someone
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
```

* top-level statements run on every invocation, then a `case` on `$1` dispatches
* unknown command → error, command list, usage, exit 1; duplicate command names (including via `@import`) are a compile error
* a parent with only nested commands prints usage when called alone; a parent with its own statements runs them
* every command accepts `--help` (prints usage, exit 0) unless it declares `--help` itself
* `--help` skips hooks of the scope it names

### Root command

`.()` is the script's own behavior. Defined once at indent 0, cannot contain nested commands.

```scrippo
.(value<int>)
  -> $value
```

* with parameters: every invocation that is not another top-level command goes to `.()`, including no arguments
* with `.()` empty: runs only when there are no arguments
* **a package cannot contain `.()`**; put it in a script (`--script`)

## Parameters

Functions and commands share one grammar; text inside `( )` may span lines and contain `#` / `#?` comments.

| Form | Meaning |
|------|---------|
| `path` | required string arg |
| `count<int>` | required typed arg |
| `[path]` | optional arg |
| `path=notes` / `path<str="notes">` | default |
| `--loud` | required bool option |
| `[--loud]` | optional bool; absent = `false` |
| `-v<str>` | required string option (`-v value`) |
| `[--mode<str {a b c}>]` | option limited to a literal set |
| `[--n<int=2>]` | option with default |
| `--items<str>...` | repeatable option, collects into an array |
| `files...` | repeatable arg |
| `*rest` | everything after fixed positionals, verbatim (options included, `--` kept) |
| `items<ref list>` | caller passes a variable name; parameter is a nameref |
| `-s / --select` | alias; first flag is the variable, others are namerefs |
| `[-a] \| [-b]` | exclusive set; supplying two is an error |
| `[ … ]` | group (optional as a whole) |

* a word starting with `-` is an option, anything else is a positional; default types: arg `str`, option `bool`
* options become variables with leading dashes dropped and `-` → `_`: `--dry-run` → `$dry_run`. `$--dry-run` is the same read; prefer the bare name
* `/` aliases only join flags; `|` is exclusion, not alias
* `--` ends option parsing; with `*rest` it is kept as a rest word
* `ref` is one parameter: `items<ref list>...` and `*rest<ref>` are errors. Do not name the caller's variable the same as the parameter (circular nameref warning)
* missing/extra args, unknown options, exclusion clashes, and type mismatches print the message plus usage and exit 1

```scrippo
choose(
  path
  [optional_arg<int>]
  -v<str>
  [
    -s / --select
    --options<str>...
    [--multi]
  ] | [
    --confirm
  ]
)
  if $select
    for i in $options
      -> $i
```

## Usage text

* `#?` directly above a command/function documents it; `#?` above or after a parameter documents that parameter (key = first token). Several for one key join with `; `
* comments above the closing `)` of a multiline header stay on the command
* `@usage` prints the enclosing command's usage and exits 1; `@usage 0` exits 0; `@usage --no-exit` continues; any other argument is a compile error
* usage prints `{name}` for required and `[name]` for optional, non-default types after the name, `one of:` for exclusive sets. `NO_COLOR` disables color

## @inherit

Copies the parent's parameter list into a nested command; extra parameters may be added before/after. Needs an enclosing function or command.

```scrippo
.path(path [--count<int=2>])
  .rel(@inherit)
    -> $path
  .abs(@inherit [--verbose])
    -> $path
```

`@inherit` is a built-in macro. You cannot define macros; use a function to reuse code.

## Hooks

```scrippo
@hook exec                     # before any command
  echo "script"

.cmd()
  @hook exit code=CODE         # after cmd and its subcommands
    if $CODE != 0
      echo "failed: $CODE"
```

* `exec` runs before, `exit` after; `exit` exposes `code` (an int) via `-a code=NAME` or just `code=NAME`
* allowed only at the top level or directly inside a command (including `.()`), never in a function, `if`, or loop
* `exec` runs outermost first, `exit` innermost first; several hooks of one name run in written order
* an `exit` hook's own status is ignored; unknown-command and `@usage` exits skip `exit` hooks
* `@hook pre-install` / `post-install` are top-level and run only during `scrippolib install`; see [packages](../packages/SKILL.md)
