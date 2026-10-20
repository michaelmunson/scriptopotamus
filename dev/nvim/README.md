# Scriptopotamus for Neovim

Syntax highlighting, indentation, and autocomplete for `.scrippo` files in Neovim (0.9+).

## Install

Symlink it as a native package:

```bash
make install-nvim   # -> ~/.local/share/nvim/site/pack/scriptopotamus/start/scrippo.nvim
```

Or point your plugin manager at this directory, e.g. with lazy.nvim:

```lua
{ dir = '/path/to/scriptopotamus/dev/nvim', ft = 'scrippo' }
```

Files are detected by the `.scrippo` / `.scriptopotamus` extension, or by a `scrippo` shebang.

## Syntax highlighting

`syntax/scrippo.vim` highlights the whole language: `#` and `#?` comments, strings, `$var` / `${var}` / `$--opt`
references, `$( )` substitutions, typed declarations, command / macro / function headers and their parameters,
`@macro` calls, `@import ... --as`, `if` / `for` / `while` / `case`, and `->`.
It links to the standard highlight groups, so it follows your colorscheme.

## Autocomplete

Completion is context aware and understands the buffer you are editing:

| Context | Completes |
| --- | --- |
| statement start | functions, top-level commands, std functions you imported, `@macros`, keywords |
| `@` | builtin and locally defined macros |
| `$` / `${` | parameters and variables in scope |
| `$--` | options of the enclosing command or function |
| `name<` / `param<` | `const`, `int`, `float`, `str`, `bool`, `list`, `dict`, `any` |
| `@import ` | `scrippo/std/*`, project / global packages, local `.scrippo` files |
| `@import x --` / `--as ` | `--as`, `.command` / function scope |
| `cmd <tab>` | nested sub-commands |
| `cmd --` | options of that command or function (including `prt`, `input`, ... from the std library) |

### Omni completion

The ftplugin sets `omnifunc`, so `<C-x><C-o>` works out of the box.

### Auto popup

Opt in to a popup as you type (set this before opening a scrippo file):

```lua
vim.g.scrippo_autotrigger = true
-- or: require('scrippo').setup({ autotrigger = true, min_chars = 2 })
vim.o.completeopt = 'menu,menuone,noselect'
```

### nvim-cmp

A source named `scrippo` is registered when a scrippo buffer opens and nvim-cmp is already loaded (otherwise call
`require('scrippo').setup()` after loading cmp). Add it to your sources:

```lua
cmp.setup.filetype('scrippo', {
  sources = cmp.config.sources({ { name = 'scrippo' } }, { { name = 'buffer' } }),
})
```

Snippet-style items (keywords, macros, std functions) expand through whichever snippet engine cmp is configured with.

Other completion frameworks that can use `omnifunc` (e.g. `cmp-omni`, blink.cmp's `omni` source) work as well.

## Development

```bash
make test-nvim   # nvim --headless -u NONE -l dev/nvim/test/run.lua
```
