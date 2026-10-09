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

## Overview