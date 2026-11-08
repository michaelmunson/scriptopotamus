---
name: scrippo-syntax
description: >-
  Core Scriptopotamus (.scrippo) language syntax: indentation, variables and
  types, values, lists and dicts, math, `->` output, if/for/while/case, functions,
  and raw bash passthrough. Use when writing or editing the body of any .scrippo
  file.
---

# Scrippo syntax

Scrippo compiles to bash. Prefer scrippo syntax over raw bash unless a construct has no scrippo form yet. Reference: [docs/pages/docs](../../../../docs/pages/docs/) (`syntax.md`, `variables.md`, `values.md`, `control-flow.md`, `functions.md`, `bash.md`). Runnable samples: [docs/examples](../../../../docs/examples/).

## Style

* match surrounding file indentation (examples use 2 spaces; tabs are fine, never mix)
* no `#` comments unless the user asks or a non-obvious invariant needs one (`#?` usage docs are not comments, see [commands](../commands/SKILL.md))
* prefer typed declarations (`name<type>=…`) over bare bash assigns
* use `->` instead of `echo` for output
* end blocks by outdenting; never write `fi` / `done` / `esac` (a line that is exactly one is a compile error)
* keep bodies short; extract helpers or packages when a script grows

## Statements

A statement is one logical line plus the lines indented under it. The compiler classifies each line in this order: `@info`, inline `;` sequence, `if/for/while/case`, `->`, `@` macro, function/command header, assignment/declaration, then **anything else is bash** (copied through with `$` references rewritten).

`;` joins statements on one line for `if`/`elif`/`else`/`for`/`while` (not `case`):

```scrippo
if $flag; -> "yes"; else -> "no"
for i in a b c; -> $i
```

## Variables & types

```scrippo
n<int>=1
s<str>="hi"
flag<bool>=true
ratio<float>=0.5
PI<const float>=3.14
items<list>=(a b c)
table<dict>=(a=1 b=2)
choice<str {patch minor major}>
any_val<any>
view<ref list>=items
```

| Kind | Alias | Bash | Check |
|------|-------|------|-------|
| `str` | `string` | plain | none (unless literal set) |
| `int` | `integer` | `declare -i` | `^-?[0-9]+$` |
| `float` | | plain | `^-?[0-9]*\.?[0-9]+$` |
| `bool` | `boolean` | plain | `true` / `false` |
| `list` | `array` | `declare -a` | none per element on literals |
| `dict` | | `declare -A` | none |
| `any` | | plain | none |

* `name<type>` declares, `name<type>=v` declares and assigns, `name=v` assigns (checked against the earlier type; mismatch exits 1 at runtime)
* `const` / `ref` come before the kind: `<const ref str>`; reassigning a const is a compile error
* literal sets: `<str {a b c}>` or `<int {1 2 3}>`; quote words with spaces
* bools are the words `true` / `false`

## Values

* `$name`, `${name}` pass through to bash. Quote `"$name"` when it may contain spaces; the compiler adds no quotes
* `$list[0]` → `${list[0]}`, `$dict[key]` → `${dict[key]}`
* `$--flag` / `$-v` → option variables (dashes dropped, `-` → `_`; `--dry-run` is `$dry_run`); the bare name is preferred
* a parenthesized value is a **list** unless every word contains `=`, then it is a **dict**; both can span lines and contain `#` comments
* quoted strings can span lines; double quotes interpolate, single quotes are literal

```scrippo
items<list>=(
  one
  "two words"
)
user<dict>=(
  name="Ada Lovelace"
  year=1815
)
```

* **math**: a value with `+ - * / %` surrounded by spaces becomes `$(bc <<< "...")`; `$a+$b` (no spaces) is a plain scalar. Needs `bc`. For integer-only work use bash `$((n + 1))`
* `->` prints with `echo`; bare `->` prints a blank line. Its operand uses value syntax, including math: `-> $a + $b`

## Control flow

```scrippo
if $flag
  -> "yes"
elif $n -lt 3
  -> "small"
else
  -> "no"

for i in $items
  -> $i

while $n -lt 3
  -> $n
  n=$((n + 1))

case $choice
  patch | minor) -> "ok"
  major)
    -> "major"
    ;&
  *) -> "?"
```

Conditions:

* lone `$name` / `$--flag` → `[[ $name == true ]]`; `! ` negates the whole condition
* starts with a letter/`_` (a command) → its exit status: `if grep -q TODO NOTES.md`
* anything else → wrapped in `[[ ]]`: `if $n -lt 3`, `if $v =~ ^[0-9]+$`
* an int variable (e.g. `$code`) is **not** a bool; test `if $code != 0`
* `for x in $items` expands to `"${items[@]}"` only when the word is exactly `$items`
* `case` arms end with `;;` by default; `;&` and `;;&` fall through

## Functions

```scrippo
double(n<int>) -> $n * 2

circumference(diameter<float>)
  -> $PI * $diameter

-> "$(circumference 10)"
```

* a function compiles to a bash function; call it like one (`name arg arg`), including in `$( )` and pipelines
* parameters use the grammar in [commands](../commands/SKILL.md) (defaults, options, repeats, `*rest`, `ref`)
* parameters are local; a missing required one prints usage and exits 1; `--help` prints usage and returns 0
* same-named functions both compile and the later one wins at runtime; name helpers to avoid collisions
* nested functions are emitted inside the outer function (bash dynamic scope); prefer top-level helpers

## Raw bash

Unrecognized lines pass through, with `$` references rewritten. Use it for tests, redirects, `local`/`declare`/`eval`, here-docs, and pipelines.

```scrippo
n=$((n + 1))
[[ -f $path ]] && -> "present"   # whole line is bash; the -> is NOT rewritten
result="$(realpath "$path")"
printf '%s\n' "$name" >&2
```

* a simple command followed by more-indented lines takes them as further arguments (no `\` needed); folding is skipped after a trailing `do then { ( ; | && ||`, inside here-docs, or when an indented line starts with `fi`/`done`/`esac`/`}`
* to print from scrippo, put `->` on its own statement line
* `scrippo --noexec` runs `bash -n` and is the check for this layer
* imported `.bash` / `.sh` files are prepended raw, not parsed

## Names

Variables and functions: letter or `_`, then letters/digits/`_`. Command names may also contain `-`. A shebang (`#! /usr/bin/env scrippo`) is a comment to the compiler.
