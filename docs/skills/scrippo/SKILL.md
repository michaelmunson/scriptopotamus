---
name: scrippo
description: >-
  Write Scriptopotamus (.scrippo) scripts, packages, commands, and imports that
  compile to bash. Use when creating or editing .scrippo files, .scrippo/lib
  packages, .scrippo/commands.scrippo, @import usage, or scrippo CLI packaging.
---

# Writing Scrippo

Scrippo compiles to bash. Prefer scrippo syntax over raw bash unless a construct has no scrippo form yet.

When this skill is referenced, also skim [docs/pages](../../pages/) and [docs/examples/](../../examples/) if unsure about a construct.

## Style

* match surrounding file indentation (examples use 2 spaces; tabs are fine)
* no comments unless the user asks or a non-obvious invariant needs one
* prefer typed declarations (`name<type>=…`) over bare bash assigns
* use `->` instead of `echo` for output
* end blocks with indentation, never `fi` / `done` / `esac`
* keep bodies short; extract helpers or packages when a script grows

## Layout

| Kind | Path | Import / run |
|------|------|----------------|
| Project package | `.scrippo/lib/<name>.scrippo` | `@import <name>` |
| Global package | `~/.scrippo/lib/<name>.scrippo` | `@import <name>` (local wins) |
| Project commands | `.scrippo/commands.scrippo` | `scrippo .scrippo/commands.scrippo <cmd>` or autoload alias |
| Global script | `~/.scrippo/scripts/<name>.scrippo` | `scrippo <name>` |
| App script | `<name>.scrippo` | `scrippo <name>.scrippo …` |

Create with CLI (prefer this over hand-rolling dirs):

```bash
scrippolib new package [-g] <name>   # package; local unless -g (needs scrippolib init)
scrippolib new script [-g] <name>    # script; local unless -g
```

Install and manage packages and scripts with `scrippolib` (a separate command):

```bash
scrippolib init                       # create .scrippo in the current directory
scrippolib install <name> [-g]        # <name>: registry name, git URL, or directory (use ./dir for a directory)
scrippolib update [<name>] [-g]
scrippolib uninstall <name> [-g]
scrippolib list [-g]
scrippolib info <name> [-g]
scrippolib configure registry=<url>   # default: https://github.com/michaelmunson/scrippolib-registry
```

Local package management requires a project `.scrippo` directory; create it with `scrippolib init`. Global storage (`~/.scrippo` or `SCRIPPO_HOME`) is excluded from project lookup; `-g` installs there. A bare name is looked up in the registry; a Git URL or directory installs every `.scrippo` file under it whose top `@info` block declares `--name`, `--version`, and `--package` or `--script` on indented lines (files without a matching `@info` are support files and skipped). Git sources use the highest stable version tag (`--tag` pins one); registries and directories use `@info --version`. An entry file may declare `@hook pre-install` (runs before publish) and `@hook post-install` (runs after); they run as scripts built from the file's `@import` lines plus the hook body.

Optional shebang on runnable scripts:

```scrippo
#! /usr/bin/env scrippo
```

`SCRIPPO_HOME` overrides `~/.scrippo`. Project packages always resolve from the project `.scrippo/lib/` first.

## Core syntax

### Variables & types

```scrippo
n<int>=1
s<str>="hi"
flag<bool>=true
PI<const float>=3.14
items<list>=(a b c)
map<dict>=(a=1 b=2)
choice<str {patch minor major}>
any_val<any>
```

Index with `$list[0]` / `$dict[key]` (compiles to bash `${…}` forms). Arithmetic uses `$a + $b` (emits `bc` when needed).

### Control flow

```scrippo
if $flag
  -> "yes"
else
  -> "no"

if $flag; -> "yes"; else -> "no"

for i in $items
  -> $i

while $n -lt 3
  n=$((n + 1))

case $choice
  patch | minor) -> "ok"
  *) -> "?"
```

### Functions vs commands

