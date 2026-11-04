---
title: Configuration
description: autoload, autocomplete, and command aliases in ~/.scrippo/config.
---

Settings live in `~/.scrippo/config`, or in `$SCRIPPO_HOME/config`. `SCRIPPO_CONFIG` replaces that path entirely. If `SCRIPPO_CONFIG` is unset, `~/.scrippo/config` does not exist, and `~/.config/scrippo/config` does, the legacy file is read. Saves always write the current config path.

## Print and set

```bash
scrippo --configure
scrippo --configure autoload=true autocomplete=true
scrippo --configure alias=s,sc
scrippo --configure alias=
scrippo --configure editor="cursor"
scrippo --configure editor="nvim"
```

With no pairs, the command prints the file it would save. With pairs, it updates those keys, writes the file, and prints the result. Unknown keys are an error. The file is `key=value` lines. Blank lines and `#` comments are ignored.

| Key | Values | Default |
| --- | --- | --- |
| `autoload` | `true`, `false`, `1`, `0`, `yes`, `no`, `on`, `off` | `false` |
| `autocomplete` | same booleans | `false` |
| `alias` | comma-separated command names, or empty | empty |
| `editor` | editor command, or empty to disable | empty |

Alias names start with a letter or `_` and contain letters, digits, `_`, or `-`. Duplicates are dropped. `alias=` clears the list.

Turning `autoload` on prints a reminder to eval the shell hook. The hook is what actually defines aliases. Editing the config file alone does not change the current shell. See [Shell integration](/shell).

## autoload

With `autoload=true`, entering a directory defines aliases from that directory and its parents. The nearest directory wins on a name clash. Then global scripts fill in names that are still unused.

| Found | Alias | Expands to |
| --- | --- | --- |
| `package.scrippo`, or any file with a scrippo shebang | `package` | `scrippo <path>` |
| Top-level `.build` in `.scrippo/commands.scrippo` | `build` | `scrippo <path> build` |
| `~/.scrippo/scripts/hello.scrippo` | `hello` | `scrippo <path>` |

Dotfiles are skipped, except the project command file (`.scrippo/commands.scrippo` or a legacy `.scrippo` file). A file whose `@info` says `--autoload disable` contributes no alias. `--autoload enable` is the default and still requires the global `autoload` setting to be true.

Command aliases from `commands.scrippo` are only the top-level commands. Nested commands stay behind the parent word, and completion offers them after the alias.

## autocomplete

With `autocomplete=true`, the hook installs completion for:

- `scrippo` and `scriptopotamus`
- each name in `alias`
- each autoload alias

Completion of `scrippo` itself follows the wrapper's modes. After you choose a mode, candidates that cannot combine with it disappear: `scrippo -c` does not offer `--noexec`. After a script name, completion switches to that script's commands and options.

## alias

`alias=s,sc` makes `s` and `sc` invoke `scrippo`, with the same completion when autocomplete is on. These names are applied by the hook in new shells, and in bash after a directory change because the hook re-runs. They are shell aliases, not separate binaries.

## editor

After creating a local or global script or package, `scrippolib new` opens its new `.scrippo` file with this command. For example, `scrippo --configure editor="nvim"` followed by `scrippolib new script -g myscript` opens the global script in Neovim.

The command runs through Bash with the file path appended as one quoted argument. Arguments and quoted executable paths are supported, for example `scrippo --configure editor="cursor --wait"`. Scrippo inherits the terminal and waits for the editor command to finish. A launch failure or nonzero exit reports an error and preserves the created file. Existing files are never opened or overwritten by `scrippolib new`.

Use `scrippo --configure editor=` to disable opening. An unset editor also leaves creation without opening a program.
