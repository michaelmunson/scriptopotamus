---
title: Installing
description: Install packages and scripts with scrippolib, from the registry, a Git repository, or a directory.
---

Package management is done by `scrippolib`, a separate command. `scrippo` compiles and runs scripts; `scrippolib` installs, updates, lists, and removes the packages and scripts they use.

```bash
scrippolib init                      # create .scrippo in this directory
scrippolib install shell             # from the registry
scrippolib install -g shell          # into ~/.scrippo
scrippolib install https://github.com/owner/helpers.git
scrippolib install ./helpers         # a plain directory
scrippolib update [name] [-g]
scrippolib uninstall <name> [-g]
scrippolib list [-g]
scrippolib info <name> [-g]
scrippolib configure registry=<url>
```

## Commands

| Command | Result |
| --- | --- |
| `init` | Create `.scrippo` in the current directory. Safe to repeat. It creates an environment here even if a parent already has one. |
| `install <name> [-g] [--tag VERSION]` | Install the packages and scripts a registry name, repository, or directory provides. |
| `update [<name>] [-g]` | Upgrade one install, or everything in the scope, to the newest version of the source it came from. |
| `uninstall <name> [-g]` | Remove an installed package or script. |
| `list [-g]` | List what is installed, with versions and origins. |
| `info <name> [-g]` | Show an installed entry's metadata, or the registry's entry when it is not installed. |
| `configure [registry=<url>]` | Print settings, or set them. |

`-g` / `--global` uses `~/.scrippo` (or `SCRIPPO_HOME`): packages in `lib/`, scripts in `scripts/`. Without it the scope is the project: packages in the nearest initialized project's `.scrippo/lib` (the global directory never counts as a project), scripts in the current directory.

## What `<name>` means

- A bare name such as `shell` is looked up in the configured registry.
- Anything else is a source: a Git URL, a path to a Git checkout, or a plain directory. Use `./helpers` for a directory whose name is also a valid registry name.

With a source, both kinds are installed: every `.scrippo` file whose `@info` declares `--package` or `--script` (skipped silently when a kind has no entries). A registry name installs the entry with that name, whichever kind it is.

## The registry

The registry is a Git repository (or a plain directory) of `.scrippo` files with `@info`. Layout is free-form; the default registry uses `packages/<name>/<name>.scrippo`. The default is <https://github.com/michaelmunson/scrippolib-registry>:

```bash
scrippolib configure                                  # print settings
scrippolib configure registry=https://example.com/my-registry.git
```

Settings live in `~/.scrippo/scrippolib.config` (override with `SCRIPPOLIB_CONFIG`). Registries are not tagged: an entry's version is its `@info --version`, taken from the default branch. Updates install the entry when that version is higher than the installed one. The registry an entry came from is recorded, so `update` keeps using it.

## Git and directory sources

Git sources need Git and Bash. The default tag is the highest stable `MAJOR.MINOR.PATCH` tag, with or without a `v`. Untagged commits and prereleases are ignored; a repository with no stable tag is an error. `--tag` selects an exact tag. `update` moves to the highest tag.

A plain directory (no `.git`) skips Git. With `--tag` omitted, every selected entry of a kind must share one `@info` version, which is recorded as the tag. The commit is `local`. The directory is remembered by absolute path so `update` works from anywhere, and updates when the entries' `@info` version rises.

Each entry's version must match the chosen tag, ignoring a leading `v`. Each `@info --name` must match the file stem. Names are unique per kind. Metadata options must be indented below `@info`. Hidden names (starting with `.`) are skipped.

## Packages and scripts

Packages are compiled in a sandbox to `<name>.bash`, checked with `bash -n`, and hard-linked into the destination. The sandbox allows imports from the repository and the standard library only. Repository symlinks and Git submodules are rejected. Package code is not executed by the compile.

Scripts keep the full source tree. `<name>.scrippo` is a relative symlink into that tree, so imports and `here` keep working. On Windows, creating that symlink requires the usual symlink permission.

## Where the bytes go

```
<destination>/
├── is-even.bash                         # or greet.scrippo
└── .scrippo-installed/packages/<id>/content/
    ├── source/
    ├── compiled/
    └── install.json
```

`install.json` records the repository, registry (if any), tag, commit, and each entry's metadata. `<id>` is the entry name for registry installs and the source basename (without `.git`) otherwise.

Existing public files are never overwritten; `install` points you at `update` when the name is already installed. Publication takes a directory lock, `.scrippo-install.lock`. Validation and compilation finish before anything is published, and a failed install deletes only what it created. A failed update restores the previous version. `uninstall` removes an entry's public file; when a source's last entry goes, its snapshot goes too. Files made with `scrippo -n` are not installs, so `uninstall` leaves them alone.

## Install hooks

An installable file can run its own logic while it is installed, with `@hook pre-install` and `@hook post-install`. See [Hooks](/hooks).

```scrippo
@info
  --name greet --version 1.0.0 --package

@hook pre-install
  -> "preparing"

@hook post-install
  -> "finished"
```

`pre-install` runs before anything is published. `post-install` runs after the public files exist. For each selected entry, in sorted path order, `scrippolib` runs that file's hooks in the order written. Each hook runs as a small script made from the file's `@import` lines plus the hook body, with the source snapshot as the working directory and `_SCRIPPO_FILE` set to the entry. Commands, variables, and functions in the file are not available to the hook. Hooks run on updates too.

A failing hook fails the install. A `pre-install` failure happens before publication. A `post-install` failure rolls the publication back. The hooks are not part of the compiled package or script.
