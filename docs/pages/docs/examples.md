---
title: Examples
description: The example scripts in this repository, and what each one shows.
---

The repository keeps short programs under `docs/examples/`. Compile any of them with `scrippo -c` or run them with `scrippo`. Start with the tour if you want to see the whole language in one place.

## tour.scrippo

`docs/examples/tour.scrippo` is one script that uses every part of the language: types, values, math, control flow, functions, commands, parameters, `@inherit`, `@usage`, hooks, imports, raw bash, and the standard library. Each section below is a piece of that file. Every command shown was run against it.

```bash
scrippo docs/examples/tour.scrippo --help
scrippo docs/examples/tour.scrippo values
```

### Imports

```scrippo
#! /usr/bin/env scrippo
@import scrippo/std
@import config
@import "./demo/fs.scrippo" --as .fs
```

`scrippo/std` brings in `prt`, `input`, `throw`, `list`, `dict`, `str`, and `here`. `config` is a separate module and needs `jq`. The last line imports another script under a scope: its commands become `fs get-path ...`. Paths are relative to the file that contains the import. See [Imports](/imports).

```bash
scrippo docs/examples/tour.scrippo fs get-path abs .
```

### Variables and types

```scrippo
PI<const float>=3.14
limit<const int>=3
app<str>="tour"
verbose<bool>=false
ratio<float>=0.5
anything<any>
bump<str {patch minor major}>=patch
colors<list>=(red green "dark blue")
ports<list>=(
  80
  443 # https
  8080
)
person<dict>=(
  name="Ada Lovelace"
  year=1815
)
view<ref list>=colors
banner<str>="line one
line two"
```

Each name carries a type that is checked on assignment. `const` cannot be reassigned, and reassigning one is a compile error. A literal set like `{patch minor major}` restricts the allowed values. A parenthesized value is a list, or a dict when every word has `=`, and either can span lines and hold `#` comments. `view` is a `ref`: reads and writes follow `colors`. Quoted strings can span lines. See [Variables and types](/variables) and [Values and output](/values).

### Hooks

```scrippo
@hook exec
  prt -s dim "[tour] starting"

@hook exit code=CODE
  if $CODE != 0
    prt -s red "[tour] failed with $CODE"
  else
    prt -s dim "[tour] done"
```

Top-level hooks wrap every command. `exec` runs first and `exit` runs last with the command's status as `CODE`. `code` is an integer, so the test is `$CODE != 0`, not `if $CODE`. A `throw` exits the process at once and skips `exit` hooks; use `return` to see the failure hook run. See [Hooks](/hooks).

```bash
scrippo docs/examples/tour.scrippo fail
```

```text
[tour] starting
[tour] failed with 3
```

### Functions

```scrippo
circumference(diameter<float>)
  -> $PI * $diameter

double(n<int>) -> $n * 2

greet_all(greeting<str="hello"> [--shout] names...)
  for name in $names
    if $shout
      prt -s bold "${greeting^^} $name"
    else
      prt "$greeting $name"

total(*nums)
  sum<int>=0
  for n in $nums
    sum=$((sum + n))
  -> $sum

classify(n<int>)
  if $n -lt 0
    -> "negative"
  elif $n -eq 0
    -> "zero"
  else
    -> "positive"
```

A function is a bash function with a checked parameter list. `-> $PI * $diameter` is math because the operator has spaces around it, so it compiles to `bc`. `double` shows the one-line body. `greet_all` mixes a default, an option, and a repeated argument. `total` takes `*rest` and uses bash integer arithmetic, which needs no `bc`. `classify` shows `if`, `elif`, and `else`. Call a function the way you call any command: `greet_all hi --shout ada grace`. See [Functions](/functions).

### The root command

```scrippo
#? Entry point: runs when no command is named
.(
  [--name<str="world">] #? Who to greet
)
  prt -s green "Hello, $name!"
  prt "try:" -s yellow "tour --help"
```

`.()` is the script's own behavior. Because it has parameters, anything that is not another top-level command goes to it, so `scrippo docs/examples/tour.scrippo --name Ada` greets Ada. `#?` comments become the usage text. See [Commands](/commands) and [Usage text](/usage).

### Values, math, and output

