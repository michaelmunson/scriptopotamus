---
title: Installation
description: Build scrippo and install the compiler and its wrapper.
---

`scrippo` is two programs that sit side by side. `scrippo` is a bash wrapper compiled from [`cli/scrippo.scrippo`](https://github.com/michaelmunson/scriptopotamus/blob/main/cli/scrippo.scrippo). It checks arguments, prints usage, and completes itself. `scrippo-core` is the Rust binary that parses, compiles, and emits shell hooks and completions. The wrapper execs the core binary next to it.

## Build

From a clone of the repository:

```bash
make build
cargo test
```

`make build` runs `cargo build`, then compiles `cli/scrippo.scrippo` to `target/debug/scrippo`. A release build is:

```bash
make build-release
```

That writes `target/release/scrippo` and `target/release/scrippo-core`.

## Install onto PATH

```bash
make install
```

This copies both binaries to `/usr/local/bin` and links `scriptopotamus` to `scrippo`. If `/usr/local/bin` is not writable:

```bash
make sudo-install
```

After that, `scrippo`, `scriptopotamus`, and `scrippo-core` are the same installation. The wrapper looks for `scrippo-core` in its own directory. Set `SCRIPPO_CORE` to point at a different core binary.

## Check the install

```bash
scrippo --help
scrippo -V
```

`--help` prints the wrapper usage and exits 0. `-V` / `--version` asks the core binary for its version.

Editor tooling is optional and documented on [Editors](/editors): `make build-extension` for VS Code, `make install-nvim` for Neovim.

## Install scrippolib

Creating, installing, and updating packages and scripts is done by `scrippolib`, which is built and installed separately. See [scrippolib](/scrippolib#get-it).

Next: [Getting started](/getting-started).
