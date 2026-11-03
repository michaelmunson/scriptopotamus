---
title: Packages
description: Create, import, and compile a scrippo package.
---

A package is a `.scrippo` file meant to be imported. It contributes functions and commands to the importer. It does not become the importer's command line.

## Create one

```bash
scrippolib init
scrippo -n -p mylib
scrippo -n -p -g mylib
```

`-p` / `--package` selects packages. Without `-g`, the file is `.scrippo/lib/mylib.scrippo` in the nearest initialized project. Run `scrippolib init` to create the project environment first; the global directory never counts as a project. `-g` writes `~/.scrippo/lib/mylib.scrippo`. The new file contains a comment with the package name. Replace that with the functions you want to export.

Installed packages are managed with [scrippolib](/installing): `scrippolib list`, `scrippolib uninstall <name>`. Files you create with `scrippo -n` are plain files; delete them directly.

## Write one

```scrippo
@info
  --name mylib
  --version 1.2.0
  --description "Small helpers"
  --package

is_even(value<int>)
  [[ $((value % 2)) == 0 ]]
```

`@info` is optional for a package you only use locally. It is required when the file should be installed from Git. The name must match the filename without `.scrippo`. See [Metadata](/metadata) and [Publishing](/publishing).

Export functions and, when useful, named commands (`.greet`). A root command `.()` is rejected when the file is compiled as an installable package. Put `.()` in a script.

## Import one

```scrippo
@import mylib

if is_even 4
  -> "even"
```

Lookup is the project lib, then the global lib, source file before compiled `.bash`. Details and `--as` scopes are in [Imports](/imports).

## Compiled packages

Installing a package compiles it to `<name>.bash` with imports inlined, runs `bash -n`, and exposes that file through `@import <name>`. The compiled form defines bash functions for the package's functions and named commands. It does not install a command dispatcher and does not respond as the importer's `--help`.

Compilation of an installed package does not execute the package. Imports are limited to the downloaded repository and the standard library. Symlinks and submodules in the repository are rejected before that compile. This is a compiler sandbox, not an operating-system sandbox. Install hooks are the exception: `@hook pre-install` and `@hook post-install` blocks are executed. See [Installing](/installing).

Keep a package to one concern. Callers depend on the function names, which are pasted into their script and are not namespaced unless they import with `--as`.
