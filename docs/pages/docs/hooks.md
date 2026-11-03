---
title: Hooks
description: "@hook runs your logic around commands and around installation."
---

`@hook` is a built-in [macro](/macros) that wraps your logic around command execution. It takes the name of a hook, optional flags, and an indented body. The body is ordinary scrippo.

Four hooks exist. `exec` runs before a command and `exit` runs after it. `pre-install` and `post-install` run while the file is installed.

## @hook exec and @hook exit

```scrippo
@hook exec
  echo "calling script"

.cmd()
  @hook exec
    echo "calling command"

  @hook exit -a code=CODE
    if $CODE != 0
      echo "Command failed with code $CODE"
    else
      echo "Command succeeded"

  -> "cmd output"

  .subcmd()
    @hook exec
      echo "calling subcommand"
    -> "subcmd output"
```

```bash
scrippo cmds.scrippo cmd
# calling script
# calling command
# cmd output
# Command succeeded

scrippo cmds.scrippo cmd subcmd
# calling script
# calling command
# calling subcommand
# subcmd output
# Command succeeded
```

| Form | Runs |
| --- | --- |
| `@hook exec` at the top level | Before any command is executed. |
| `@hook exec` inside a command | Before that command's own logic, and before any of its subcommands. |
| `@hook exit` at the top level | After any command has finished. |
| `@hook exit` inside a command | After that command, or one of its subcommands, has finished. It is scoped to that command. |

A command's hooks run around the whole command, including the subcommands under it. `exec` hooks run outermost first: script, command, subcommand. `exit` hooks unwind innermost first: subcommand, command, script.

A scope may have several hooks of the same name. They run in the order written.

## Where exec and exit may be written

- At the top level of a file.
- Directly inside a command (`.name()`), including the root command `.()`.

A hook inside a plain function, an `if`, a loop, or any other block is a compile error. A top-level hook needs at least one command in the script to wrap, and a hook cannot be used in a library. A top-level hook brought in by `@import` joins the importing script's hooks.

## Passing variables to the body

`@hook` takes `-a / --args`, a list of hook variables to expose to the body. Each entry is `variable` or `variable=name`. `variable=name` renames it inside the body. The flag word may be left out, so `@hook exit code=CODE` means the same as `@hook exit -a code=CODE`.

| Hook | Variables |
| --- | --- |
| `exec` | none |
| `exit` | `code`: the exit status of the command, as an integer |

`code` is an integer, so test it with a comparison. `if $code` alone is a [bool](/control-flow) test and is never true for a number. Use `if $code != 0` for failure.

## Install hooks

`@hook pre-install` and `@hook post-install` run when the file is installed with `scrippo -i`, not when it runs. They are written at the top level of an installable file (one with `@info`). They take no arguments.

```scrippo
@info
  --name greet --version 1.0.0 --package

@hook pre-install
  -> "preparing"

@hook post-install
  -> "finished"
```

`pre-install` runs before publication, and `post-install` runs after it. A failure in `pre-install` stops the install. A failure in `post-install` rolls it back. Each hook runs as its own script made from the file's `@import` lines and the hook body, so the file's commands and variables are not available. See [Installing](/installing#install-hooks).

An install hook inside a command is a compile error. Install hooks are left out of the compiled script or package.

## What hooks do not do

- A scope's hooks are skipped when its first argument is `--help`. `script --help` runs no hooks. `script cmd --help` runs the script's hooks but not `cmd`'s.
- A script exits immediately on an unknown command or on `@usage` without `--no-exit`. Those exits skip `exit` hooks.
- An `exit` hook's own exit status is ignored. The command's status is kept.
