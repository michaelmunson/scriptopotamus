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
```

## Install

```bash
make install   # installs scrippo (+ scriptopotamus symlink) to /usr/local/bin
```

## Examples

See [docs/examples/](docs/examples/).
