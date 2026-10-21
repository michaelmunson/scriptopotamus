local M = {}

M.BUILTINS = {
  throw = {
    signature = 'throw(code<int> msg...)',
    description = 'Exits with `code`, printing `msg` as a red error message. From `@import scrippo/std/io`.',
    snippet = 'throw ${1:1} "${2:message}"',
    flags = {},
    pkg = 'io',
  },
  prt = {
    signature = 'prt(\n  msg...\n  [-s / --style<str>...]\n  [-j / --join<str>]\n  [-n]\n)',
    description = 'Echo with style. Each `-s` styles only the message that follows it. Messages are joined by a space, or by `-j`. From `@import scrippo/std/io`.',
    snippet = 'prt "${1:message}"',
    flags = { '-s', '--style', '-j', '--join', '-n' },
    pkg = 'io',
  },
  input = {
    signature = 'input(\n  prompt\n  -v<str>\n  [\n    --select<str>...\n    [--multi]\n  ] | [\n    --confirm\n  ]\n)',
    description = 'Styled prompt for text, an arrow-key selection (`--multi` for checkboxes), or a y/n confirmation. Stores the answer in `-v`. From `@import scrippo/std/io`.',
    snippet = 'input "${1:prompt}" -v ${2:answer}',
    flags = { '-v', '--select', '--multi', '--confirm' },
    pkg = 'io',
  },
  list = {
    signature = 'list(\n  array<list>\n  [--join<str>]\n  [--length]\n)',
    description = 'Prints a list, joins it with `--join`, or prints its length. From `@import scrippo/std/types`.',
    snippet = 'list $${1:array}',
    flags = { '--join', '--length' },
    pkg = 'types',
  },
  dict = {
    signature = 'dict(\n  dict<dict>\n  [--keys]\n  [--values]\n  [--length]\n)',
    description = 'Prints a dict, or its keys, values, or length. From `@import scrippo/std/types`.',
    snippet = 'dict $${1:dict}',
    flags = { '--keys', '--values', '--length' },
    pkg = 'types',
  },
  str = {
    signature = 'str(\n  string<str>\n  [--length]\n  [--split<str>]\n)',
    description = 'Prints a string, its length, or the parts from splitting on `--split`. From `@import scrippo/std/types`.',
    snippet = 'str "${1:text}"',
    flags = { '--length', '--split' },
    pkg = 'types',
  },
  here = {
    signature = 'here(\n  [--file]\n  [--rel]\n)',
    description = 'Script directory, or script path with `--file`. With `--rel`, the result is relative to the calling directory. From `@import scrippo/std/path`.',
    snippet = 'here',
    flags = { '--file', '--rel' },
    pkg = 'path',
  },
  inherit = {
    signature = '@inherit',
    description = 'Inherit parameters and options from the parent command or function. Use inside a parameter list.',
    macro = true,
    snippet = 'inherit',
  },
  usage = {
    signature = '@usage [code<int=1>] [--no-exit]',
    description = 'Print usage for the current scope, including #? documentation. Exit with code unless --no-exit is given.',
    macro = true,
    snippet = 'usage ${1:0}',
  },
  import = {
    signature = '@import path [--as .command|function]',
    description = 'Inlines another scrippo or bash file, or a standard library (`scrippo/std`, `scrippo/std/io`, `scrippo/std/types`, `scrippo/std/path`). Relative paths and globs resolve from the importing file. Each file is imported once per scope. Use `--as .command` or `--as function` to wrap the imported script.',
    macro = true,
    snippet = 'import ${1:scrippo/std}',
    flags = {},
  },
  info = {
    signature = '@info name version [description]',
    description = 'Installable entry metadata. Use one top-level declaration with literal values. The name matches the filename and the version matches the stable Git tag. Emits no runtime code.',
    macro = true,
    snippet = 'info ${1:name} ${2:1.0.0} "${3:description}"',
    flags = {},
  },
}

