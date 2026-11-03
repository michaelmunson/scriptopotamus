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

`--autoload disable` in the indented `@info` block removes a file from this discovery. The commands still complete when you invoke `scrippo` on the file directly.

## Managed names

The hook records the aliases it created and deletes those names before defining a new set. Aliases you defined yourself, under other names, are left in place. A name that collides with an autoload alias is redefined on each refresh while you remain in a directory that provides it.

## Editable command input

The repository's `packages/shell/shell.scrippo` package exports `shell(text)`.
Copy it to your project's `.scrippo/lib/shell.scrippo` (or your global
`~/.scrippo/lib/shell.scrippo`), and enable the hook above. Then:

```scrippo
@import shell

text='echo "Hello"'
shell "$text"
```

After the script finishes, the calling shell shows `echo "Hello"` as editable
input. Edit it and press Enter to run it in that shell; the accepted command is
added to its history, subject to the shell's history settings. Commands such as
`cd` and variable assignments therefore affect the calling shell.

The hook creates a private temporary file and exports its path as
`SCRIPPO_SHELL_INPUT` to child scripts. `shell` only writes text; it does not
execute it. The next prompt consumes the file. The last call before that prompt
wins. Without the hook, `shell` reports an error and returns status 1.

Zsh places the text in its normal editor buffer. Bash uses a Readline prompt
before the next normal prompt, records accepted input, and evaluates it in the
calling shell. Press Ctrl-D to discard the Bash input. Handoffs require an
interactive shell; they are not consumed in batch runs. Temporary handoff files
are empty after consumption and may be removed after closing the shell.
