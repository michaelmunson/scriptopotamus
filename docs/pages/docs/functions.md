---
title: Functions
description: Named functions, calls, and how they differ from commands.
---

A function is a name, parentheses, optional parameters, and an indented body. It compiles to a bash function. Call it the way you call a bash function: `name arg arg`.

```
circumference(diameter<float>)
  -> $PI * $diameter

-> "$(circumference 10)"
```

A one-line body may sit after the header when it is a `->` expression:

```
double(n<int>) -> $n * 2
```

## Calls

The compiler does not invent a call syntax. A line that looks like `circumference 10` is bash, and bash runs the function. Command substitution, pipelines, and redirects around the call are bash as well.

Parameters inside the function are local bash variables. The original `"$@"` is saved and restored after binding, so the body can still forward the arguments it was called with.

## Parameters

Function parameters use the same grammar as command parameters: positional arguments, options, groups, exclusive sets, defaults, repeats, and `*rest`. See [Parameters](/parameters).

```
choose(
  path
  [optional_arg<int>]
  -v<str>
)
  -> "$path $optional_arg $v"
```

Calling `choose file -v debug` binds `path=file` and `v=debug`. A missing required parameter prints usage and exits 1. `--help` passed to a function prints that function's usage and returns 0, unless the function declared `--help` itself.

## Nested functions

A function may contain other functions. They are emitted as bash functions inside the outer function, which is bash's dynamic scoping: the inner function exists after the outer one has been called. Prefer top-level functions when the helper should be callable immediately.

Commands (names that start with `.`) nested in a function become subcommands of that command when the outer name is itself a command. Nested commands inside a plain function are still emitted, and they dispatch when that function is entered. The usual place for a command tree is the top level. See [Commands](/commands).

## Documentation

`#?` lines directly above the function, and `#?` on its parameters, become its usage entry. `@usage` inside the body prints that entry. See [Usage text](/usage).

## Imports

Imported functions are pasted into the importer and emitted as bash functions. Two commands with the same name are a compile error. Two functions with the same name become two bash `function` definitions, and the later one wins when the script runs. Name shared helpers so they do not collide, or import them with `@import ... --as` into a scope. See [Imports](/imports).
