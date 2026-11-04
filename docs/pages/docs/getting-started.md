---
title: Getting started
description: Compile, check, and run a .scrippo file.
---

A scrippo file is text with a `.scrippo` extension. Blocks are indented. `->` prints a line. Names followed by `()` are functions. Names that start with `.` and `()` are commands.

Create `hello.scrippo`:

```scrippo
#! /usr/bin/env scrippo

.greet(name [--loud])
  if $loud
    -> "HELLO $name"
  else
    -> "hello $name"
```

The shebang is a comment to the compiler. It matters when the file is executed directly, and when autoload decides that a file without a `.scrippo` name is still a scrippo script.

## Run it

```bash
scrippo hello.scrippo greet ada
scrippo hello.scrippo greet ada --loud
scrippo hello.scrippo --help
scrippo hello.scrippo greet --help
```

`scrippo` compiles the file, writes a temporary bash script, and runs it with the arguments that follow the filename. `_SCRIPPO_FILE` is set to the source path so helpers such as `here` know which file they came from.

A bare name is resolved for you:

```bash
scrippo hello greet ada
```

Resolution looks for `hello.scrippo` in the current directory, then in each parent directory, then in `~/.scrippo/scripts`. See [Scripts](/scripts).

## Compile without running

```bash
scrippo -c hello.scrippo
scrippo --compile hello.scrippo
```

The bash is written to stdout. Use this when you want to read the output, or pipe it somewhere else.

```bash
scrippo --noexec hello.scrippo
```

`--noexec` compiles and runs `bash -n` on the result. A clean exit means the generated bash parses. It does not run the script.

## Direct execution

With the shebang and the executable bit:

```bash
chmod +x hello.scrippo
./hello.scrippo greet ada
```

`/usr/bin/env` must find `scrippo` on `PATH`. The wrapper still compiles on each run. There is no separate bytecode cache.

## A file with no commands

Top-level statements run as the program:

```scrippo
name<str>="ada"
-> "hello $name"
```

```bash
scrippo greet.scrippo
```

If the file also defines commands, those top-level statements still run first, and then the first argument selects a command. Prefer keeping executable work inside functions and commands so importing the file does not run it. See [Commands](/commands) and [Imports](/imports).

## Start a script with scrippolib

`scrippolib new script hello` writes a starter `hello.scrippo` like the one above, and `scrippolib new script -g hello` puts it in `~/.scrippo/scripts` so `scrippo hello` finds it from anywhere. `scrippolib` also creates packages and installs other people's. See [scrippolib](/scrippolib).

## When something fails

Compile errors name the file and line. Argument errors print usage on stderr and exit 1. `--help` prints usage on stdout and exits 0, unless you declared `--help` yourself. `NO_COLOR` turns usage colors off when it is set to anything, including an empty value.
