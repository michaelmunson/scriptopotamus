---
name: scrippo-tooling
description: >-
  Running, compiling and verifying Scriptopotamus code with the `scrippo` CLI:
  -c/--noexec, name resolution, config (autoload, autocomplete, alias, editor),
  shell integration and completion, environment variables, and compile or
  runtime errors. Use when checking a .scrippo file, debugging a compile error,
  or setting up aliases and shell hooks.
---

# Scrippo tooling

Reference: [docs/pages/docs](../../../../docs/pages/docs/) (`cli.md`, `compilation.md`, `configuration.md`, `shell.md`, `scripts.md`).

## Verify

```bash
scrippo -c path.scrippo         # compile to stdout (alias: --compile)
scrippo --noexec path.scrippo   # compile + bash -n
scrippo path.scrippo …          # compile and run
cargo test                      # when changing the compiler itself
```

Fix compile errors before adding features. Prefer reading golden fixtures under `tests/fixtures/` and `docs/examples/` over inventing syntax. `-c`, `--noexec`, `--configure`, `--hook`, `--env`, `--complete` are mutually exclusive modes.

## Running

* `scrippo [mode] [file] [args…]`; scrippo's own flags go **before** the file, everything after belongs to the script
* a path that exists is used as given; a bare `hello` / `hello.scrippo` resolves to the nearest `<name>.scrippo` walking up from cwd, then `~/.scrippo/scripts`
* a directory argument uses `<dir>/commands.scrippo`
* `scrippo <file> --help` prints the script's usage; no arguments prints scrippo's own usage and exits 1
* `scrippo` does not create/install/remove packages; that is `scrippolib` (see [packages](../packages/SKILL.md)). The old `-n -i -ls -rm` flags are gone

## What compilation does

Parse → import → macros (`@inherit`, unknown `@name` fails) → check (duplicate commands, const reassignment) → emit → run the temp bash with `_SCRIPPO_FILE` set to the source path.

* functions become `function name`; nested commands `function parent__child`; root is `_scrippo_root`; dispatch is a `case` on `$1` at the end
* top-level non-command statements run before dispatch, on every invocation
* libraries compiled for install skip the run step, dispatch, and top-level `--help`; a library with `.()` is rejected
* type checks test the *text* of values at assignment and parameter binding; they do not follow bash arithmetic or `eval`
* parse/check errors name file and line and produce no script; runtime errors (types, missing args, unknown command) go to stderr, usually with usage, exit 1

Common failures:

| Message / symptom | Cause |
|-------------------|-------|
| `unknown macro` / defining `@name()` | macros cannot be user-defined |
| `cannot reassign const 'X'` | assignment to a `const` |
| `@inherit requires a parent command or function` | `@inherit` at top level |
| a line that is exactly `fi`/`done`/`esac` | scrippo ends blocks by outdent |
| `expected int, received …` (runtime) | typed assignment or parameter mismatch |
| duplicate command compile error | same `.name` defined or imported twice at one level |
| `bash -n failed` | raw-bash lines are unchecked until `--noexec` |
| math fails at runtime | `bc` not installed |
| `here` fails | `realpath` not installed |

## Config

File: `~/.scrippo/config` (`$SCRIPPO_HOME/config`, or `SCRIPPO_CONFIG`). `key=value` lines.

```bash
scrippo --configure                                   # print
scrippo --configure autoload=true autocomplete=true
scrippo --configure alias=s,sc                        # alias= clears
scrippo --configure editor="nvim"                     # editor= disables
```

| Key | Default | Meaning |
|-----|---------|---------|
| `autoload` | `false` | entering a directory aliases its scripts and top-level project commands |
| `autocomplete` | `false` | tab completion for `scrippo`, `scriptopotamus`, aliases, autoload names |
| `alias` | empty | extra names that invoke `scrippo` |
| `editor` | empty | command `scrippolib new` uses to open new files (path appended as one quoted arg) |

Autoload: `package.scrippo` (or any file with a scrippo shebang) → alias `package`; top-level `.build` in `.scrippo/commands.scrippo` → alias `build` → `scrippo .scrippo/commands.scrippo build`; global scripts fill names still free. Nearest directory wins on clashes. Dotfiles are skipped except the project command file. `@info --autoload disable` opts a file out.

## Shell integration

```bash
eval "$(scrippo-core --init)"    # in ~/.bashrc or ~/.zshrc; bash and zsh only
```

On startup and on directory change the hook runs `eval "$(SCRIPPO_SHELL=<shell> scrippo --env)"`, which removes the previous aliases/completions and defines the new ones. Completion calls `scrippo --complete <file> -- <words>` and follows the command tree (commands, then options, then nested commands). Editing the config file alone does not change the current shell; open a new one or `cd`.

The optional `shell` package (`packages/shell/shell.scrippo`) exports `shell(text)`, which leaves text as editable input at the calling shell's next prompt; it needs the hook and an interactive shell.

## Environment

| Variable | Effect |
|----------|--------|
| `SCRIPPO_HOME` | replaces `~/.scrippo` |
| `SCRIPPO_CONFIG` | exact config path |
| `SCRIPPO_CORE` | core binary the wrapper execs |
| `SCRIPPO_SHELL` | `bash` / `zsh`, set by the hook |
| `_SCRIPPO_FILE` | source path of the running script |
| `NO_COLOR` | plain usage text |
| `SCRIPPOLIB_CONFIG` / `SCRIPPOLIB_CORE` | scrippolib config path / core binary |
