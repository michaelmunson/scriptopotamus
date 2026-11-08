---
name: scrippo-packages
description: >-
  Scriptopotamus packages, scripts, imports and distribution: @import resolution,
  .scrippo/lib layout, @info metadata, scrippolib (init/new/install/update/
  publish/registry), install hooks, and project vs global scope. Use when
  creating or importing a package, structuring a project's .scrippo directory,
  or publishing and installing scrippo code.
---

# Scrippo packages, imports, and scrippolib

Reference: [docs/pages/docs](../../../../docs/pages/docs/) (`imports.md`, `packages.md`, `scripts.md`, `layout.md`, `metadata.md`, `scrippolib.md`, `installing.md`, `publishing.md`, `registry.md`).

## Layout

| Kind | Path | Import / run |
|------|------|--------------|
| Project package | `.scrippo/lib/<name>.scrippo` | `@import <name>` |
| Global package | `~/.scrippo/lib/<name>.scrippo` | `@import <name>` (project wins) |
| Project commands | `.scrippo/commands.scrippo` | `scrippo .scrippo/commands.scrippo <cmd>` or autoload alias |
| Global script | `~/.scrippo/scripts/<name>.scrippo` | `scrippo <name>` |
| App script | `<name>.scrippo` | `scrippo <name>.scrippo …` (or just `scrippo <name>`) |

* `SCRIPPO_HOME` replaces `~/.scrippo`; the global dir never counts as a project
* the project root is the nearest ancestor containing a `.scrippo` **directory** (`scrippolib init` creates it); a legacy `.scrippo` *file* is a command tree and blocks local package commands until moved to `.scrippo/commands.scrippo`
* precedence: project package > global; `.scrippo` source > compiled `.bash`; script in cwd/parents > `~/.scrippo/scripts`
* a package is a `.scrippo` file meant to be imported: it contributes functions and named commands, not the importer's command line

## Imports

```scrippo
@import scrippo/std               # built-in stdlib (see ../stdlib/SKILL.md)
@import mylib                     # package: .scrippo/lib then ~/.scrippo/lib
@import ./relative.scrippo        # relative to the importing file, not cwd
@import ~/scripts/utils.scrippo   # ~/ expands from HOME
@import ./*                       # sorted glob, files only, each once
@import tools.scrippo --as .tools # commands nested under `tools`
@import tools.scrippo --as helper # wrapped in a function `helper`
```

* `@import` is compile-time only; imported functions/commands are pasted into the script, no runtime loader
* path is one literal word; `$` and command substitution are rejected; zero glob matches is an error
* a bare word (no `/` or `.`) is a package name; a missing package is a compile error naming both search dirs
* `.bash` / `.sh` imports are inserted raw
* unscoped imports merge top-level commands (duplicates are a compile error) **and run the imported file's top-level statements**; use packages or `--as` scopes for anything with a command tree
* `--as .name` makes a command scope (imported `.()` becomes its body); `--as name` makes a function scope
* may appear at top level or inside a function/command; each file is included once per scope; cycles are skipped
* macros are built into the compiler; scrippo cannot define its own, so use a function to reuse code

## Creating things

```bash
scrippolib init                       # create .scrippo in the current directory
scrippolib new package [-g] <name>    # .scrippo/lib/<name>.scrippo (-g: ~/.scrippo/lib)
scrippolib new script [-g] <name>     # ./<name>.scrippo (-g: ~/.scrippo/scripts)
```

Prefer these over hand-rolling directories. The starter file has an `@info` block (version `0.1.0`). Names: letter or `_`, then letters/digits/`_`/`-`. Existing files are never overwritten. If `editor` is configured, the new file opens in it.

Package skeleton:

```scrippo
@info
  --name mylib
  --version 1.2.0
  --description "Small helpers"
  --package

is_even(value<int>)
  [[ $((value % 2)) == 0 ]]
```

Script skeleton:

```scrippo
#! /usr/bin/env scrippo
@info
  --name greet
  --version 0.1.0
  --description "A greet script"
  --script

.()
  -> "root command"
```

