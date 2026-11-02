---
title: Bash
description: Lines the compiler copies through to the generated bash program.
---

Any statement the compiler does not recognize as scrippo syntax is bash. The line is emitted as written, after `$` references are rewritten. Lines indented under that statement belong to the same bash snippet. Their shared leading indent, equal to the statement's indent, is removed.

```
if $ready
  grep -n "$pattern" "$file" \
    | head
```

The `if` is scrippo. The `grep` pipeline is bash. Use this for tests, redirects, arithmetic, and programs that already have a bash shape.

## What is rewritten

`$name`, `${name}`, `$name[key]`, `$--flag`, and `$-flag` are rewritten everywhere in the bash text, including inside double quotes. The rules are the same as in [Values and output](/values).

A bash line that begins with `@name` expands that macro when it exists. See [Macros](/macros).

## Patterns that belong here

```
n=$((n + 1))
[[ -f $path ]] && -> "present"
result="$(realpath "$path")"
printf '%s\n' "$name" >&2
```

`->` inside a bash line is still just the characters `-` and `>`. The second example works because `&&` makes the whole line bash, and the compiler does not reinterpret `->` inside it. To print from scrippo, use a `->` statement as its own line.

`declare`, `local`, `eval`, pipelines, here-docs, and `{ ... }` groups are all available because they are bash. The compiler does not check them. `scrippo --noexec` runs `bash -n` on the finished program and is the check for this layer.

## Indentation and blocks

Scrippo blocks and bash blocks use the same indent rule. A bash `if` written out in full, with `then` and `fi`, is one bash snippet only when the continuation lines are indented under the first line. A `fi` at the same indent as surrounding scrippo statements is a compile error, because a line that is exactly `fi` is rejected.

Prefer scrippo `if`, `for`, `while`, and `case` for new code. Drop to bash for an operator or program the structured syntax does not represent.

## Helpers

Imported `.bash` and `.sh` files are prepended to the generated script as raw text. They are not parsed as scrippo. Package installs refuse to compile against bash that lives outside the repository sandbox, aside from the standard library embedded in the compiler.
