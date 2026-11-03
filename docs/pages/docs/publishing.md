---
title: Publishing
description: Lay out a repository so scrippolib can install it, or add it to a registry.
---

Publishing is a Git tag on a repository of `.scrippo` files. There is no build hook on the Git host. `scrippolib` clones the tag and reads `@info`. To make an entry installable by bare name, add it to a registry (see below).

## Layout

Put installable files anywhere. Nested directories are fine. Names must be unique for each kind (`--package` and `--script` are separate namespaces).

```
helpers/
├── is-even.scrippo
├── greet.scrippo
└── support/common.scrippo
```

`support/common.scrippo` has no `@info`, so it is a support file. Importers inside the repository can still `@import` it by relative path. Hidden files and directories (names starting with `.`) are not selected as entries. An entry can run its own `@hook pre-install` and `@hook post-install` blocks. See [Installing](/installing).

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

The installer selects the highest stable `MAJOR.MINOR.PATCH` tag, with or without a `v` prefix. Tags that are not exactly that shape, including prereleases, are ignored. Callers can pin a tag with `scrippolib install --tag v1.2.0`.

Every selected entry's `@info --version` must equal that tag. One repository tag is one version for every entry installed from it.

## What callers run

```bash
scrippolib install https://github.com/example/helpers.git
scrippolib install -g https://github.com/example/helpers.git
scrippolib update
```

After a package install, `@import is-even` works from the project or from the home lib, depending on `-g`. After a script install, `scrippo greet` runs the installed script when no local `greet.scrippo` is closer.

Ship the source. The installer compiles packages on the machine that runs `scrippolib install`.

## Publish to a registry

A registry is a Git repository of `.scrippo` files with `@info`; the default is [scrippolib-registry](https://github.com/michaelmunson/scrippolib-registry). Add your file (for example `packages/is-even/is-even.scrippo`) and commit it to the default branch. Registries are not tagged. Raising `@info --version` is what makes `scrippolib update` pick up a change. Then callers run:

```bash
scrippolib install is-even
```
