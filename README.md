# Scriptopotamus

Scriptopotamus (`scrippo`) compiles `.scrippo` to bash. See [docs/vision.md](docs/vision.md) for the language.

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
scrippo -n script.scrippo          # compile + bash -n
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
scrippo --new --package [--local|--global] <name>   # default: --local
scrippo --new --script  [--local|--global] <name>   # default: --global
```

Project commands live in `.scrippo/commands.scrippo` (a legacy `.scrippo` file still works).

### Installing from Git or a directory

```bash
scrippo -i -p https://github.com/owner/helpers.git         # project .scrippo/lib
scrippo -i -p -g https://github.com/owner/helpers.git      # ~/.scrippo/lib
scrippo -i -s -g https://github.com/owner/helpers.git      # ~/.scrippo/scripts
scrippo -i -s https://github.com/owner/helpers.git ./tools # explicit destination
scrippo -i -p ./helpers                                    # plain local directory
scrippo --install --package --tag v1.2.0 https://github.com/owner/helpers.git
```

Use `-p | --package` or `-s | --script`. Installs default to local: packages go
in the nearest project's `.scrippo/lib` (created in the current directory if
needed), and scripts go in the current directory. `-g | --global` follows
`SCRIPPO_HOME`; it cannot be combined with an explicit destination. Put scrippo
options before the source path; arguments after a script filename belong to that
script.

The source may be a Git URL/repository or a plain directory with the same
`packages/` and `scripts/` layout. Git installs need Git and Bash and select the
highest stable `MAJOR.MINOR.PATCH` tag (with or without a `v` prefix), ignoring
prereleases and untagged commits. `--tag` selects an exact tag. Plain directories
skip Git and use each entry's `@info` version when `--tag` is omitted (all
selected entries must share one version).

If the source contains `.install.scrippo`, the installer runs it twice: once
before publishing (`before`) and once after (`after`). Top-level statements run
on both passes; put phase-specific work in `.before()` / `.after()`:

```scrippo
-> "runs before and after"

.before()
  -> "prepare"

.after()
  -> "finish"
```

Packages are compiled to `<name>.bash` with all imports inlined, checked with
`bash -n`, and exposed through the existing `@import <name>` lookup. Source
packages still work; project packages take precedence over global packages.
Compiled libraries expose named functions and commands as callable Bash
functions, without taking over the importing script's CLI dispatch or `--help`.
Executable root commands (`.()`) belong in scripts.

Scripts keep the complete repository source tree. `<name>.scrippo` links into
that snapshot, preserving relative imports, `here`, global name lookup, autoload,
and command completion. Links are relative so a local installation can move
with its project. Windows script installs require permission to create symlinks.

Snapshots, compiled outputs, and a JSON manifest recording the repository, tag,
commit, and entries are stored under the destination's
`.scrippo-installed/<packages|scripts>/<repo>/content/`. Existing files and
repository installations are never overwritten. Choose a separate destination
to try another version. Validation and compilation finish before publication;
failed installs clean up their own outputs.

Package compilation never executes repository code. Its compiler sandbox permits
imports only from the downloaded repository and bundled standard library, without
falling back to the installer's local or global packages. Repository symlinks
and submodules are rejected before extraction. This is compiler-level isolation,
not an operating system sandbox.

### Publishing installable entries

Use this repository layout (nested directories are allowed, with unique names):

```text
helpers/
├── packages/is-even.scrippo
├── scripts/greet.scrippo
└── support/common.scrippo
```

Every `.scrippo` file under the selected `packages/` or `scripts/` tree needs
exactly one metadata declaration at the top (after any shebang and comments):

```scrippo
@info(
  --name is-even
  --version 1.2.0
  --description "Checks whether an integer is even"
  --author "Ada Lovelace"
  --license MIT
  --repository "https://github.com/example/helpers"
  --package
  --autoload enable
)

is_even(value<int>)
  [[ $((value % 2)) == 0 ]]
```

`@info(...)` accepts `--name`, `--description`, `--version`, `--author`,
`--license`, `--repository`, mutually exclusive `--script` / `--package`, and
`--autoload enable|disable` (default: `enable`). Text options take one literal
value; quote text containing spaces or punctuation. Shell interpolation and
command substitutions are rejected. Options may share a line or span lines;
trailing `#` comments are allowed. Metadata emits no runtime code.

All options are optional for ordinary files. Installable entries require a name
matching the filename without `.scrippo` and a stable `MAJOR.MINOR.PATCH` version
matching the selected tag without its optional `v`. If a kind is declared, it
must match the `packages/` or `scripts/` directory. All metadata is retained in
the installation manifest. The original `@info <name> <version> [<description>]`
form and the single-line `@info --name ... --version ...` form also work.
Shared support files without metadata belong outside the entry directories.
Bare package imports during compilation resolve from the repository's
`packages/` or `.scrippo/lib/`; relative imports and file globs can reference
other files inside the repository.

Commit the source, create a tag such as `git tag v1.2.0`, then push the commit
and tag to your Git host. No registry or build hook is required.

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
It also covers `scrippo` itself: modes and their modifiers (only what fits what you have
typed: `scrippo --new -p <tab>` no longer offers `-s`), and once a script is named,
that script's own commands and options (`scrippo app.scrippo <tab>`).

Put `@info(--autoload disable)` at the top of a file to omit its aliases and shell
completions, including commands in `.scrippo/commands.scrippo`. Direct execution
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
