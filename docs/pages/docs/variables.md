---
title: Variables and types
description: Declarations, assignments, constants, and the types the compiler checks.
---

A variable is a bash variable with an optional scrippo type. The type is checked when a value is assigned and when a typed parameter is bound. The check is a pattern match on the text of the value.

## Declarations and assignments

```scrippo
n<int>=1
label<str>="hello"
flag<bool>=true
items<list>=(a b c)
table<dict>=(name=ada year=1843)
choice<str {patch minor major}>=patch
loose<any>
```

`name<type>` declares without a value. `name<type>=value` declares and assigns. `name=value` assigns. A later assignment to a name that already has a type is checked against that type.

```scrippo
n<int>=1
n=2
n=two
```

The last line compiles, and the generated bash exits 1 at runtime with `n: expected int, received two`.

## Kinds

| Written | Also accepted | Bash declaration | Check |
| --- | --- | --- | --- |
| `str` | `string` | plain variable | none, unless a literal set is present |
| `int` | `integer` | `declare -i` | `^-?[0-9]+$` |
| `float` | | plain variable | `^-?[0-9]*\.?[0-9]+$` |
| `bool` | `boolean` | plain variable | `true` or `false` |
| `list` | `array` | `declare -a` | elements are not pattern-checked on a list literal |
| `dict` | | `declare -A` | entries are not pattern-checked on a dict literal |
| `any` | | plain variable | none |

`const` and `ref` may appear before the kind. `const` with no kind is `const any`. `ref` with no kind is `ref any`. Write `const` first when a type has both: `<const ref str>`.

```scrippo
PI<const float>=3.14
limit<const int>=3
```

Reassigning a const is a compile error: `cannot reassign const 'PI'`. Const integers and lists are bash readonly variables (`declare -r` combined with `-i` or `-a`).

## ref

`ref` declares a nameref. The value you assign is the name of another variable, and reads and writes follow that variable.

```scrippo
items<list>=(a b c)
view<ref list>=items
-> ${#view[@]}
```

`view=z` assigns through the nameref. On a list, that sets the first element. On a parameter, the caller passes the name and the parameter is the nameref. See [Parameters](/parameters).

## Literal sets

A type may list the only values it accepts, in braces or parentheses:

```scrippo
bump<str {patch minor major}>
mode<str (fast slow)>
level<int {1 2 3}>=1
```

The set is part of the runtime check. For `str`, `any`, `list`, and `dict`, the literal set is the whole check. For `int`, `float`, and `bool`, the value must match both the kind and the set. Usage text prints the set inside the type, for example `<str {patch minor major}>`.

Quote a literal that contains spaces or punctuation. The words are split the same way as the rest of the language: whitespace separates words, and quotes keep a word together.

## Bool values

Use the words `true` and `false`. Conditions written as `$flag` succeed only when that variable is the string `true`. A bool option the caller omitted is `false` unless you gave it another default.

## Where types are stored

Types live in the compiler while it emits one function. A typed declaration records the type for later assignments in that same function body. Parameter types are recorded the same way, so a later `name=value` inside the function is checked.

Imports paste declarations into the importer. A type does not exist at runtime except as the bash `declare` flags and the `if` tests the compiler inserted.