```scrippo
#? Variables, types, lists, dicts and math
.values()
  -> "app=$app limit=$limit ratio=$ratio"
  -> "first color: $colors[0]"
  -> "third color: $colors[2]"
  -> "port count: ${#ports[@]}"
  -> "$person[name] was born in $person[year]"
  -> "$banner"
  -> "circumference of 10: $(circumference 10)"
  -> "double of 21: $(double 21)"
  -> 2 + 3 * 4
  area<float>=$PI * 2 * 2
  -> "area: $area"
  -> "view sees: $view[1]"
  anything="now a string"
  anything=42
  -> "anything=$anything bump=$bump"
  greet_all hi --shout ada grace
  greet_all
    hello
    linus
  -> "total: $(total 1 2 3 4)"
  n<int>=1
  n=2
  prt "n is" -s cyan $n
  ->
```

`$colors[0]` and `$person[name]` index a list and a dict. `-> 2 + 3 * 4` prints `14`, and `area` is `12.56`. `greet_all` is also written across several lines: lines indented under a command become its arguments, with no `\`. `n=two` would compile and then fail at run time with `n: expected int, received two`. A bare `->` prints a blank line.

### Control flow

```scrippo
#? Loops, conditions and case
.flow(
  count<int> #? How many times to loop
  [--mode<str {fast slow}>] #? Pick a mode
)
  i<int>=0
  while $i -lt $count
    -> "tick $i"
    i=$((i + 1))
  for c in $colors
    -> "color: $c"
  for p in 1 2 3; -> "port slot $p"
  if $verbose; -> "verbose"; else -> "quiet"
  if ! $verbose
    -> "not verbose"
  if grep -q "^PI" "$(here --file)"
    -> "found PI in this script"
  if $mode =~ ^f
    -> "mode starts with f"
  case $mode
    fast | quick)
      -> "going fast"
    slow) ;&
      -> "going slow"
    *)
      -> "no mode, or fell through"
  classify $count
```

The three kinds of condition all appear. `if ! $verbose` is a bool test, negated. `if grep -q ...` uses a command's exit status. `if $mode =~ ^f` is a test wrapped in `[[ ]]`. `for c in $colors` expands the whole list, one item per pass. `for p in 1 2 3; -> ...` and the one-line `if ...; ...; else ...` use `;`. In the `case`, patterns join with `|`, and `slow) ;&` falls through into the next arm. Nothing needs `fi`, `done`, or `esac`. See [Control flow](/control-flow).

```bash
scrippo docs/examples/tour.scrippo flow 2 --mode slow
```

```text
tick 0
tick 1
color: red
color: green
color: dark blue
port slot 1
port slot 2
port slot 3
quiet
not verbose
found PI in this script
going slow
no mode, or fell through
positive
```

### Parameters

```scrippo
#? Arguments, options, defaults, repeats and rest
.args(
  required<int> #? A required int
  [optional=notes] #? Optional with a default
  -v<str> #? A required option with a value
  [--level<int {1 2 3}=2>] #? Limited by a literal set
  [--tag<str>...] #? Repeatable option
  [-d / --dry-run] #? Bool option with an alias
  *rest #? Everything else, verbatim
)
  -> "required=$required optional=$optional v=$v level=$level"
  -> "dry-run=$dry_run d=$d"
  -> "tags: ${tag[*]}"
  -> "rest: ${rest[*]}"
```

```bash
scrippo docs/examples/tour.scrippo args 7 -v x --level 3 --tag a --tag b -d extra --more
```

```text
required=7 optional=extra v=x level=3
dry-run=true d=true
tags: a b
rest: --more
```

`extra` fills `optional`, and `--more` is an unknown option, so it lands in `rest`. `-d / --dry-run` is one option with two spellings. The variable is `$d`, and `$dry_run` is a nameref to it. Leaving out `-v` prints `args: -v is required` plus the usage and exits 1. `--level 9` fails the literal set with `--level expected int {1 2 3}, received '9'`. See [Parameters](/parameters).

```scrippo
#? Exclusive sets and groups
.pick(
  [
    --item<str>...
    [--multi]
  ] | [
    --all
  ]
)
  if $all
    -> "everything"
  elif $multi
    -> "many: ${item[*]}"
  else
    -> "one: ${item[*]}"
```

The `|` makes the two groups exclusive. `pick --item a --item b --multi` prints `many: a b`, and `pick --item a --all` fails with `[--item --multi] and --all are mutually exclusive`.

### Nested commands and @inherit

```scrippo
#? Nested commands that inherit their parent's parameters
.release(
  version<str> #? Version to release
  [--force] #? Skip checks
)
  #? Show what would happen
  .plan(@inherit)
    -> "would release $version"
  #? Do the release
  .ship(@inherit [--tag<str="latest">])
    if ! $force
      if ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$
        throw 1 "Invalid version: $version"
    -> "shipping $version as $tag"
  #? Show this command's usage and stop
  .help()
    @usage 0
