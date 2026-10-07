'use strict';

const BUILTINS = {
  throw: {
    signature: 'throw(code<int> msg)',
    description: 'Exits with `code`, printing `msg` as a red error message.',
  },
  prt: {
    signature: 'prt(\n  msg...\n  [-s|--style <str>...]...\n  [-n]\n)',
    description: 'Echo with style.',
  },
  input: {
    signature: 'input(\n  prompt\n  -v<str>\n  [\n    --select\n    --options<str>...\n    [--multi]\n  ] | [\n    --confirm\n  ]\n)',
    description: 'Wrapper around `read`: prompts for text, a selection from options, or a confirmation, storing the answer in `-v`.',
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
