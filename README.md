# Scriptopotamus

Scriptopotamus (`scrippo`) compiles `.scrippo` to bash. See [docs/vision.md](docs/vision.md) for the language.

## Build

```bash
cargo build
cargo test
```

Binary: `target/debug/scrippo` (or `cargo run --bin scrippo -- …`).

## Usage

```bash
scrippo -c script.scrippo          # compile to stdout
scrippo -n script.scrippo          # compile + bash -n
scrippo script.scrippo             # compile and run
scrippo script.scrippo greet world # run with args (command dispatch)
scrippo hello                      # run ~/.scrippo/scripts/hello.scrippo
```

## Packages & scripts

Local packages live in `.scrippo/lib/` and are importable from anywhere in the project:

```scrippo
@import mylib
```

Global packages and scripts live under `~/.scrippo` (override with `SCRIPPO_HOME`):

```
~/.scrippo/
├── config                     # autoload / autocomplete
├── lib/<package>.scrippo      # @import <package>
└── scripts/<script>.scrippo   # scrippo <script> / scrippo <script>.scrippo
```

Create them with:

```bash
scrippo --new --package [--local|--global] <name>   # default: --local
scrippo --new --script  [--local|--global] <name>   # default: --global
```

Project commands live in `.scrippo/commands.scrippo` (a legacy `.scrippo` file still works).

## Config & autoload

```bash
scrippo --configure                          # print settings
scrippo --configure autoload=true autocomplete=true
```

Settings live in `~/.scrippo/config` (override with `SCRIPPO_CONFIG` or `SCRIPPO_HOME`).

With `autoload=true`, add this to your shell rc:

```bash
eval "$(scrippo --hook)"
```

Entering a directory then aliases:

- `package.scrippo` (or any scrippo-shebang file) as `package` → `scrippo package.scrippo`
- each top-level command in `.scrippo/commands.scrippo` (e.g. `build`) as `build` → `scrippo .scrippo/commands.scrippo build`
- global scripts in `~/.scrippo/scripts/` (local names win on conflict)

With `autocomplete=true`, tab completion covers those aliases' commands and `--` options.

## Install

```bash
make install   # installs scrippo (+ scriptopotamus symlink) to /usr/local/bin
```

## Examples

See [docs/examples/](docs/examples/).
