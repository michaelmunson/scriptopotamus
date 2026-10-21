# scriptopotamus
<img src="dev/vscode-ext/images/icon.png" alt="Scriptopotamus" width="128" height="128">

* scriptopotamus is a scripting language that compiles to bash
* written in bash

## Installation
Requires bash 4+.

```bash
curl -fsSL https://raw.githubusercontent.com/michaelmunson/scriptopotamus/main/install.sh | bash
```

## Shell integration
Turn on directory aliases and tab completion (stored in `~/.scriptopotamus/config.json`):

```bash
scrippo configure --autoload 1 --autocomplete 1
```

Then load the integration from `~/.zshrc` or `~/.bashrc`:

```bash
source <(scrippo __source)
```

On bash 3.2 (the macOS default), use `eval "$(scrippo __source)"` instead.

* **autoload**: on `cd`, scrippo looks at the current directory and its parents, stopping at `$HOME` or `/`
    * `app.scrippo` becomes the alias `app` -> `scrippo /path/to/app.scrippo`
    * a bare `.scrippo` file adds one alias per top-level command, e.g. `.build()` -> `build` -> `scrippo /path/to/.scrippo build`
    * aliases are removed when you leave the project, and existing aliases or functions with the same name are never overwritten
* **autocomplete**: completes commands, nested subcommands, and flags for `scrippo <file>` and for autoloaded aliases, e.g. `app r<TAB>` -> `app run`, `app run d<TAB>` -> `app run dev`

## Packages and scripts

Install packages from a version-tagged Git repository:

```bash
scrippo -i -p https://github.com/owner/helpers.git
scrippo --install --package --tag v1.2.0 https://github.com/owner/helpers.git ./lib
scrippo -i -s https://github.com/owner/helpers.git ./lib
scrippo -i -p -g https://github.com/owner/helpers.git
scrippo -i --help
```

Use exactly one of `-p | --package` and `-s | --script`. The destination
defaults to `./`. `-g | --global` uses `~/.scriptopotamus` and cannot be combined
with a destination. Git is required in addition to Bash 4+, jq, and bc.

The installer selects the highest stable `MAJOR.MINOR.PATCH` tag (with an
optional `v` prefix), or the exact tag passed to `--tag`. Prerelease tags and
untagged commits are ignored. Local Git paths work too.

For a repository named `helpers.git`, installations look like this:

```text
<destination>/
├── packages/helpers/content/
│   ├── is-even.bash
│   └── .scrippo-install.json
└── scripts/helpers/content/
    ├── scripts/greet.scrippo
    ├── ... other repository files, preserving relative imports
    └── .scrippo-install.json
```

Each install creates only the selected kind. Package files are fully compiled
to Bash, including their imports, and checked with `bash -n` before saving.
Compilation runs in a separate process context without executing repository
code; the compiler sandbox confines imports to the downloaded source tree.
This is compiler-level isolation, not an operating system sandbox. Repository
symlinks and submodules are rejected. Script installs retain the repository's
source files, without Git history. Both modes record the repository, tag, and
exact commit in `.scrippo-install.json`.

Import a compiled library or run an installed script explicitly:

```scrippo
@import "lib/packages/helpers/content/is-even.bash"
```

```bash
scrippo ./lib/scripts/helpers/content/scripts/greet.scrippo
```

Existing installation directories are never overwritten. Use a different
destination to try another version. Downloads and compiled outputs are staged;
a failed download, validation, or compilation leaves no installed repository.

### Publishing a repository

Put installable entries under `packages/` or `scripts/` (subdirectories are
allowed). Every `.scrippo` entry in the selected directory must contain exactly
one top-level declaration:

```scrippo
@info is-even 1.2.0 "Checks whether an integer is even"

is_even(value<int>)
  [[ $((value % 2)) == 0 ]]
```

The format is `@info <name> <version> [<description>]`. Values must be literals;
the name must match the filename without `.scrippo`, and the version must match
the repository tag without its optional `v`. Metadata emits no runtime code.
Keep shared support files outside the entry directories when they do not have
their own metadata. Imports can use relative paths and globs within the repo.

```text
helpers/
├── packages/is-even.scrippo
├── scripts/greet.scrippo
└── support/common.bash
```

Commit the files, tag the release with `git tag v1.2.0`, and push the commit
and tag to your Git host. Installation does not run build hooks or publish
anything to a registry. Review installed code before executing it.
