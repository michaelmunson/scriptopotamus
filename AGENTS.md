# Agent instructions

When a change affects scrippo syntax, semantics, the CLI, packages, scripts, install, shell integration, or editor behavior, always update all four of these in the same change. Do not leave any of them for a follow-up:

1. The documentation (`docs/pages`)
2. The agent skills (`dev/skills/scrippo`)
3. nvim support (`dev/nvim`)
4. VS Code support (`dev/vscode-ext`)

Update each one even when the change looks small. If a part truly has nothing to change, check it first and say so in your summary.

## Documentation

The language reference is the Docusaurus site in `docs/pages`. Update the page that owns the behavior (`docs/pages/docs/`). Add a page and a sidebar entry in `docs/pages/sidebars.js` when the topic is new.

Scrippo samples use `scrippo` fences so they highlight. Shell commands stay `bash`. Editor snippets stay `lua`. Directory trees stay plain text.

If the change adds or changes tokens the docs highlighter should recognize, update `docs/pages/src/theme/scrippo-prism.js` too.

## Development Tools

### Skills
Always update the skills when a change touches anything they describe. The agent skill for writing scrippo lives in `dev/skills/scrippo`. `SKILL.md` is the router and cheat sheet; nested skills each own one area. Update the one that matches the change:

- Syntax, types, values, control flow, functions: `syntax/SKILL.md`
- Commands, parameters, usage text, `@inherit`, `@hook exec`/`exit`: `commands/SKILL.md`
- Standard library (`prt`, `input`, `throw`, `list`, `dict`, `str`, `here`, `config`): `stdlib/SKILL.md`
- Imports, packages, `@info`, `scrippolib`, install hooks, publishing: `packages/SKILL.md`
- CLI flags, config, shell integration, environment, errors: `tooling/SKILL.md`

Add a new nested directory with its own `SKILL.md` (and a row in the router table) when a topic does not fit one of these.

### nvim Support
Editor support for Neovim lives in `dev/nvim`. Update the part that matches the change:

- Syntax highlighting: `dev/nvim/syntax/scrippo.vim`
- Indentation and filetype behavior: `dev/nvim/indent/`, `dev/nvim/ftplugin/`, `dev/nvim/ftdetect/`
- Completion, analysis, and macro/stdlib data: `dev/nvim/lua/scrippo/` and `dev/nvim/lua/cmp_scrippo/`
- Tests next to that behavior in `dev/nvim/test/run.lua`
- `dev/nvim/README.md` when user-visible nvim behavior changes

A syntax or completion change needs a test that fails without the change and passes with it.

### VS Code extension
Editor support lives in `dev/vscode-ext`. Update the part that matches the change:

- Syntax highlighting: `dev/vscode-ext/syntaxes/scriptopotamus.tmLanguage.json`
- Completion, analysis, hover docs, or the embedded compiler: `dev/vscode-ext/src/`
- Tests next to that behavior in `dev/vscode-ext/test/`

A syntax or completion change needs a test that fails without the change and passes with it.

