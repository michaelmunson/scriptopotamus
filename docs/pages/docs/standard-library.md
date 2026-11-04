---
title: Standard library
description: prt, input, throw, list, dict, str, here, and config.
---

The standard library is compiled into `scrippo`: the `io`, `types`, and `path` modules, plus `config`. Import the pieces you need, or all of the first three:

```scrippo
@import scrippo/std
@import scrippo/std/io
@import scrippo/std/types
@import scrippo/std/path
```

Importing any of `prt`, `input`, or `throw` causes the compiler to embed the shared bash helpers those functions call. You call the scrippo functions below; the `_scrippo_*` helpers are an implementation detail.

## io

### throw

```scrippo
throw(code<int> msg...)
```

Prints `msg` in red to stderr and exits with `code`. The words of `msg` are joined by the first character of `IFS`, normally a space.

```scrippo
throw 1 "Bump aborted"
```

### prt

```scrippo
prt(
  msg...
  [-s / --style<str>...]
  [-j / --join<str>]
  [-n]
)
```

Prints messages. `-j` / `--join` is the separator, defaulting to a space. `-n` omits the trailing newline.

`-s` / `--style` applies ANSI styles to the next message. Several style names after one `-s` combine. A later `-s` styles a later message.

```scrippo
prt -s green "ok" -s bold -s red "stop"
prt -j "." a b c
```

Style names: `bold`, `dim`, `italic`, `underline`, `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`.

### input

```scrippo
input(
  prompt
  [-v<str>]
  [
    --select<str>...
    [--multi]
  ] | [
    --confirm
  ]
)
```

Asks the user a question and stores the answer in the variable named by `-v`.

| Form | Answer |
| --- | --- |
| `input "Name" -v name` | The line the user types. |
| `input "Proceed?" --confirm -v answer` | `true` or `false`. |
| `input "Pick" --select a --select b -v choice` | The chosen word. |
| `input "Pick" --select a --select b --multi -v picks` | An array of chosen words. |

On a terminal, confirm is a `y` / `n` keypress (`enter` is no). Select is an arrow-key list: enter chooses, and with `--multi` space toggles, `a` toggles all, and enter confirms. Without a terminal, confirm reads a line and accepts a leading `y` or `Y`. Select prints numbered lines and reads numbers, separated by spaces or commas.

`--select` and `--confirm` are mutually exclusive.

## types

`list`, `dict`, and `str` take a [ref](/parameters): the argument is the variable's name. `list` names that parameter `array`, `dict` names it `dict`, and `str` names it `string`. Passing a variable with that same name prints a bash circular-nameref warning. The call uses that variable.

### list

```scrippo
list(array<ref list> [item] [--join<str>] [--length] [--reverse] [--contains])
```

`--length` prints the number of elements. `--reverse` prints the elements from last to first. `--contains` prints `true` when `item` equals an element and `false` otherwise. `--join` sets `IFS` to that string and prints the elements joined. With none of these, the function prints the elements.

```scrippo
mylist<list>=(a b c)
list --length mylist
list --join "." mylist
list --reverse mylist
list --contains mylist b
```

`list --length mylist` prints `3`. `list --join "." mylist` prints `a.b.c`. `list --reverse mylist` prints `c b a`. `list --contains mylist b` prints `true`. `item` is the word after the variable name. A match is the whole element.

### dict

```scrippo
dict(dict<ref dict> [key] [--keys] [--values] [--length] [--contains])
```

`--keys` prints the keys. `--values` and the default form print the values. `--length` prints the number of entries. `--contains` prints `true` when `key` is a key and `false` otherwise.

```scrippo
mydict<dict>=(a=1 b=2 c=3)
dict --length mydict
dict --keys mydict
dict --contains mydict a
```

`dict --length mydict` prints `3`. `dict --keys mydict` prints the keys `a`, `b`, and `c`. `dict --contains mydict a` prints `true`. `key` is the word after the variable name. A value that is not also a key does not match.

### str

```scrippo
str(string<ref str> [text] [--length] [--split] [--reverse] [--contains])
```

`--length` prints the character count. `--reverse` prints the characters from last to first. `--contains` prints `true` when `text` occurs in the string and `false` otherwise. `--split` splits the string on `text` and prints the parts. With none of these, prints the string.

```scrippo
mystr<str>="a.b.c"
str --length mystr
str --split mystr "."
str --reverse mystr
str --contains mystr "b."
```

`str --length mystr` prints `5`. `str --split mystr "."` prints `a b c`. `str --reverse mystr` prints `c.b.a`. `str --contains mystr "b."` prints `true`. `text` is the word after the variable name. `IFS` uses the first character of that word when splitting.

## path

### here

```scrippo
here([--file] [--rel])
```

Prints the directory that contains the running script. `--file` prints the script path instead. `--rel` prints that path relative to the current directory; a path that is the current directory prints `.`.

`here` uses `_SCRIPPO_FILE` when scrippo set it, and `BASH_SOURCE` otherwise, then `realpath`. Call it from a script you ran with `scrippo` or via a shebang so the source path is the `.scrippo` file and not the temporary bash file.

## config

```scrippo
@import config
```

`config` is imported by name and is not included in `scrippo/std`. It needs `jq` on the path.

```scrippo
config(
  field...
  [
    --init<str>
    [--force]
  ]
  [
    --assign<str>...
  ]
)
```

Reads and writes a JSON config file.

| Call | Effect |
| --- | --- |
| `config --init myscript` | Selects `.scrippo/configs/myscript.config.json`, creating it as `{}` when it does not exist. The file is found in the nearest project `.scrippo` directory, or created under the current directory. |
| `config --init ~/.myscript/config.json` | Selects that file. A value ending in `.json` is a path; anything else is an id of letters, digits, `_`, `-` and `.`. |
| `config --init myscript --force` | Resets an existing file to `{}`. |
| `config hello` | Prints the value of `hello`. Strings print as they are, other values as JSON. Exits 1 when the key is missing or null. |
| `config hello=world foo=bar` | Sets any number of values in one write. |
| `config --assign hello:HELLO foo:FOO` | Sets the variable `HELLO` to the value of `hello`, and `FOO` to the value of `foo`. A missing key sets an empty string. |

`--init` leaves the chosen file in effect for every later `config` call in the script, so it is usually written once near the top. Without it, `config` stops with an error. `--init` does nothing to an existing file unless `--force` is given, and stops if that file is not valid JSON.

A key with dots is a path: `a.b.c=1` sets `{"a":{"b":{"c":1}}}`, and a numeric part indexes an array. A value is stored as JSON when it parses as JSON (`5`, `true`, `null`, `{"k":[1]}`) and as a string otherwise. To store the string `"5"`, write `'"5"'`.

One call can hold any number of sets or one get, not both. `--assign` takes every word after it, so write it last.

```scrippo
@import config

config --init myscript

.set()
  config
    hello=world
    foo=bar

.get()
  config hello

.assign()
  config --assign hello:HELLO foo:FOO
  -> $HELLO $FOO
```
