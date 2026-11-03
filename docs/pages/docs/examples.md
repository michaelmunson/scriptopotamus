---
title: Examples
description: The example scripts in this repository, and what each one shows.
---

The repository keeps short programs under `docs/examples/`. Compile any of them with `scrippo -c` or run them with `scrippo`.

## simple.scrippo

A constant, a function, and a top-level print. This is the smallest program that uses a type and arithmetic:

```scrippo
PI<const float>=3.14

circumference(diameter<float>)
  -> $PI * $diameter

-> "$(circumference 10)"
```

`-> $PI * $diameter` is math because the operator has spaces around it, so the compiler emits a `bc` substitution. The call site uses command substitution, which is ordinary bash inside the output expression.

## commands.scrippo

`.greet` is a top-level command. `.math` has nested commands `.add` and `.mul`:

```bash
scrippo docs/examples/commands.scrippo greet ada --loud
scrippo docs/examples/commands.scrippo math add 2 3
```

Nested commands are invoked as extra words: `math add`, not `math.add`.

## imports.scrippo

```scrippo
@import scrippo/std

prt -s green "hello from stdlib" -s red "hello from imports"

myls<list>=(1 2 3 4)

list (1 2 3 4) --join "."
```

`@import scrippo/std` pulls in `prt`, `input`, `throw`, `list`, `dict`, `str`, and `here`. `prt -s` colors the following argument. The `list (1 2 3 4)` line is copied through as bash; bash does not accept those parentheses as an argument, so execution stops there. Build a list with `items<list>=(1 2 3 4)` and index it with `$items[0]`. `str` takes one string, so `str "1.2.3.4" --split "."` is the helper that matches a single positional.

## demo/

`docs/examples/demo/demo.scrippo` imports `fs.scrippo` as the command `.fs`, imports the standard library, and defines `.hello` with a nested `.world` that copies `name` through `@inherit`.

`docs/examples/demo/fs.scrippo` defines `.get-path` with `.rel` and `.abs`. Both subcommands inherit `path`. The bodies call `realpath`, which is bash command substitution inside `->`.

The repository's own `_.scrippo` is a larger command script: it imports the standard library and the demo, then defines `.version` and `.build` for this project. It is a useful picture of commands, typed options, `input --confirm`, and `throw` together. It is a project tool, not a library.