```

The subcommand name comes first, then the arguments the parent declared:

```bash
scrippo docs/examples/tour.scrippo release plan 1.2.3
scrippo docs/examples/tour.scrippo release ship 1.2.3 --tag beta
scrippo docs/examples/tour.scrippo release ship bad
scrippo docs/examples/tour.scrippo release ship bad --force
scrippo docs/examples/tour.scrippo release help
```

`@inherit` copies `version` and `--force` into each child and `.ship` adds `--tag`. The third call fails with `Invalid version: bad`; the fourth skips the check. `release help` runs `@usage 0`, which prints usage for that command and exits 0. See [Inherit](/inherit).

### Raw bash

```scrippo
#? Scripts can use plain bash too
.bash()
  files=$(printf '%s\n' one two three)
  -> "files: $files"
  printf '%s\n' "printf works" >&2
  [[ -n $app ]] && echo "app is set"
  show
    one
    "two words"
    three
  n=$((limit * 2))
  -> "limit doubled: $n"

show(*rest)
  for i in $rest
    echo "arg: $i"
```

A line the compiler does not recognize is bash, copied through with `$` references rewritten. Pipelines, redirects, `[[ ]]` tests, and `$(( ))` arithmetic all work this way. `->` inside a bash line such as the `&&` one is not rewritten, which is why that line uses `echo`. See [Bash](/bash).

### Standard library

```scrippo
#? Pass a list to a function by name
.lists()
  mylist<list>=(a b c)
  mydict<dict>=(x=1 y=2 z=3)
  mystr<str>="a.b.c"
  -> "length $(list --length mylist)"
  -> "joined $(list --join "." mylist)"
  -> "reversed $(list --reverse mylist)"
  -> "has b: $(list --contains mylist b)"
  -> "keys $(dict --keys mydict)"
  -> "dict length $(dict --length mydict)"
  -> "has x: $(dict --contains mydict x)"
  -> "str length $(str --length mystr)"
  -> "split $(str --split mystr ".")"
  -> "contains b: $(str --contains mystr "b")"
```

`list`, `dict`, and `str` take the variable's name, not `$name`. They print to stdout, so capture them with `$( )`. Dict keys print in bash's hash order, not insertion order.

```scrippo
#? Stdlib: prt, throw and input
.io()
  prt -s green "ok" -s bold red "stop"
  prt -j "." a b c
  prt -n "no newline"
  ->
  input "Your name" -v who
  input "Proceed?" --confirm -v answer
  input "Pick one" --select a --select b -v choice
  if ! $answer
    throw 2 "Aborted by" "$who"
  -> "$who chose $choice"
```

`prt -s` styles the next message, and several style names after one `-s` combine. `-j` sets the separator and `-n` drops the newline. `input` stores a line, a confirmation, or a choice in the variable named by `-v`. `throw 2 ...` prints in red to stderr and exits with that code. Without a terminal, `input` reads from stdin, so the command can be scripted:

```bash
printf 'Ada\ny\n2\n' | scrippo docs/examples/tour.scrippo io
```

```scrippo
#? Stdlib: config
.settings()
  config --init tour --force
  config
    hello=world
    nested.key=1
    flag=true
  config --assign hello:HELLO nested.key:KEY
  -> "$HELLO $KEY"
  -> "flag is $(config flag)"
  -> "missing: $(config nope || echo none)"
```

`config --init tour` selects `.scrippo/configs/tour.config.json` under the current directory and `--force` resets it to `{}`. A dotted key writes a nested object, and a value that parses as JSON is stored as JSON, so `nested.key=1` stores a number. `--assign` copies keys into shell variables and must be the last thing on its line. A missing key makes `config key` exit 1, which the last line handles with `||`. Running `settings` leaves a `.scrippo` directory behind, because that is where the file lives. See [Standard library](/standard-library).

### Running it as a program

The script starts with a shebang, so after `chmod +x` it runs directly. To install it under a name, give it an `@info` header and publish it with `scrippolib`; see [Metadata](/metadata) and [Scripts](/scripts).

```bash
chmod +x docs/examples/tour.scrippo
./docs/examples/tour.scrippo values
scrippo --noexec docs/examples/tour.scrippo
```

`--noexec` compiles the script and runs `bash -n` over the output, which is the quickest check after an edit. See [Compilation](/compilation).

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
