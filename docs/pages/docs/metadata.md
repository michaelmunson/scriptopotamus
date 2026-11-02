---
title: Metadata
description: The @info declaration for name, version, kind, and autoload.
---

`@info` describes the file. It emits no bash. Ordinary scripts may omit it. A file that `scrippo -i` should install needs it.

## Placement

`@info` is the first statement in the file. A shebang and `#` comments may precede it, because those are skipped. It sits at indent 0. A second `@info`, or an `@info` inside a function, is a compile error.

Two spellings parse. The parenthesized form is the one install understands:

```
@info(
  --name is-even
  --version 1.2.0
  --description "Checks whether an integer is even"
  --author "Ada Lovelace"
  --license MIT
  --repository "https://github.com/example/helpers"
  --package
  --autoload enable
)
```

The legacy form is `@info <name> <version> [<description>]`. It sets those three fields and no kind, so the installer skips the file.

## Options

| Option | Value |
| --- | --- |
| `--name` | Install name. Letters, digits, `_`, `-`, starting with a letter or `_`. |
| `--version` | `MAJOR.MINOR.PATCH`. Digits only, no leading zeros, exactly three numbers. |
| `--description` | Text. |
| `--author` | Text. |
| `--license` | Text. |
| `--repository` | Text, usually a Git URL. |
| `--package` | This file is a package. |
| `--script` | This file is a script. |
| `--autoload` | `enable` (default) or `disable`. |

`--script` and `--package` are mutually exclusive. Repeating any option is an error. Unknown options are an error. Every option is optional for a file you compile yourself. Installable entries need a kind, a name equal to the filename without `.scrippo`, and a version equal to the selected tag (ignoring a leading `v` on the tag).

Options may share a line or span lines. A trailing `#` comment is allowed. Text values are literals: a bare word of letters, digits, and `_.-+/:@`, or a single-quoted or double-quoted string. Double-quoted values cannot contain `"`, `$`, `` ` ``, or `\`. Shell interpolation is rejected.

## Autoload

`--autoload disable` removes the file from shell aliases and from the completions those aliases would offer. Commands inside a disabled `.scrippo/commands.scrippo` are not aliased. Running the file with `scrippo` and importing it still work.

`--autoload enable` follows the global `autoload` setting. With `autoload=false` in config, no file is aliased.

## What is stored

The installer writes every field into `install.json` inside the snapshot: name, version, description, author, license, repository, kind, autoload, and the source path. Metadata is not available as a variable at runtime.
