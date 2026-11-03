---
title: Imports
description: Compile-time @import of the standard library, packages, files, and globs.
---

`@import` copies another program into the file being compiled. There is no runtime loader. By the time bash runs, every imported function and command is already in the script.

```scrippo
@import scrippo/std
@import mylib
@import ./relative.scrippo
@import ~/scripts/utils.scrippo
@import ./* --as .tools
```

The path is one literal word. Quotes are allowed. `$` and command substitution in the path are rejected. An optional scope is exactly `--as` and a name:

```scrippo
@import projects/project-one.scrippo --as .project-one
@import projects/project-one.scrippo --as project-one-helper
```

A scope name starts with a letter or `_` and may contain digits, `_`, and `-`. A leading `.` makes the scope a command. Without the dot, the scope is a bash function.

## Standard library

These paths are built into the compiler:

| Path | Also accepted |
| --- | --- |
| `scrippo/std` | `scrippo/std.scrippo`, `scrippo/*`, `scrippo/std/*` |
| `scrippo/std/io` | `scrippo/io`, `scrippo/std/io.scrippo` |
| `scrippo/std/types` | `scrippo/types`, `scrippo/std/types.scrippo` |
| `scrippo/std/path` | `scrippo/path`, `scrippo/std/path.scrippo` |

`scrippo/std` is io, then types, then path. The helpers are documented in [Standard library](/standard-library).

## Packages

A path with no `/` and no `.` is a package name. Lookup order:

1. `<project>/.scrippo/lib/<name>.scrippo`
2. `<project>/.scrippo/lib/<name>.bash`
3. `~/.scrippo/lib/<name>.scrippo` (or `$SCRIPPO_HOME/lib`)
4. `~/.scrippo/lib/<name>.bash`

The project is the nearest ancestor directory that contains a `.scrippo` directory, starting from the file that contains the import, excluding the global directory (`~/.scrippo` or `SCRIPPO_HOME`). Bare `scrippo -i` creates a project environment in the current directory. Global imports still work without a project environment. A `.scrippo` source package wins over a compiled `.bash` of the same name, and a project package wins over a global one.

A missing package is a compile error that names both search directories.

While a package is being compiled for installation, this lookup is replaced by a sandbox: the importing file's directory, the repository root, `packages/`, and `.scrippo/lib/` inside that repository, and the standard library. The installer's own project and home are not visible. See [Installing](/installing).

## Files and globs

Anything that is not a standard-library path or a package name is a filesystem path.

- `~/...` expands from `HOME`.
- An absolute path is used as given.
- A relative path is relative to the file that contains the `@import`, not the process's current directory.
- `*`, `?`, and `[...]` in the final path component are a glob. The match is that one directory, files only, sorted by name. Directories are skipped. Zero matches is an error.

`.bash` and `.sh` files are inserted as raw bash ahead of the script. Other files are parsed as scrippo and imported recursively. Each file is included once per scope. Importing a file that is already on the stack is skipped, so a cycle does not loop.

## Scopes

`--as .name` wraps the imported statements in a command called `name`. `--as name` wraps them in a function called `name`. Top-level commands from the imported file become subcommands of a command scope, and completion follows that tree:

```bash
scrippo app.scrippo project-one version --short
```

A root command `.()` in the imported file becomes the body and parameters of the scope itself. Two root commands in one scope are an error.

Without `--as`, imported top-level commands join the importer's command list. A repeated command name is a compile error. The same file may be imported once into each scope; each scope has its own inclusion set.

## When imports run

`@import` may appear at the top level or inside a function or command. Nested imports are processed while that body is compiled. `@info` in an imported file is metadata for that file and emits no code.

Prefer packages and scoped imports once a file grows a command tree. An unscoped import of a script that has top-level statements will run those statements as part of the importer, because they are pasted into the program.
