---
title: Control flow
description: Conditions, if, for, while, and case.
---

`if`, `for`, `while`, and `case` compile to the bash statements of the same names. Bodies are indented. The compiler inserts `then`, `fi`, `do`, `done`, and `esac`.

## Conditions

A condition is the text after `if`, `elif`, or `while`. A leading `! ` (bang and a space) negates the whole condition.

**Bool.** A condition that is only `$name`, `${name}`, `$--flag`, or `$-flag` becomes `[[ $name == true ]]`.

```
if $loud
  -> "HELLO"
if ! $dry_run
  -> "applying"
```

**Command.** A condition that starts with a letter, `_`, or another character outside `$`, quotes, digits, and `-` is emitted as a command. Its exit status is the condition.

```
if grep -q TODO NOTES.md
  -> "found"
```

**Test.** Every other condition is wrapped in `[[ ]]`. A condition that already starts with `[` or `[[` is left alone.

```
if $n -lt 3
  -> "small"
if $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$
  -> "stable"
if [[ -f $path ]]
  -> "file"
```

`$n -lt 3` becomes `[[ $n -lt 3 ]]`. Bash test operators (`-lt`, `-eq`, `-f`, `=~`, `&&`, `||`) work inside that form.

## if

```
if $flag
  -> "yes"
elif $other
  -> "other"
else
  -> "no"
```

`elif` and `else` align with `if`. An `else` or `elif` without a matching `if` at that indent is a compile error. The one-line form uses semicolons:

```
if $flag; -> "yes"; else -> "no"
```

## for

```
for i in a b c
  -> $i

items<list>=(a b c)
for i in $items
  -> $i
```

The loop variable is a single identifier. Items are words. A word that is only `$name` expands to `"${name[@]}"`. Other words are copied with reference rewriting, so `$prefix/$name` stays a single word.

```
for i in $items; -> $i
```

## while

```
n<int>=0
while $n -lt 3
  -> $n
  n=$((n + 1))
```

The assignment on the last line is bash arithmetic written as a scalar. The condition is a test. `while` can be written on one line with semicolons, the same way as `if`.

## case

`case` follows bash pattern syntax. The word `in` after the subject is optional.

```
case $choice
  patch | minor)
    -> "ok"
  major)
    -> "major"
    ;;
  *)
    -> "?"
```

Each arm is `pattern)`, then an indented body. Several patterns may be joined with `|`.

The arm ends with a bash terminator:

| Terminator | Meaning |
| --- | --- |
| `;;` | Stop. This is the default when you omit a terminator. |
| `;&` | Fall through and run the next arm's body. |
| `;;&` | Fall through and test the next pattern. |

Put the terminator at the end of the arm's first line, or as its own indented line under the arm:

```
case $choice
  patch)
    -> "patch"
    ;&
  minor)
    -> "also minor"
```

A terminator alone at the arm's indent, with no pattern, is a compile error.
