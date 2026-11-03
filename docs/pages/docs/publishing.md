---
title: Publishing
description: Lay out a repository so scrippo -i can install it.
---

Publishing is a Git tag on a repository of `.scrippo` files. There is no registry and no build hook on the Git host. The installer clones the tag and reads `@info`.

## Layout

Put installable files anywhere. Nested directories are fine. Names must be unique for each kind (`--package` and `--script` are separate namespaces).

```
helpers/
├── is-even.scrippo
├── greet.scrippo
├── support/common.scrippo
└── .install.scrippo
```

`support/common.scrippo` has no `@info`, so it is a support file. Importers inside the repository can still `@import` it by relative path. Hidden files and directories (names starting with `.`) are not selected as entries. `.install.scrippo` is a hook, not an entry. See [Installing](/installing) for the hook's phases.

## Mark each entry

The first statement is `@info` with a kind, a name, and a version. The name matches the file stem. The version matches the Git tag you are about to push, without a leading `v`.

```scrippo
@info
  --name is-even
  --version 1.2.0
  --description "Checks whether an integer is even"
  --author "Ada Lovelace"
  --license MIT
  --repository "https://github.com/example/helpers"
  --package

is_even(value<int>)
  [[ $((value % 2)) == 0 ]]
```

A script uses `--script` and may contain `.()`. A package uses `--package` and exports functions or named commands. Full field rules are on [Metadata](/metadata).

## Tag and push

```bash
git tag v1.2.0
git push origin HEAD --tags
```

The installer selects the highest stable `MAJOR.MINOR.PATCH` tag, with or without a `v` prefix. Tags that are not exactly that shape, including prereleases, are ignored. Callers can pin a tag with `--tag v1.2.0`.

Every selected entry's `@info --version` must equal that tag. One repository tag is one version for every entry installed from it.

## What callers run

```bash
scrippo -i https://github.com/example/helpers.git
scrippo -i -p https://github.com/example/helpers.git
scrippo -i -s -g https://github.com/example/helpers.git
```

After a package install, `@import is-even` works from the project or from the home lib, depending on `-g`. After a script install, `scrippo greet` runs the installed script when no local `greet.scrippo` is closer.

Ship the source. The installer compiles packages on the machine that runs `scrippo -i`.
