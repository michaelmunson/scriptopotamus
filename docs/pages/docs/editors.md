---
title: Editors
description: Syntax highlighting, indentation, and completion in VS Code and Neovim.
---

Editor support lives in the repository under `dev/`. Both editors treat `.scrippo` and `.scriptopotamus` as scrippo, and they also detect a shebang that contains `scrippo`.

## VS Code

The extension is `dev/vscode-ext`. Build a package with:

```bash
make build-extension
```

That writes a `.vsix` in `dev/vscode-ext`. The extension contributes:

- Syntax highlighting, including `#` comments, `#?` documentation comments, strings, types, commands, and macros such as `@import`.
- Syntax highlighting inside Markdown fenced code blocks labeled `scrippo` or `scriptopotamus`, using backticks or tildes.
- Indentation defaults: insert spaces, tab size 2, trim trailing whitespace.
- The command **Scriptopotamus: Show Compiled Bash**, from the editor title or the command palette, while a scrippo file is open.
- Diagnostics from the `scrippo-core` compiler.

Settings:

| Setting | Default | Meaning |
| --- | --- | --- |
| `scriptopotamus.lint.enable` | `true` | Lint while you edit. |
| `scriptopotamus.compiler.run` | `onSave` | `onSave`, `onType`, or `off`. `onType` runs shortly after you stop typing. `onSave` also runs when a file is opened. |
| `scriptopotamus.compiler.path` | empty | Path to `scrippo-core` or the `scrippo` wrapper. Supports `${workspaceFolder}` and `~`. Empty searches `target/debug/scrippo-core` and `target/release/scrippo-core`, then `target/debug/scrippo` and `target/release/scrippo`. |

Diagnostics run the compiler on workspace files, so they are limited to trusted workspaces. `scriptopotamus.compiler.path` is a restricted setting in untrusted workspaces.

## Neovim

The plugin is `dev/nvim`. It targets Neovim 0.9 or newer.

```bash
make install-nvim
```

That symlinks the plugin into `~/.local/share/nvim/site/pack/scriptopotamus/start/scrippo.nvim` (`$XDG_DATA_HOME` when it is set). With lazy.nvim you can point at the directory:

```lua
{ dir = '/path/to/scriptopotamus/dev/nvim', ft = 'scrippo' }
```

Highlighting covers comments, `#?`, strings, `$var` / `${var}` / `$--opt`, command substitutions, typed declarations, command and function headers, `@import ... --as`, control flow, and `->`. Groups link to the standard highlight names, so they follow the colorscheme.

Completion depends on where the cursor is:

| Context | Completes |
| --- | --- |
| Start of a statement | Functions, commands, imported std functions, macros, keywords |
| `@` | Built-in macros |
| `$` / `${` | Parameters and variables in scope |
| `$--` | Options of the enclosing command or function |
| `name<` | `const`, `int`, `float`, `str`, `bool`, `list`, `dict`, `any` |
| `@import ` | `scrippo/std/*`, project and global packages, local `.scrippo` files |
| `@import x --` | `--as` and scope names |
| After a command name | Nested commands |
| After a command and `--` | That command's options, including `prt` and `input` when imported |

Omni completion is set for the filetype, so `<C-x><C-o>` works. Opt into a popup:

```lua
vim.g.scrippo_autotrigger = true
vim.o.completeopt = 'menu,menuone,noselect'
```

Or `require('scrippo').setup({ autotrigger = true, min_chars = 2 })`. For nvim-cmp, a source named `scrippo` is registered when a scrippo buffer opens and nvim-cmp is already loaded. Call `require('scrippo').setup()` after cmp if cmp loads later.

```lua
cmp.setup.filetype('scrippo', {
  sources = cmp.config.sources({ { name = 'scrippo' } }, { { name = 'buffer' } }),
})
```

Snippet items expand through cmp's snippet engine. Other engines that can call `omnifunc` work as well.

```bash
make test-nvim
```

runs `nvim --headless -u NONE -l dev/nvim/test/run.lua`.
