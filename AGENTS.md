# Agent instructions

When a change affects scrippo syntax, semantics, the CLI, packages, scripts, install, shell integration, or editor behavior, update the documentation and the VS Code extension in the same change. Do not leave either for a follow-up.

## Documentation

The language reference is the Docusaurus site in `docs/pages`. Update the page that owns the behavior (`docs/pages/docs/`). Add a page and a sidebar entry in `docs/pages/sidebars.js` when the topic is new.

Scrippo samples use `scrippo` fences so they highlight. Shell commands stay `bash`. Editor snippets stay `lua`. Directory trees stay plain text.

If the change adds or changes tokens the docs highlighter should recognize, update `docs/pages/src/theme/scrippo-prism.js` too.

## Development Tools
### Skills

The agent skill for writing scrippo lives in `dev/skills/scrippo`. `SKILL.md` is the router and cheat sheet; nested skills each own one area. Update the one that matches the change:

- Syntax, types, values, control flow, functions: `syntax/SKILL.md`
- Commands, parameters, usage text, `@inherit`, `@hook exec`/`exit`: `commands/SKILL.md`
- Standard library (`prt`, `input`, `throw`, `list`, `dict`, `str`, `here`, `config`): `stdlib/SKILL.md`
- Imports, packages, `@info`, `scrippolib`, install hooks, publishing: `packages/SKILL.md`
- CLI flags, config, shell integration, environment, errors: `tooling/SKILL.md`

Add a new nested directory with its own `SKILL.md` (and a row in the router table) when a topic does not fit one of these.

### nvim Support

### VS Code extension
Editor support lives in `dev/vscode-ext`. Update the part that matches the change:

- Syntax highlighting: `dev/vscode-ext/syntaxes/scriptopotamus.tmLanguage.json`
- Completion, analysis, hover docs, or the embedded compiler: `dev/vscode-ext/src/`
- Tests next to that behavior in `dev/vscode-ext/test/`

A syntax or completion change needs a test that fails without the change and passes with it.

