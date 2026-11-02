---
title: Installing
description: Install packages and scripts from a Git repository or a directory.
---

`scrippo -i` copies installable `.scrippo` files from a Git repository or a directory into a project or into `~/.scrippo`.

```bash
scrippo -i https://github.com/owner/helpers.git
scrippo -i -p https://github.com/owner/helpers.git
scrippo -i -p -g https://github.com/owner/helpers.git
scrippo -i -s -g https://github.com/owner/helpers.git
scrippo -i -p https://github.com/owner/helpers.git lib
scrippo -i -s https://github.com/owner/helpers.git . ./tools
scrippo -i -p ./helpers
scrippo --install --package --tag v1.2.0 https://github.com/owner/helpers.git
```

## Arguments

```
scrippo -i [-p | -s] [-g] [--tag VERSION] <source> [directory] [destination]
```

Flags come before the source. Words after the source are a directory inside the source, then a destination. They are not flags.

| Piece | Meaning |
| --- | --- |
| source | Git URL, path to a Git checkout, or plain directory. |
| `-p` / `--package` | Install packages only. |
| `-s` / `--script` | Install scripts only. |
| neither | Install both. A kind with no entries is skipped. |
| `-g` / `--global` | Destination is `$SCRIPPO_HOME/lib` or `.../scripts`. |
| `--tag` | Exact tag, `1.2.0` or `v1.2.0`. |
| directory | Subdirectory to scan. Default is the source root. Use `.` to keep the root when you also pass a destination. |
| destination | Directory that receives the public names. Requires `-p` or `-s`. Cannot be combined with `-g`. |

Default destinations, when you omit both `-g` and an explicit path:

- Packages go to the nearest project's `.scrippo/lib`, or `./.scrippo/lib` if there is no project yet.
- Scripts go to the current directory.

## What gets selected

The installer walks the scan root and keeps `.scrippo` files whose `@info` kind matches the request. Files with no `@info`, files of the other kind, and the legacy `@info name version` form are support files and are skipped. Hidden names (starting with `.`) are skipped. Names must be unique per kind.

Git sources need Git and Bash. The default tag is the highest stable `MAJOR.MINOR.PATCH` tag. Untagged commits and non-stable tags are ignored. A repository with no stable tag is an error unless you are installing a plain directory.

A plain directory (no `.git`) skips Git. With `--tag` omitted, every selected entry must share one `@info` version, and that version is recorded as the tag. The commit field is `local`.

Each entry's version must match the chosen tag, ignoring a single leading `v`. Each `@info --name` must match the file stem.

## Packages and scripts

Packages are compiled in a sandbox to `<name>.bash`, checked with `bash -n`, and hard-linked into the destination. The sandbox allows imports from the repository and the standard library only. Repository symlinks and Git submodules are rejected. Package code is not executed by the compile.

Scripts keep the full source tree. `<name>.scrippo` is a relative symlink into that tree, so imports and `here` keep working. On Windows, creating that symlink requires the usual symlink permission.

## Where the bytes go

```
<destination>/
├── is-even.bash                         # or greet.scrippo
└── .scrippo-installed/packages/helpers/content/
    ├── source/
    ├── compiled/
    └── install.json
```

`install.json` records the repository, tag, commit, and each entry's metadata. The repository directory name is the source basename without `.git`.

Existing public files are never overwritten. An existing snapshot for that repository name is never overwritten. Install a different version to a different destination. Publication takes a directory lock, `.scrippo-install.lock`. A failed install deletes the public files and snapshot it created. Validation and compilation finish before anything is published.

## Install hooks

If the source root contains `.install.scrippo`, it runs twice: phase `before` before publication, and phase `after` after the public files exist. The phase is the first argument and the environment variable `SCRIPPO_INSTALL_PHASE`. The working directory is the source snapshot.

Top-level statements run on both passes. Put phase-specific work in `.before()` and `.after()`:

```
-> "runs before and after"

.before()
  -> "prepare"

.after()
  -> "finish"
```

Empty `.before` and `.after` commands are added when you omit them, so the hook's command dispatch has somewhere to go. A failing hook fails the install. The `before` failure happens before publication. The `after` failure rolls the publication back.
