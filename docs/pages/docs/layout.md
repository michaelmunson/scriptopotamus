---
title: Project layout
description: Where packages, scripts, commands, and config live on disk.
---

Scrippo looks in the project directory and in a home directory. `SCRIPPO_HOME` replaces `~/.scrippo` everywhere that home directory is used.

## A project

```
project/
├── app.scrippo
├── .scrippo/
│   ├── commands.scrippo
│   └── lib/
│       └── mylib.scrippo
└── .scrippo-installed/
```

| Path | Role |
| --- | --- |
| `.scrippo/lib/<name>.scrippo` | Project package. `@import <name>` finds it first. |
| `.scrippo/lib/<name>.bash` | Compiled package installed into the project. |
| `.scrippo/commands.scrippo` | Project command tree. Autoload aliases each top-level command. |
| `.scrippo` as a file | Legacy command tree. Same role as `commands.scrippo`. |
| `<name>.scrippo` | A script in the project. `scrippo <name>` finds the nearest one walking upward. |
| `.scrippo-installed/` | Snapshots and manifests from `scrippolib`. Created when you install. |

The project root is the nearest ancestor of the current file or directory that contains a `.scrippo` directory. The global directory (`~/.scrippo` or `SCRIPPO_HOME`) never counts as a project. Run `scrippolib init` in your project root to create `.scrippo`. Local package commands require this environment and create `lib` inside it as needed.

If `.scrippo` is a file, local package commands stop and ask you to move that file to `.scrippo/commands.scrippo`. The file form still works as the command entry for autoload and for `scrippo` invoked on that directory.

## The home directory

```
~/.scrippo/
├── config
├── lib/
│   ├── <package>.scrippo
│   └── <package>.bash
└── scripts/
    └── <script>.scrippo
```

| Path | Role |
| --- | --- |
| `config` | `autoload`, `autocomplete`, and `alias`. Override the file with `SCRIPPO_CONFIG`. |
| `lib/` | Global packages. Used when the project has no package of that name. |
| `scripts/` | Global scripts. `scrippo <name>` uses them when no local `<name>.scrippo` exists. |

An older config at `~/.config/scrippo/config` is still read when `SCRIPPO_CONFIG` is unset and `~/.scrippo/config` does not exist yet. New writes go to `~/.scrippo/config`.

## Who wins

- Project packages win over global packages.
- A `.scrippo` source file in a lib directory wins over a `.bash` file with the same stem.
- A script in the current directory or a parent wins over `~/.scrippo/scripts`.
- Autoload aliases: the nearest directory wins when two scripts or commands share a name. Global script aliases are added after project ones, so they fill in names that are still free.

## Installed snapshots

`scrippolib install` stores the source contents under the destination:

```
<destination>/.scrippo-installed/<packages|scripts>/<id>/content/
├── source/
├── compiled/          # packages
└── install.json
```

Public names are hard links (packages, `<name>.bash`) or relative symlinks (scripts, `<name>.scrippo`) from the destination back into that snapshot. See [Installing](/installing).
