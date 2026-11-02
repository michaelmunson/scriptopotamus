---
title: Standard library
description: prt, input, throw, list, dict, str, and here.
---

The standard library is three modules compiled into `scrippo`. Import the pieces you need, or all of them:

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

### list

```scrippo
list(array<list> [--join<str>] [--length])
```

`--length` prints `${#array[@]}`. `--join` sets `IFS` to that string and prints `"${array[*]}"`. With neither flag, the function prints `"${array[@]}"`.

The parameter is one positional word, bound with `local array=$word`, the same way as any other non-repeatable argument. It is not a repeatable `...` parameter, so the words of a list are not collected into an array by the call. Index a list variable, or loop over it, when you need its elements:

```scrippo
items<list>=(a b c)
-> $items[0]
for i in $items
  -> $i
```

A line such as `list (1 2 3 4) --join "."` is bash text. Bash rejects the parentheses there, and the script stops at that line.

### dict

```scrippo
dict(dict<dict> [--keys] [--values] [--length])
```

`--keys` prints `${!dict[@]}`. `--values` and the default form print `${dict[@]}`. `--length` prints `${#dict[@]}`. The positional is one word, bound to `dict`. Read an entry with an index:

```scrippo
user<dict>=(name=ada year=1815)
-> $user[name]
```

### str

```scrippo
str(string<str> [--length] [--split<str>])
```

`--length` prints the character count. `--split` splits on that separator and prints the parts. With neither option, prints the string.

## path

### here

```scrippo
here([--file] [--rel])
```

Prints the directory that contains the running script. `--file` prints the script path instead. `--rel` prints that path relative to the current directory; a path that is the current directory prints `.`.

`here` uses `_SCRIPPO_FILE` when scrippo set it, and `BASH_SOURCE` otherwise, then `realpath`. Call it from a script you ran with `scrippo` or via a shebang so the source path is the `.scrippo` file and not the temporary bash file.
