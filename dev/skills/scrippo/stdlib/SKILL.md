---
name: scrippo-stdlib
description: >-
  The Scriptopotamus standard library: `prt`, `input`, `throw`, `list`, `dict`,
  `str`, `here`, and `config`, and how to import them. Use when a .scrippo script
  prints styled output, prompts the user, exits with an error, inspects lists,
  dicts or strings, finds its own path, or reads and writes JSON config.
---

# Scrippo standard library

Built into the compiler; no install needed. Reference: [docs/pages/docs/standard-library.md](../../../../docs/pages/docs/standard-library.md). Sources: [stdlib/](../../../../stdlib/) (`io.scrippo`, `types.scrippo`, `path.scrippo`, `config.scrippo`, `helpers.bash`).

## Importing

```scrippo
@import scrippo/std          # io + types + path (NOT config)
@import scrippo/std/io       # prt, input, throw
@import scrippo/std/types    # list, dict, str
@import scrippo/std/path     # here
@import config               # separate; needs jq
```

Importing any of `prt`, `input`, `throw` embeds the shared `_scrippo_*` bash helpers; never call those directly. `config` always means the built-in module, so a user package named `config` cannot be imported by that name. Prefer `@import scrippo/std` for scripts that print, prompt, or touch paths.

## io

| Call | Effect |
|------|--------|
| `throw <code> msg…` | red message to stderr, then `exit code` |
| `prt [-s style…] msg … [-j sep] [-n]` | print; `-s` styles the **next** message, `-j` joins (default space), `-n` no trailing newline |
| `input <prompt> -v var` | read a line into `$var` |
| `input <prompt> --confirm -v var` | `true` / `false` (tty: `y`/`n` keypress, enter = no) |
| `input <prompt> --select a --select b -v var` | chosen word |
| `input <prompt> --select a --select b --multi -v var` | array of chosen words |

```scrippo
prt -s green "ok" -s bold -s red "stop"
prt -j "." a b c
input "Proceed?" --confirm -v answer
if $answer
  prt -s cyan "continuing"
else
  throw 1 "Aborted"
```

* styles: `bold dim italic underline black red green yellow blue magenta cyan white`; several names after one `-s` combine, a later `-s` styles a later message
* `--select` and `--confirm` are mutually exclusive; select repeats the flag per choice
* non-tty: confirm reads a line (leading `y`/`Y` = true); select prints numbered lines and reads numbers
* `throw` joins `msg` words with the first character of `IFS`

## types

`list`, `dict`, `str` take a **ref**: pass the variable's *name*, not `$name`. Flags come first, then the variable, then the optional operand.

```scrippo
mylist<list>=(a b c)
list --length mylist             # 3
list --join "." mylist           # a.b.c
list --reverse mylist            # c b a
list --contains mylist b         # true

mydict<dict>=(a=1 b=2 c=3)
dict --length mydict             # 3
dict --keys mydict               # keys
dict --values mydict             # values (also the default)
dict --contains mydict a         # true (keys only)

mystr<str>="a.b.c"
str --length mystr               # 5
str --split mystr "."            # a b c
str --reverse mystr              # c.b.a
str --contains mystr "b."        # true
```

* the ref parameters are named `array` (list), `dict` (dict), `string` (str); a caller variable with that exact name triggers a bash circular-nameref warning, so pick another name
* `--contains` on a list matches a whole element; on a dict it matches keys, not values; on `str` it is a substring match
* with no flag, each function prints its contents
* outputs go to stdout, so capture with `$(list --length mylist)`

## path

`here` prints the directory of the running script; `--file` prints the script path instead; `--rel` makes it relative to the current directory (`.` when equal).

```scrippo
root="$(here)"
this="$(here --file)"
rel="$(here --rel)"
```

It uses `_SCRIPPO_FILE` (set by `scrippo`) or `BASH_SOURCE`, then `realpath`. Run the script through `scrippo` or a shebang so the source is the `.scrippo` file, not the temp bash file.

## config

JSON config via `jq`. `@import config` (not in `scrippo/std`).

| Call | Effect |
|------|--------|
| `config --init myscript` | select `.scrippo/configs/myscript.config.json` (nearest project `.scrippo`, else created under cwd); creates `{}` if missing |
| `config --init ~/.myscript/config.json` | select that file (value ending `.json` is a path; otherwise an id of letters, digits, `_ - .`) |
| `config --init myscript --force` | reset an existing file to `{}` |
| `config --init myscript -f ai.enabled false -f ai.provider claude` | after selecting, set each `-f` field/value pair that is not already in the file (long form `--default`) |
| `config --interactive` | menu of every `field = value` in the selected file; bools pick true/false, other values are typed; empty answer keeps the value; `Done` exits |
| `config key` | print the value (strings raw, others JSON); exit 1 if missing/null |
| `config a=1 b=two` | set any number of values in one write |
| `config --assign key:VAR other:VAR2` | set shell variables from keys; missing key → empty string |

```scrippo
@import config

config --init myscript
  -f ai.enabled false
  -f ai.provider "claude"

.set()
  config
    hello=world
    foo=bar

.assign()
  config --assign hello:HELLO foo:FOO
  -> $HELLO $FOO

.settings()
  config --interactive
```

* `--init` applies to every later `config` call in the script, so write it once near the top; without it `config` errors
* dotted keys are paths: `a.b.c=1` → `{"a":{"b":{"c":1}}}`; numeric parts index arrays
* values that parse as JSON (`5`, `true`, `null`, `{"k":[1]}`) are stored as JSON; to store the string `"5"` write `'"5"'`
* one call is any number of sets **or** one get, not both
* `--assign` swallows every word after it; write it last
* `--init` stops if the existing file is invalid JSON
* `-f` takes pairs up to the next option, so write it after `--init`; it never overwrites an existing value (use `--force` to reset first), an odd word count or `-f` without `--init` errors
* `--interactive` needs a selected file, cannot combine with a field or `--assign`, and saves after every change; seed fields with `-f` so there is something to edit

## When to use bash instead

If the stdlib helper does not cover a case (e.g. `printf` formatting, `read -s`), fall back to raw bash, but check the helpers first so the script stays consistent.
