'use strict';

const BUILTINS = {
  throw: {
    signature: 'throw(code<int> msg)',
    description: 'Exits with `code`, printing `msg` as a red error message.',
    macro: true,
    snippet: 'throw ${1:1} "${2:message}"',
    flags: [],
  },
  prt: {
    signature: 'prt(\n  msg...\n  [-s|--style <str>...]...\n  [-j|--join <str>]\n  [-n]\n)',
    description: 'Echo with style. Each `-s` styles only the msg that follows it; msgs are joined by a space, or by `-j`.',
    macro: true,
    snippet: 'prt "${1:message}"',
    flags: ['-s', '--style', '-j', '--join', '-n'],
  },
  input: {
    signature: 'input(\n  prompt\n  -v<str>\n  [\n    --select<str>...\n    [--multi]\n  ] | [\n    --confirm\n  ]\n)',
    description: 'Interactive, styled prompt for text, an arrow-key selection from options (checkboxes with `--multi`), or a y/n confirmation, storing the answer in `-v`.',
    macro: true,
    snippet: 'input "${1:prompt}" -v ${2:answer}',
    flags: ['-v', '-s', '--select', '--multi', '--confirm'],
  },
  rootdir: {
    signature: 'rootdir([--enter])',
    description: 'The absolute directory containing the running scrippo file, regardless of the working directory. Usable alone (prints it, or `cd`s into it with `--enter`) or inside any expression, e.g. `cd @rootdir/dev`.',
    macro: true,
    snippet: 'rootdir',
    flags: ['--enter'],
  },
  import: {
    signature: 'import(pattern)',
    description: 'Inlines another scrippo or bash file. Relative paths and globs resolve from the importing file, e.g. `@import ./lib/*`.',
    macro: true,
    snippet: 'import "${1:file}"',
    flags: [],
  },
  len: {
    signature: 'len(name)',
    description: 'Prints the number of items in a list or dict.',
  },
};

const KEYWORDS = {
  '->': 'Echo the value that follows. `name(args) -> value` defines a one-line function.',
  if: 'Runs the indented block when the condition holds. Blocks are closed by indentation, so there is no `then` or `fi`.',
  elif: 'Alternative branch of an `if`, at the same indentation as the `if`.',
  else: 'Fallback branch of an `if`. Must be alone on its line.',
  for: '`for <name> in <items>` runs the indented block once per item. A single `$list` item iterates over the whole list.',
  while: 'Runs the indented block while the condition holds.',
  case: '`case <value>` matches the value against the indented `pattern)` arms. There is no `in` or `esac`.',
};

const TYPES = {
  int: 'Integer. Assignments are checked against `^-?[0-9]+$`; arithmetic uses `$(( ))`.',
  float: 'Floating point number. Assignments are checked against `^-?[0-9]*\\.?[0-9]+$`; arithmetic uses `bc`.',
  str: 'String. The default type for function arguments.',
  upper: 'String that is converted to upper case on assignment (`declare -u`).',
  lower: 'String that is converted to lower case on assignment (`declare -l`).',
  bool: '`true` or `false`. The default type for function options.',
  list: '`<list T>`: an indexed array, e.g. `nums<list int>=(1 2 3)`.',
  dict: '`<dict K T>`: an associative array, e.g. `ages<dict int str>=(a=1 b=2)`.',
  const: '`<const T>`: read-only (`declare -r`). Reassigning it is a compile error.',
};

module.exports = { BUILTINS, KEYWORDS, TYPES };