* `name(…)` — function (call as `name args`)
* `.name(…)` — top-level **command** (dispatch: `scrippo script.scrippo name …`)
* nest `.sub` under a command for subcommands

```scrippo
circumference(diameter<float>) -> $PI * $diameter

.greet(name [--loud])
  if $loud
    -> "HELLO $name"
  else
    -> "hello $name"

.math()
  .add(a<int> b<int>)
    -> $a + $b
```

### Params

* `[arg]` / `[--opt]` optional
* `|` mutually exclusive groups
* `...` repeatable
* default type: args `str`, opts `bool`
* defaults with `=`
* typed opts: `--flag<str="default">`, `--mode<str {a b c}>`

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
  else
    -> "nope"
```

Inside the body, use the bare name (`$select`, `$options`) — not `$--select`.

### Imports

```scrippo
@import scrippo/std          # io + types + path
@import scrippo/std/io
@import mylib                # .scrippo/lib/mylib.scrippo then ~/.scrippo/lib/
@import ./relative.scrippo
@import ~/scripts/utils.scrippo # leading ~/ expands from HOME, also inside quotes
@import ./*                  # sorted glob; dirs skipped; each file once
```

* `@import` is compile-time only; no runtime package loader
* top-level commands from imports merge into the script; duplicate command names are a compile error
* macros are built into the compiler; scrippo cannot define its own, so use a function to reuse code

### Hooks

```scrippo
@hook exec                         # before any command
  echo "script"

.cmd()
  @hook exit code=CODE             # after cmd and its subcommands
    if $CODE != 0
      echo "failed: $CODE"
```

* `@hook exec` runs before a command; `@hook exit` runs after it with `code` (exit status, an int) available
* `@hook pre-install` / `@hook post-install` (top level only) run while the file is installed or updated with `scrippolib`
* `exec` / `exit` are allowed only at the top level or directly inside a command; `exec` hooks run outermost first, `exit` hooks innermost first

## Standard library

Prefer `@import scrippo/std` for scripts that print, prompt, or touch paths.

| Import | Helpers |
|--------|---------|
| `scrippo/std/io` | `throw code msg…`, `prt [-s style…]… [-j sep] [-n] msg…`, `input prompt -v var [--confirm \| --select … [--multi]]` |
| `scrippo/std/types` | `list name [item] [--join sep\|--length\|--reverse\|--contains]`, `dict name [key] [--keys\|--values\|--length\|--contains]`, `str name [text] [--length\|--split\|--reverse\|--contains]` |
| `scrippo/std/path` | `here [--file] [--rel]` — script dir/path, optional cwd-relative |
| `config` (own import, not in `scrippo/std`) | `config [--init id\|path.json [--force]] [key \| key=value…] [--assign key:VAR…]` — JSON config via `jq` |

`prt` styles: `bold dim italic underline black red green yellow blue magenta cyan white`.

## Workflows

### New project commands

1. `scrippolib new package <lib>` if sharing helpers
2. Put dispatchable commands in `.scrippo/commands.scrippo` (`.build()`, `.test()`, …)
3. `@import` libs from commands or app scripts as needed
4. Verify: `scrippo --noexec .scrippo/commands.scrippo` then `scrippo .scrippo/commands.scrippo <cmd> …`

### New package

1. `scrippolib new package <name>` (or `-g` for global)
2. Export functions and/or `.commands`
3. Consumers: `@import <name>` then call
4. Keep packages focused; one concern per file

### New runnable script

1. `scrippolib new script …` or create `app.scrippo` with shebang
2. Add `.commands` if the script should dispatch subcommands
3. `scrippo --noexec app.scrippo` then `scrippo app.scrippo [cmd] [args]`

## Verify

```bash
scrippo -c path.scrippo         # compile to stdout
scrippo --noexec path.scrippo   # compile + bash -n
scrippo path.scrippo …          # compile and run
cargo test                      # when changing the compiler itself
```

Fix compile errors before adding features. Prefer reading golden fixtures under `tests/fixtures/` and `docs/examples/` over inventing syntax.
