---
title: Compilation
description: How a .scrippo file becomes bash, and what the generated program contains.
---

`scrippo file.scrippo` parses the file, expands it, checks it, emits bash, and runs that bash. `scrippo -c` stops after emit and writes the bash to stdout. `scrippo --noexec` runs `bash -n` on it.

## Pipeline

1. **Parse.** Indentation builds the tree. `@info` is validated and kept only as metadata.
2. **Import.** `@import` loads the standard library, packages, and files. Each imported scrippo file repeats this step.
3. **Macros.** `@inherit` in a parameter list is replaced by the parent parameter list. An unknown `@name` fails here.
4. **Check.** Duplicate command names and const reassignment fail here.
5. **Emit.** The tree becomes bash. Parameter binding, type tests, usage text, and command dispatch are generated.
6. **Run.** The bash is written to a temporary executable. `_SCRIPPO_FILE` is the source path. The script's arguments are passed through. A non-zero status from bash becomes scrippo's status. The temp file is removed after the process exits.

Libraries compiled during install skip the run step. They also skip command dispatch and the automatic top-level `--help`. A library that contains `.()` is rejected.

## The bash file

The output starts with `#!/usr/bin/env bash` when the program needs the standard-library helpers or imported `.bash` text. Helpers are embedded for `prt`, `input`, and `throw`. Imported bash files follow, then the program.

Functions become `function name`. Nested commands become `function parent__child`. The root command is `_scrippo_root`. Dispatch is a `case` on `$1` at the bottom of the file. Top-level statements that are not command definitions run before that `case`, on every invocation.

Usage strings are baked in twice: plain and colored. `_scrippo_usage` prints the colored form when stdout is a terminal and `NO_COLOR` is unset.

## Types at runtime

`int` variables are `declare -i`. `list` is `declare -a`. `dict` is `declare -A`. `const` adds `declare -r`. After a scalar assignment, `int`, `float`, `bool`, and literal sets get an `if` that exits 1 on mismatch. List and dict literals do not run that pattern on the whole value. Parameters are checked after binding, one element at a time for arrays.

These checks see the text of the value. They do not track types through bash arithmetic or through `eval`.

## Math and references

Math values, the ones with spaces around `+ - * / %`, become `$(bc <<< "...")`. `$name[index]` becomes `${name[index]}`. `$--flag` becomes the option variable. The rest of a bash line is unchanged.

## Install sandbox

`scrippo -i -p` compiles each package with a root set to the downloaded tree. An import whose real path is outside that root, or outside the embedded standard library, is an error. The compile does not run the package. `.install.scrippo` is a separate step and does run, before and after publication.

## Failure

Parse and check errors name a file and a line and are printed by the compiler. They do not produce a script. Runtime type errors, missing arguments, and unknown commands are bash `echo` to stderr, often followed by usage, then `exit 1`. `scrippo` forwards that exit code.

`bash -n` failures from `--noexec` are reported as `bash -n failed`. A script that needs `bc` fails at the math expression if `bc` is not installed. `here` fails if `realpath` is not installed.
