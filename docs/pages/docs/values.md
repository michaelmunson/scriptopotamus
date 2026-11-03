---
title: Values and output
description: Scalars, lists, dicts, indexes, math, and the -> statement.
---

`->` prints a value with `echo` and a trailing newline. A bare `->` prints a blank line.

```scrippo
-> "hello"
-> $name
->
```

The right-hand side of `->` and the right-hand side of an assignment use the same value syntax.

## Scalars

A scalar is everything that is not a parenthesized list or dict and not recognized as math. Quotes are preserved and handed to bash.

```scrippo
title<str>="hello world"
path<str>='$HOME/notes'
message<str>="count is $n"
```

`$name` is left as a bash expansion. `"$name"` is the right habit when the value may contain spaces, because the compiler does not add quotes for you.

Quoted strings can span lines in assignments and after `->`. Newlines, blank lines, and indentation inside the quotes are preserved; `#` inside the string is literal text.

```scrippo
name<str>=Ada
message<str>="Hello $name
This is the second line."
raw<str>='Literal $name
No variable expansion here.'
-> "$message"
```

Double quotes use Bash interpolation and escaping. Single quotes preserve text literally. Put the closing quote at the end of the value, optionally followed by a comment. There is no automatic indentation removal. Multiline strings also work as list elements and dict values. A missing closing quote reports an error at the value's opening line.

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

```scrippo
items<list>=(a b "c d")
-> $items[0]
```

Lists can span lines. Parentheses delimit the value, so continuation lines do not open indentation blocks. Whitespace separates elements; commas are not separators.

```scrippo
items<list>=(
  one
  "two words" # an inline comment
  three
)
```

In `for i in $items`, a lone `$items` is expanded as `"${items[@]}"`, so each element is one loop value. In other positions `$items` is a normal bash array expansion and usually needs `[@]` or `[*]` yourself if you are writing bash directly.

## Dicts

If every word inside the parentheses contains `=`, the value is a dict. Keys become bash associative-array keys.

```scrippo
user<dict>=(name=ada year=1815)
-> $user[name]
```

Quote a key or value that has spaces. The compiler emits `[key]=value` entries.

Dicts can span lines too, with whitespace-separated `key=value` entries:

```scrippo
user<dict>=(
  name="Ada Lovelace"
  year=1815
  # Additional profile fields
  role=programmer
)
-> "$user[name]"
```

You may put entries on the opening line and continue on following lines. Both list and dict literals support blank lines, `#` comments. A closing `)` ends the value regardless of its indentation. Quoted parentheses and `#` characters remain part of the value. An unclosed literal reports an error at its opening line. Multiline literals work in assignments.

## Math

A value is math when it contains no quotes or parentheses and has a `+`, `-`, `*`, `/`, or `%` with whitespace on both sides. Math is emitted as `$(bc <<< "...")`.

```scrippo
n<int>=2
-> $n + 3
area<float>=$width * $height
```

`$n+$n` is a scalar, because the operator has no surrounding spaces, and bash will not evaluate it. Integer `declare -i` variables still do bash integer arithmetic on assignment of a bash arithmetic expression, which is separate from this `bc` rule.

`bc` is required on `PATH` whenever a script evaluates a math value. Integer-only programs can use bash `$((n + 1))` as a [bash statement](/bash) or inside a scalar.
