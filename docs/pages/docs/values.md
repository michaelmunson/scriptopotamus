---
title: Values and output
description: Scalars, lists, dicts, indexes, math, and the -> statement.
---

`->` prints a value with `echo` and a trailing newline. A bare `->` prints a blank line.

```
-> "hello"
-> $name
->
```

The right-hand side of `->` and the right-hand side of an assignment use the same value syntax.

## Scalars

A scalar is everything that is not a parenthesized list or dict and not recognized as math. Quotes are preserved and handed to bash.

```
title<str>="hello world"
path<str>='$HOME/notes'
message<str>="count is $n"
```

`$name` is left as a bash expansion. `"$name"` is the right habit when the value may contain spaces, because the compiler does not add quotes for you.

## References

The compiler rewrites a few reference forms before bash sees them:

| Written | Emitted |
| --- | --- |
| `$name` | `$name` |
| `${name}` | `${name}` |
| `$list[0]` | `${list[0]}` |
| `$table[key]` | `${table[key]}` |
| `$--loud` | `$loud` |
| `$-v` | `$v` |

Option references drop leading dashes and turn remaining `-` into `_`. `--dry-run` is the variable `dry_run`, so `$--dry-run` and `$dry_run` are the same expansion. Alias flags also create namerefs, described in [Parameters](/parameters).

`$(command)` and `` `command` `` are ordinary bash and are not rewritten, except for `$name` forms inside them.

## Lists

A parenthesized value whose words do not all contain `=` is a list.

```
items<list>=(a b "c d")
-> $items[0]
```

In `for i in $items`, a lone `$items` is expanded as `"${items[@]}"`, so each element is one loop value. In other positions `$items` is a normal bash array expansion and usually needs `[@]` or `[*]` yourself if you are writing bash directly.

## Dicts

If every word inside the parentheses contains `=`, the value is a dict. Keys become bash associative-array keys.

```
user<dict>=(name=ada year=1815)
-> $user[name]
```

Quote a key or value that has spaces. The compiler emits `[key]=value` entries.

## Math

A value is math when it contains no quotes or parentheses and has a `+`, `-`, `*`, `/`, or `%` with whitespace on both sides. Math is emitted as `$(bc <<< "...")`.

```
n<int>=2
-> $n + 3
area<float>=$width * $height
```

`$n+$n` is a scalar, because the operator has no surrounding spaces, and bash will not evaluate it. Integer `declare -i` variables still do bash integer arithmetic on assignment of a bash arithmetic expression, which is separate from this `bc` rule.

`bc` is required on `PATH` whenever a script evaluates a math value. Integer-only programs can use bash `$((n + 1))` as a [bash statement](/bash) or inside a scalar.
