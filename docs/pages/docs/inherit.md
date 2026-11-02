---
title: Inherit
description: Copy a parent command's parameters into a nested command with @inherit.
---

`@inherit` in a parameter list copies the enclosing function or command's parameters into the child. Nested commands use it so a subcommand can take the same arguments as its parent without repeating them.

```
#? get a path
.path(
  #? input path
  path
  [--count<int=2>] #? number of paths
)
  #? relative path
  .rel(@inherit)
    -> $path
  .abs(@inherit [--verbose])
    -> $path
```

```bash
scrippo app.scrippo path rel hello
scrippo app.scrippo path abs hello --verbose
```

`path rel` accepts `path` and `--count`, then runs `.rel`. `path abs` accepts those and `--verbose`.

## What is copied

`@inherit` is a positional stand-in. It is replaced by the parent's parameter list, in order, at the place you wrote it. You may add parameters before or after it:

```
.world(@inherit [--age<int>])
  -> "$name $age"
```

The child's usage lists the inherited parameters and the child's own. Documentation comments whose key is a parameter name are copied onto the child. The parent's command-level comment stays on the parent; the child keeps its own `#?`.

Inheritance walks the chain. A grandchild that writes `@inherit` receives the child's list, which already includes the grandparent's parameters. Each level binds the arguments again, so `path rel deep hello --loud` still fills `path` and `loud` inside the deepest command.

## Where it is legal

`@inherit` needs an enclosing function or command. A top-level command that writes `@inherit` is a compile error: `@inherit requires a parent command or function`.

The parent is the function or command whose body contains the child. Import scopes and nested commands both count. A child command is still dispatched by name; `@inherit` does not change the words the user types, only the parameters that command accepts after its name.

## Usage

`path rel --help` shows `path rel`, the child's `#?` text, and the inherited parameters with their comments. That is the same text `@usage` prints from inside `.rel`.
