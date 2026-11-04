---
title: Registry
description: How bare names resolve, how to point scrippolib at another registry, and how to add to one.
---

A registry is what `scrippolib install shell` searches. It is a Git repository, or a plain directory, of `.scrippo` files that declare [`@info`](/metadata). The default registry is <https://github.com/michaelmunson/scrippolib-registry>.

## Use a registry

A bare name (letters, digits, `_`, `-`) is looked up in the configured registry. Both kinds are searched, and the entry with that name is installed, whichever kind it is.

```bash
scrippolib install shell
scrippolib info shell                 # what the registry has, before installing
scrippolib install -g ai
```

If a directory with the same name exists in the current directory, `scrippolib` asks you to write `./name` when you mean the directory.

## Change the registry

```bash
scrippolib configure                                   # print settings
scrippolib configure registry=https://example.com/my-registry.git
scrippolib configure registry=/path/to/a/directory
```

Settings live in `~/.scrippo/scrippolib.config` (`$SCRIPPO_HOME/scrippolib.config`), or at `SCRIPPOLIB_CONFIG`. The registry an entry was installed from is recorded in its manifest, so `update` keeps using it even after you change the setting.

## Layout

The layout is free-form; entries are found by their `@info`, not by path. The default registry uses:

```
packages/<name>/<name>.scrippo
scripts/<name>/<name>.scrippo
```

A file is an entry only if it has `@info` with `--name`, `--version`, and `--package` or `--script`. A registry file without `@info` is invisible to `install`, `info`, and `update`, which is the usual reason for `'<name>' was not found in registry`. The name must match the file stem. Hidden files and directories are skipped.

Installing snapshots the whole registry checkout, so an entry can `@import` another file in the registry by a path relative to itself (`@import ../ai/ai.scrippo`). Write the path from the entry's own directory: `@import ./ai.scrippo` inside `scripts/q/` looks for `scripts/q/ai.scrippo`. A package is compiled on the installer's machine in the usual sandbox, so it may import only files inside the checkout and the standard library.

## Versions

Registries are not tagged. An entry's version is its `@info --version` on the default branch, and `scrippolib update` installs it when that version is higher than the installed one. A local registry that is a Git repository serves only committed content.

## Add your own entry

1. Write the file with `@info`:

   ```scrippo
   @info
     --name is-even
     --version 1.0.0
     --description "Checks whether an integer is even"
     --package
   ```

2. Place it in the registry, for example `packages/is-even/is-even.scrippo`.
3. Commit and push to the default branch.
4. Others run `scrippolib install is-even`.

To ship a change, raise `--version` in the same commit.

A Git repository of your own, with tags, works without any registry: `scrippolib install https://github.com/you/helpers.git`. See [Publishing](/publishing).
