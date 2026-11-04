---
title: Scripts
description: Runnable scripts, name resolution, project commands, and global scripts.
---

A script is a `.scrippo` file you run. It may define a root command, a command tree, or only top-level statements.

## Create one

```bash
scrippolib new script greet
scrippolib new script -g greet
```

A local script is created in the current directory:

```scrippo
#! /usr/bin/env scrippo
.main()
  -> "hello from greet"
```

With `-g`, the script is created in `~/.scrippo/scripts/` instead. You can also create the file yourself; `scrippolib new` only writes the starter text. Installing, listing, and removing scripts is also `scrippolib`: see [scrippolib](/scrippolib).

## Run one

```bash
scrippo greet.scrippo
scrippo greet
scrippo greet.scrippo world
./greet.scrippo world
```

Resolution for a single path component (`greet` or `greet.scrippo`):

1. `<name>.scrippo` in the current directory, then in each parent directory.
2. `~/.scrippo/scripts/<name>.scrippo`, then `~/.scrippo/scripts/<name>` if that file exists.

A path that already points at a file is used as given. A directory is used only when `<dir>/commands.scrippo` exists; otherwise the argument is an error. Local files win over global scripts. Tab completion for `scrippo <name>` uses the same search.

Arguments after the script name are the script's arguments. `scrippo greet --loud` passes `--loud` to `greet`. Flags that belong to `scrippo` itself (`-c`, `--noexec`, `--configure`, and the rest) go before the script name.

## Project commands

`.scrippo/commands.scrippo` holds the commands for a repository: `.build()`, `.test()`, and so on. Run them explicitly:

```bash
scrippo .scrippo/commands.scrippo build
```

With autoload, each top-level command is an alias, so `build` runs `scrippo .scrippo/commands.scrippo build`. A file named `.scrippo` (not a directory) is the legacy place for those commands and is still discovered.

`--autoload disable` in the indented `@info` block at the top of a script or of `commands.scrippo` omits aliases and completions for that file. The file still runs and can still be imported.

## Installed scripts

`scrippolib install` installs scripts by keeping the repository source tree and linking `<name>.scrippo` at the destination. Relative imports, `here`, package lookup, and command completion keep working because the link points into the snapshot. Links are relative, so a project install can move with the project. Creating the link on Windows requires permission to create symlinks.

An installed script's `@info --name` must match its filename. The snapshot layout is described in [Installing](/installing#where-the-bytes-go).
