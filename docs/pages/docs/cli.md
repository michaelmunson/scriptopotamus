---
title: Command line
description: Flags for compiling, running, packages, scripts, configuration, and completion.
---

The `scrippo` command is itself a scrippo script. Its options, nesting, and exclusivity are the usage text and the completion candidates. Put scrippo's own flags before the script or source path. Everything after a script filename belongs to that script.

```bash
scrippo [mode] [file] [args...]
```

With no arguments, `scrippo` prints usage and exits 1.

## Run and compile

| Invocation | Result |
| --- | --- |
| `scrippo <file> [args...]` | Compile and run. Arguments after the file are the script's. |
| `scrippo -c <file>` | Compile to stdout. |
| `scrippo --compile <file>` | Same as `-c`. |
| `scrippo --noexec <file>` | Compile and syntax-check with `bash -n`. |
| `scrippo -h` / `scrippo --help` | Print wrapper usage and exit 0. |
| `scrippo -V` / `scrippo --version` | Print the compiler version. |
| `scrippo <file> --help` | Run that file's usage. `--help` is passed through once the filename is present. |

`-c`, `--noexec`, `-n`, `--configure`, `--hook`, `--env`, and `--complete` are mutually exclusive modes.

A filename that exists is used as given. A bare name such as `hello` or `hello.scrippo` resolves to the nearest `<name>.scrippo` walking upward from the current directory, then to `~/.scrippo/scripts`. A directory argument uses `<dir>/commands.scrippo` when that file exists. Details are on [Scripts](/scripts).

## Create

New scripts and packages open automatically when an [editor is configured](/configuration#editor), for both local and global creation. Set it with `scrippo --configure editor="cursor"` or `scrippo --configure editor="nvim"`.

`-n` needs `-p` / `--package` or `-s` / `--script`. `-g` / `--global` uses `~/.scrippo` (or `SCRIPPO_HOME`) instead of the project.

| Invocation | Result |
| --- | --- |
| `scrippo -n -p <name>` | Create `.scrippo/lib/<name>.scrippo`. |
| `scrippo -n -s <name>` | Create `<name>.scrippo` in the current directory, with a shebang and a `.main` command. |
| `scrippo -n -p -g <name>` | Create `~/.scrippo/lib/<name>.scrippo`. |
| `scrippo -n -s -g <name>` | Create `~/.scrippo/scripts/<name>.scrippo`. |

`-n` is `--new`. Names start with a letter or `_` and then use letters, digits, `_`, or `-`.

Local package creation requires a project environment. Run `scrippolib init` in your project root to create `.scrippo`; commands in descendants then use its `.scrippo/lib`. The global directory is excluded from project lookup. If `.scrippo` is a legacy command file rather than a directory, create the directory layout first. See [Project layout](/layout).

## Install, update, list, remove

Package management moved out of `scrippo`. `scrippolib install`, `update`, `uninstall`, `list`, `info`, and `init` replace `scrippo -i`, `-ls`, and `-rm`. See [Installing](/installing).

## Configuration and the shell

| Invocation | Result |
| --- | --- |
| `scrippo --configure` | Print `autoload`, `autocomplete`, and `alias`. |
| `scrippo --configure autoload=true autocomplete=true` | Update those keys and print the result. |
| `scrippo --configure alias=s,sc` | Extra command names for the wrapper. `alias=` clears them. |
| `scrippo --env` | Print aliases and completions for the current directory. |
| `scrippo --complete <file> [words...]` | Print completion candidates. A leading `--` before the words is ignored. |
| `scrippo --hook` | Wrapper mode for the shell startup snippet. |

The core binary prints that startup snippet for `--init`:

```bash
scrippo-core --init
```

`scrippo --configure` reminds you to add `eval "$(scrippo --init)"` after autoload is turned on. The snippet the core emits calls `scrippo --env` when the directory changes. See [Configuration](/configuration) and [Shell integration](/shell).

## Environment

| Variable | Effect |
| --- | --- |
| `SCRIPPO_HOME` | Replaces `~/.scrippo` for packages, scripts, and the default config path. |
| `SCRIPPO_CONFIG` | Exact path of the config file. |
| `SCRIPPO_CORE` | Core binary the wrapper execs. |
| `SCRIPPO_SHELL` | `bash` or `zsh`, set by the hook when it reloads `--env`. |
| `_SCRIPPO_FILE` | Source path of the running script. |
| `NO_COLOR` | Usage is printed without ANSI color when this variable is set. |
| `HOME` | Used for `~` in imports and for the default `SCRIPPO_HOME`. |
