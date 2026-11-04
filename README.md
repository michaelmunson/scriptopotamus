# Scriptopotamus

Scriptopotamus (`scrippo`) compiles `.scrippo` to bash. The language reference is published from [docs/pages](docs/pages/).

## Build

```bash
make build   # cargo build + compile the scrippo wrapper
cargo test
```

`scrippo` is itself a scrippo script: [cli/scrippo.scrippo](cli/scrippo.scrippo) declares
the options, their nesting and exclusivity, the `#?` usage text, and (through the same
declaration) tab completion. `make build` compiles it to `target/debug/scrippo`, a bash
script that validates its arguments and then execs `scrippo-core`, the Rust binary doing
the actual work, found next to it (override with `SCRIPPO_CORE`). Both must sit side by
side, as `make install` arranges.

## Usage

```bash
scrippo -c script.scrippo          # compile to stdout
scrippo --noexec script.scrippo    # compile + bash -n
scrippo script.scrippo             # compile and run
scrippo script.scrippo greet world # run with args (command dispatch)
scrippo hello                      # run ./hello.scrippo (or a parent dir's), else ~/.scrippo/scripts/hello.scrippo
```

A bare script name resolves to the nearest `<name>.scrippo` in the current directory or
any parent directory, so it works from subdirectories. Local scripts override global ones.
Tab completion for `scrippo <name> <tab>` uses the same resolution.

## Packages & scripts

Local packages live in `.scrippo/lib/` and are importable from anywhere in the project:

```scrippo
@import mylib
```

Global packages and scripts live under `~/.scrippo` (override with `SCRIPPO_HOME`):

```
~/.scrippo/
├── config                     # autoload / autocomplete / alias
├── lib/<package>.scrippo      # @import <package>
│       <package>.bash        # compiled, installed packages
└── scripts/<script>.scrippo   # scrippo <script> / scrippo <script>.scrippo
```

Create them with:

```bash
scrippolib new package [-g] <name>   # package; local unless -g (needs scrippolib init)
scrippolib new script [-g] <name>    # script; local unless -g
```

Installing, updating, listing, and removing packages and scripts is done by
[`scrippolib`](../scrippolib/README.md) (`scrippolib init` creates the project `.scrippo`).

Project commands live in `.scrippo/commands.scrippo` (a legacy `.scrippo` file still works).

### Installing with scrippolib

```bash
scrippolib init                                    # initialize .scrippo here
scrippolib install shell                           # from the registry
scrippolib install -g shell                        # ~/.scrippo
scrippolib install https://github.com/owner/helpers.git
scrippolib install --tag v1.2.0 https://github.com/owner/helpers.git
scrippolib install ./helpers                       # plain local directory
scrippolib update [name] / uninstall <name> / list / info <name>
```

A bare name is installed from the configured registry
(`scrippolib configure registry=<url>`); anything else is a Git URL or directory.
Packages are compiled to `<name>.bash` with all imports inlined and exposed through
`@import <name>`; scripts keep their source tree and are linked as `<name>.scrippo`.
Package compilation never executes repository code and imports are sandboxed to the
downloaded source. The full behavior is documented in
[Installing](docs/pages/docs/installing.md).

### Publishing installable entries

Lay the repository out however you like (nested directories are allowed, with
unique names per kind):

```text
helpers/
├── is-even.scrippo
├── greet.scrippo
└── support/common.scrippo
```

Each installable `.scrippo` file needs exactly one metadata declaration at the
top (after any shebang and comments) with `--package` or `--script`:

```scrippo
@info
  --name is-even
  --version 1.2.0
  --description "Checks whether an integer is even"
  --author "Ada Lovelace"
  --license MIT
  --repository "https://github.com/example/helpers"
  --package
  --autoload enable

is_even(value<int>)
  [[ $((value % 2)) == 0 ]]
```

The indented `@info` block accepts `--name`, `--description`, `--version`, `--author`,
`--license`, `--repository`, mutually exclusive `--script` / `--package`, and
`--autoload enable|disable` (default: `enable`). Text options take one literal
value; quote text containing spaces or punctuation. Shell interpolation and
command substitutions are rejected. Options use the same indentation and may
share a line or span lines; trailing `#` comments are allowed. Metadata emits no runtime code.