Keep a package to one concern. Function names are pasted into the caller unnamespaced, so use distinctive names or tell callers to `--as`.

## @info

First statement in the file (a shebang and `#` comments may precede it), indent 0, options indented below it, one `@info` per file. Parentheses and inline forms are rejected. Emits no bash and is not available at runtime.

| Option | Value |
|--------|-------|
| `--name` | install name; must equal the file stem; letter/`_` then letters/digits/`_`/`-` |
| `--version` | `MAJOR.MINOR.PATCH`, no leading zeros |
| `--description` `--author` `--license` `--repository` | text |
| `--package` / `--script` | kind, mutually exclusive |
| `--autoload` | `enable` (default) or `disable` (no shell alias/completion for this file) |

Text values: bare word of `[A-Za-z0-9_.-+/:@]`, or quoted; double-quoted values cannot contain `"`, `$`, `` ` ``, `\`. Repeating or unknown options is an error. `@info` is optional for local files; installable files need kind, name and version.

## scrippolib

```bash
scrippolib install <name> [-g] [--tag vX.Y.Z]   # registry name, git URL, or ./directory
scrippolib update [<name>] [-g]
scrippolib uninstall <name> [-g]
scrippolib list [-g]
scrippolib info <name> [-g]
scrippolib configure registry=<url>             # default: github.com/michaelmunson/scrippolib-registry
```

* `-g` targets `~/.scrippo`; without it: packages go to the project's `.scrippo/lib` (needs `scrippolib init`), scripts to the cwd
* a bare name (`[A-Za-z0-9_-]`) is a registry lookup; for a local directory with that name write `./name`
* a Git URL or directory installs **every** `.scrippo` file whose top `@info` has `--name`, `--version`, and `--package` or `--script`; files without that are support files and skipped
* Git sources use the highest stable `MAJOR.MINOR.PATCH` tag (with or without `v`), `--tag` pins one; every entry's `@info --version` must equal the tag; registries/directories use `@info --version`
* packages are compiled in a sandbox (imports limited to the repo and stdlib; symlinks/submodules rejected; compile does not execute them) to `<name>.bash` and linked into `lib/`
* scripts keep their whole source tree; `<name>.scrippo` is a relative symlink into it so relative imports and `here` still work
* existing files are never overwritten; `install` points to `update`; failed installs roll back
* files made by `scrippolib new` are plain files, not installs; delete them directly

### Install hooks

```scrippo
@hook pre-install      # before publication; failure aborts
  -> "preparing"

@hook post-install     # after publication; failure rolls back
  -> "finished"
```

Top level of an installable file only, no arguments. Each runs as a small script made of the file's `@import` lines plus the hook body, with the source snapshot as cwd and `_SCRIPPO_FILE` set. The file's own functions, commands, and variables are not available. Left out of the compiled output.

### Publishing

1. put `.scrippo` files with `@info` in a Git repo (any directory structure; hidden paths skipped)
2. set each entry's `--version`, then `git tag vX.Y.Z && git push origin HEAD --tags`
3. consumers run `scrippolib install <git-url>`

For bare-name installs, commit the file to a registry's default branch (layout `packages/<name>/<name>.scrippo`, `scripts/<name>/<name>.scrippo`) and raise `--version` to ship changes; registries are not tagged. Inside a registry, import siblings with paths relative to the entry (`@import ../ai/ai.scrippo`). A file with no `@info` is invisible to `install`/`info`/`update` ("not found in registry").

## Workflows

**New project commands**

1. `scrippolib new package <lib>` if sharing helpers
2. put dispatchable commands in `.scrippo/commands.scrippo` (`.build()`, `.test()`, …)
3. `@import` libs from the commands file or app scripts
4. verify per [tooling](../tooling/SKILL.md)

**New package**

1. `scrippolib new package <name>` (or `-g`)
2. export functions and/or `.commands` (no `.()`)
3. consumers `@import <name>` then call

**New runnable script**

1. `scrippolib new script …` or create `app.scrippo` with a shebang
2. add `.commands` (or `.()`) for dispatch; see [commands](../commands/SKILL.md)
3. verify, then run `scrippo app.scrippo [cmd] [args]`