M.KEYWORDS = {
  ['if'] = 'Runs the indented block when the condition holds. Blocks are closed by indentation, so there is no `then` or `fi`.',
  elif = 'Alternative branch of an `if`, at the same indentation as the `if`.',
  ['else'] = 'Fallback branch of an `if`. Must be alone on its line.',
  ['for'] = '`for <name> in <items>` runs the indented block once per item. A single `$list` item iterates over the whole list.',
  ['while'] = 'Runs the indented block while the condition holds.',
  case = '`case <value>` matches the value against the indented `pattern)` arms. `in` and `;;` are optional. There is no `esac`.',
}

M.KEYWORD_ORDER = { 'if', 'elif', 'else', 'for', 'while', 'case' }

M.KEYWORD_SNIPPETS = {
  ['if'] = 'if ${1:condition}\n\t$0',
  elif = 'elif ${1:condition}\n\t$0',
  ['else'] = 'else\n\t$0',
  ['for'] = 'for ${1:item} in ${2:items}\n\t$0',
  ['while'] = 'while ${1:condition}\n\t$0',
  case = 'case ${1:value}\n\t${2:pattern})\n\t\t$0',
}

M.TYPES = {
  int = 'Integer. Assignments are checked against `^-?[0-9]+$`. `integer` is an alias. Literals: `<int {1 2 3}>`.',
  float = 'Floating point number. Assignments are checked against `^-?[0-9]*\\.?[0-9]+$`. Arithmetic uses `bc`.',
  str = 'String. The default type for function arguments. `string` is an alias. Literals: `<str {a b c}>`.',
  bool = '`true` or `false`. The default type for function options. `boolean` is an alias.',
  list = 'Indexed array, e.g. `nums<list>=(1 2 3)`. `array` is an alias.',
  dict = 'Associative array, e.g. `ages<dict>=(a=1 b=2)`.',
  any = 'Any value. `<const>` with no other type is `<const any>`.',
  const = '`<const T>` is read-only. Reassigning it is a compile error. `<const>` alone means `<const any>`.',
}

M.TYPE_NAMES = { 'int', 'float', 'str', 'bool', 'list', 'dict', 'any' }

M.PACKAGES = {
  { path = 'scrippo/std', detail = 'io, types, path', description = 'The whole standard library.' },
  { path = 'scrippo/std/io', detail = 'throw, prt, input', description = 'Styled printing, prompts, and throw.' },
  { path = 'scrippo/std/types', detail = 'list, dict, str', description = 'List, dict, and string helpers.' },
  { path = 'scrippo/std/path', detail = 'here', description = 'The running script directory or file.' },
  { path = 'scrippo/io', detail = 'scrippo/std/io', description = 'Alias of `scrippo/std/io`.' },
  { path = 'scrippo/types', detail = 'scrippo/std/types', description = 'Alias of `scrippo/std/types`.' },
  { path = 'scrippo/path', detail = 'scrippo/std/path', description = 'Alias of `scrippo/std/path`.' },
  { path = 'scrippo/*', detail = 'io, types, path', description = 'Every standard library module.' },
}

local ALL = { 'io', 'types', 'path' }
M.PACKAGE_MODULES = {
  ['scrippo/std'] = ALL,
  ['scrippo/std.scrippo'] = ALL,
  ['scrippo/*'] = ALL,
  ['scrippo/std/*'] = ALL,
  ['scrippo/std/io'] = { 'io' },
  ['scrippo/io'] = { 'io' },
  ['scrippo/std/io.scrippo'] = { 'io' },
  ['scrippo/std/types'] = { 'types' },
  ['scrippo/types'] = { 'types' },
  ['scrippo/std/types.scrippo'] = { 'types' },
  ['scrippo/std/path'] = { 'path' },
  ['scrippo/path'] = { 'path' },
  ['scrippo/std/path.scrippo'] = { 'path' },
}

--- Path argument of an `@import` line (first word, quotes removed).
function M.import_path(raw)
  local word = raw:gsub('#.*$', ''):match('^%s*(%S+)') or ''
  local quoted = word:match('^"(.*)"$') or word:match("^'(.*)'$")
  if quoted then
    return quoted
  end
  return (word:gsub('^["\']', ''):gsub('["\']$', ''))
end

function M.modules_for_import(raw)
  return M.PACKAGE_MODULES[M.import_path(raw)] or {}
end

return M