All options are optional for ordinary files. Installable entries require a kind
(`--package` or `--script`), a name matching the filename without `.scrippo`, and
a stable `MAJOR.MINOR.PATCH` version matching the selected tag without its
optional `v`. All metadata is retained in the installation manifest. Parentheses
and inline or positional arguments are rejected; the next statement at indent 0 ends the metadata block. Shared support files simply omit
`@info`. Bare package imports during compilation resolve from the importing
file's directory, the repository root, `packages/`, or `.scrippo/lib/`; relative
imports and file globs can reference other files inside the repository.

Commit the source, create a tag such as `git tag v1.2.0`, then push the commit
and tag to your Git host. No registry or build hook is required; to make the entry
installable by name, add it to a registry.

## Config & autoload

```bash
scrippo --configure                          # print settings
scrippo --configure autoload=true autocomplete=true
scrippo --configure alias=s                  # `s build` == `scrippo build`, with completion
```

`alias` takes comma-separated command names (`alias=s,sc`); `alias=` clears them. Aliases
are defined by the shell hook below, so they appear in new shells (or after a `cd` in bash),
and `autocomplete=true` gives them the same tab completion as `scrippo`.

Settings live in `~/.scrippo/config` (override with `SCRIPPO_CONFIG` or `SCRIPPO_HOME`).

With `autoload=true`, add this to your shell rc:

```bash
eval "$(scrippo --init)"
```

Entering a directory loads aliases from that directory and all its parents. The nearest directory wins when names conflict:

- `package.scrippo` (or any scrippo-shebang file) as `package` → `scrippo package.scrippo`
- each top-level command in `.scrippo/commands.scrippo` (e.g. `build`) as `build` → `scrippo .scrippo/commands.scrippo build`
- global scripts in `~/.scrippo/scripts/` (local names win on conflict)

With `autocomplete=true`, tab completion covers those aliases' commands and `--` options.
It also covers `scrippo` itself: modes (only what fits what you have
typed: `scrippo -c <tab>` no longer offers `--noexec`), and once a script is named,
that script's own commands and options (`scrippo app.scrippo <tab>`).

Put `--autoload disable` in the indented `@info` block at the top of a file to
omit its aliases and shell completions, including commands in `.scrippo/commands.scrippo`. Direct execution
and imports still work. Per-file `enable` follows the global autoload setting.

## Editor support

- VS Code: [dev/vscode-ext](dev/vscode-ext) (`make build-extension`)
- Neovim: [dev/nvim](dev/nvim) — syntax highlighting, indentation, and autocomplete (`make install-nvim`)

## Install

```bash
make install   # installs scrippo, scrippo-core (+ scriptopotamus symlink) to /usr/local/bin
```

## Examples

See [docs/examples/](docs/examples/).

## Aliases, scoped imports, and usage

Use `/` for flags that name the same option, and `|` for mutually exclusive options:

```scrippo
.files([-ls / --list | -rm / --remove])
  if $--list
    -> listing
```

Import commands into a command or function scope with `--as`. CLI completion follows
command scopes, so `projects project-one <tab>` lists the imported commands:

```scrippo
@import projects/project-one.scrippo --as .project-one
@import projects/project-one.scrippo --as project-one-helper
```

Nested commands can copy their parent's parameters and options with `@inherit`.
Add `#?` documentation above or beside commands, functions, parameters, and options:

```scrippo
#? Resolve a path
.get-path(
  path #? Path to resolve
  [--verbose] #? Print details
)
  #? Resolve an absolute path
  .abs(@inherit)
    -> $(realpath "$path")
```

Usage shows nesting: a group's nested options are indented beneath its leading option, and the
alternatives of an exclusive set are bulleted under `one of:`.

A `*rest` parameter after the positionals takes everything from the point the positionals are
filled verbatim, options included, like a trailing var-arg (`scrippo` uses it so script
arguments are never interpreted).

`--help` prints usage and exits successfully, unless explicitly declared as an
option. Missing commands and invalid arguments print usage with their errors.
`@usage [code]` prints usage for the current script, command, or function and exits
with the given code (default `1`); `@usage --no-exit` prints usage and continues.
Usage uses colors on a terminal and respects `NO_COLOR`.
