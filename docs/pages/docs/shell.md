---
title: Shell integration
description: The bash and zsh hook, aliases, and tab completion.
---

Shell integration is a snippet you eval from your shell startup file. It reloads aliases when the working directory changes, and it installs completions when `autocomplete` is on.

## Enable it

Turn the settings on, then load the hook:

```bash
scrippo --configure autoload=true autocomplete=true
```

The compiler binary prints the snippet:

```bash
scrippo-core --init
```

Add the evaluation to `~/.bashrc` or `~/.zshrc`:

```bash
eval "$(scrippo-core --init)"
```

`scrippo --configure` prints the reminder `eval "$(scrippo --init)"`. The `scrippo` wrapper declares this mode as `--hook` and forwards its arguments to `scrippo-core`. The core flag that prints the snippet is `--init`. The snippet supports bash and zsh. Any other shell gets an error from the snippet.

## What the hook does

On startup, and again when you change directory, the hook runs:

```bash
eval "$(SCRIPPO_SHELL=bash scrippo --env)"
```

`SCRIPPO_SHELL` is `zsh` under zsh. `--env` prints shell code that:

1. Removes aliases and completions the previous load defined.
2. Defines each configured alias (`alias s=scrippo` and the rest) .
3. Defines an alias for each discovered script and project command. See [Configuration](/configuration).
4. When autocomplete is on, registers a completion function for `scrippo`, `scriptopotamus`, the configured aliases, and the discovered names.

Bash remembers the last directory and skips the reload when `PWD` has not changed, using `PROMPT_COMMAND`. Zsh uses `chpwd`. Zsh also sets `COMPLETE_ALIASES` so completion runs on the alias, and it runs `compinit` if completion was requested and `compdef` is missing.

Open a new shell after editing the startup file. In bash, a later `cd` also refreshes aliases.

## Completion behavior

The completion function calls:

```bash
scrippo --complete <file> -- <words typed so far>
```

For `scrippo` itself, the file is the wrapper's own declaration. Candidates are the flags and script names that fit the words already typed. Once a script path is among those words, later candidates come from that script: command names, then that command's options, then nested commands.

A project command alias such as `build` completes as if you had typed `scrippo .scrippo/commands.scrippo build`. Options of `build` are offered after the alias, and nested commands are offered as the next word.

`@info(--autoload disable)` removes a file from this discovery. The commands still complete when you invoke `scrippo` on the file directly.

## Managed names

The hook records the aliases it created and deletes those names before defining a new set. Aliases you defined yourself, under other names, are left in place. A name that collides with an autoload alias is redefined on each refresh while you remain in a directory that provides it.
