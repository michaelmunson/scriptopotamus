---
title: scrippolib
description: The package manager for scrippo. Create, install, update, list, and remove packages and scripts.
---

`scrippo` compiles and runs scripts. `scrippolib` is the separate command that manages the packages and scripts they use: it creates new ones, and installs, updates, lists, and removes them. It replaces the old `scrippo -n`, `-i`, `-ls`, and `-rm`, which no longer exist.

```bash
scrippolib init                       # create .scrippo in this directory
scrippolib new package mylib          # start a package
scrippolib new script greet           # start a script
scrippolib install shell              # from the registry
scrippolib install -g shell           # into ~/.scrippo
scrippolib install https://github.com/owner/helpers.git
scrippolib install ./helpers          # a plain directory
scrippolib update                     # everything installed here
scrippolib list
scrippolib info shell
scrippolib uninstall shell
scrippolib configure registry=<url>
scrippolib --help
scrippolib --version
```

## Get it

`scrippolib` is built the same way as `scrippo`. It is a scrippo script, [`cli/scrippolib.scrippo`](https://github.com/michaelmunson/scrippolib/blob/main/cli/scrippolib.scrippo), compiled to a bash wrapper. The wrapper checks arguments, prints usage, and execs `scrippolib-core`, the Rust binary that does the work. The two sit side by side.

From a clone of [`scrippolib`](https://github.com/michaelmunson/scrippolib), next to a clone of `scriptopotamus` (the wrapper is compiled by `scrippo-core`, and the core links the compiler as a library):

```bash
make build         # cargo build, then compile the wrapper to target/debug/scrippolib
make test
make install       # scrippolib and scrippolib-core into /usr/local/bin
```

Set `SCRIPPOLIB_CORE` to run a different core binary. `scrippolib install` of a Git repository needs Git and Bash.

## Commands

| Command | Result |
| --- | --- |
| `init` | Create `.scrippo` in the current directory. Safe to repeat. It creates an environment here even if a parent already has one. |
| `new package <name> [-g]` | Create `.scrippo/lib/<name>.scrippo` (`-g`: `~/.scrippo/lib`). |
| `new script <name> [-g]` | Create `<name>.scrippo` in the current directory (`-g`: `~/.scrippo/scripts`). |
| `install <name> [-g] [--tag VERSION]` | Install the packages and scripts that a registry name, repository, or directory provides. |
| `update [<name>] [-g]` | Upgrade one install, or everything in the scope, to the newest version of the source it came from. |
| `uninstall <name> [-g]` | Remove an installed package or script. |
| `list [-g]` | List what is installed, with versions and origins. |
| `info <name> [-g]` | Show an installed entry's metadata, or the registry's entry when it is not installed. |
| `configure [registry=<url>]` | Print settings, or set them. |
| `-V` / `--version` | Print the version. |

Every command takes `--help`.

## Scopes

`-g` / `--global` works on `~/.scrippo` (or `SCRIPPO_HOME`): packages in `lib/`, scripts in `scripts/`. Without it the scope is the project:

- packages go to the nearest initialized project's `.scrippo/lib`. The global directory never counts as a project, so run `scrippolib init` in the project root first.
- scripts go to the current directory.

`update`, `uninstall`, `list`, and `info` look only in the scope you name. See [Project layout](/layout) for how `scrippo` finds each of these afterwards.

## Create packages and scripts

`new package` and `new script` write a starter file and, when an [editor is configured](/configuration#editor), open it. Each file starts with an `@info` block, so it can be installed as it is: the name is the file name, the version is `0.1.0`, and the kind is `--package` or `--script`. A script is a shebang, the `@info`, and a root command `.()`:

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

A package is the same `@info` with `--package`, followed by a comment where its functions go. Edit the description, and raise the version when you publish a change.

Names start with a letter or `_` and then use letters, digits, `_`, or `-`. An existing file is never overwritten or opened. If the editor cannot launch or exits with an error, you get an error and the new file is kept.

The files are plain files. Delete them directly; `uninstall` only removes things that `install` put there. To share one, [publish it](/publishing); the `@info` block is described on [Metadata](/metadata).

## A typical workflow

Write a helper library and use it from a project:

```bash
cd myproject
scrippolib init
scrippolib new package strings        # edit .scrippo/lib/strings.scrippo
```

```scrippo
@import strings
```

Use something somebody else published:

```bash
scrippolib install shell              # a registry name
scrippolib install -g ai              # a script, available as `scrippo ai` everywhere
scrippolib update                     # later, pick up new versions
```

Share your own: put the files in a Git repository with `@info`, tag a version, and others run `scrippolib install <url>`; or add them to a [registry](/registry) so a bare name works. See [Publishing](/publishing).

## Environment

| Variable | Effect |
| --- | --- |
| `SCRIPPO_HOME` | Replaces `~/.scrippo` for the global scope and the config path. |
| `SCRIPPOLIB_CONFIG` | Exact path of the `scrippolib` config file. |
| `SCRIPPOLIB_CORE` | Core binary the wrapper execs. |
| `SCRIPPO_CONFIG` | `scrippo`'s config file, which holds the `editor` that `new` opens. |

## Next

- [Installing](/installing): sources, versions, and exactly what is written to disk.
- [Registry](/registry): bare names, and how to run or publish to one.
- [Publishing](/publishing): lay out a repository so it can be installed.
