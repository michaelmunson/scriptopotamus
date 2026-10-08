'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { tokenize } = require('./tokenize');

async function scopeOf(source, text, occurrence = 0) {
  const lines = await tokenize(source);
  const matches = lines.flat().filter((t) => t.text === text);
  assert.ok(matches[occurrence], `token ${JSON.stringify(text)} not found`);
  return matches[occurrence].scopes.join(' ');
}

test('typed declarations', async () => {
  const src = 'PI<const float>=3.14';
  assert.match(await scopeOf(src, 'PI'), /variable\.other\.declaration/);
  assert.match(await scopeOf(src, 'const'), /storage\.modifier\.const/);
  assert.match(await scopeOf(src, 'float'), /support\.type\.primitive/);
  assert.match(await scopeOf(src, '3.14'), /constant\.numeric/);
});

test('literal types', async () => {
  const src = 'mode<str (fast slow)>';
  assert.match(await scopeOf(src, 'fast'), /constant\.other\.literal/);
});

test('function definitions with multi-line parameters', async () => {
  const src = ['my_func(', '  path', '  [-s | --select<str>...]', '  --name<str="mike"> # note', ')', '  echo $path'].join('\n');
  assert.match(await scopeOf(src, 'my_func'), /entity\.name\.function/);
  assert.match(await scopeOf(src, 'path'), /variable\.parameter/);
  assert.match(await scopeOf(src, '--select'), /variable\.parameter\.option/);
  assert.match(await scopeOf(src, '...'), /keyword\.operator\.repeat/);
  const starred = ['f(', '  [*rest]', ')', '  echo'].join('\n');
  assert.match(await scopeOf(starred, '*'), /keyword\.operator\.rest/);
  assert.match(await scopeOf(starred, 'rest'), /variable\.parameter\.rest/);
  assert.match(await scopeOf(src, '|'), /keyword\.operator\.exclusive/);
  assert.match(await scopeOf(src, 'mike'), /string\.quoted\.double/);
  assert.match(await scopeOf(src, ' note'), /comment\.line/);
  assert.match(await scopeOf(src, 'echo'), /support\.function\.builtin/);
});

test('commands', async () => {
  const src = '.deploy(env)\n  -> "$env"';
  assert.match(await scopeOf(src, 'deploy'), /entity\.name\.function\.command/);
  assert.match(await scopeOf('.bump-version(v)\n  -> 1', 'bump-version'), /entity\.name\.function\.command/);
  assert.match(await scopeOf('my-fn(v)\n  -> 1', 'my-fn'), /entity\.name\.function\.scriptopotamus/);
  assert.match(await scopeOf(src, '->'), /keyword\.control\.echo/);
});

test('control flow', async () => {
  const src = ['if -n $x', '  echo a', 'elif $y', '  echo b', 'else', '  echo c', 'for i in $list', '  echo $i', 'while true', '  break', 'fi'].join('\n');
  assert.match(await scopeOf(src, 'if'), /keyword\.control\.conditional/);
  assert.match(await scopeOf(src, '-n'), /keyword\.operator\.test/);
  assert.match(await scopeOf(src, 'elif'), /keyword\.control\.conditional/);
  assert.match(await scopeOf(src, 'else'), /keyword\.control\.conditional/);
  assert.match(await scopeOf(src, 'for'), /keyword\.control\.loop/);
  assert.match(await scopeOf(src, 'in'), /keyword\.control\.in/);
  assert.match(await scopeOf(src, 'while'), /keyword\.control\.loop/);
  assert.match(await scopeOf(src, 'true'), /constant\.language\.boolean/);
  assert.match(await scopeOf(src, 'break'), /keyword\.control\.flow/);
  assert.match(await scopeOf(src, 'fi'), /invalid\.illegal/);
});

test('case', async () => {
  const src = ['case $x in', '  hello | "a b") -> 1 ;;', '  @(x|y)*)', '    -> 2', 'esac', 'other)'].join('\n');
  assert.match(await scopeOf(src, 'case'), /keyword\.control\.case/);
  assert.match(await scopeOf(src, 'in'), /keyword\.control\.in/);
  assert.match(await scopeOf(src, 'hello'), /meta\.case\.entry\.pattern.*string\.regexp\.unquoted/);
  assert.match(await scopeOf(src, '|'), /keyword\.operator\.alternation/);
  assert.match(await scopeOf(src, 'a b'), /meta\.case\.entry\.pattern.*string\.quoted\.double/);
  assert.match(await scopeOf(src, ')'), /keyword\.operator\.pattern\.case/);
  assert.match(await scopeOf(src, ';;'), /punctuation\.terminator\.statement\.case/);
  assert.match(await scopeOf(src, 'x', 1), /string\.regexp\.unquoted/);
  assert.match(await scopeOf(src, '*'), /keyword\.operator\.quantifier/);
  assert.doesNotMatch(await scopeOf(src, '2'), /meta\.case\.entry\.pattern/);
  assert.match(await scopeOf(src, 'esac'), /invalid\.illegal/);
  assert.doesNotMatch(await scopeOf(src, 'other)'), /meta\.case/);
});

test('variables, option references, and indexing', async () => {
  const src = 'echo $name $--opt "$dict[key]" ${arr[@]} $1';
  assert.match(await scopeOf(src, 'name'), /variable\.other/);
  assert.match(await scopeOf(src, '--opt'), /variable\.parameter\.option\.reference/);
  assert.match(await scopeOf(src, 'key'), /string\.unquoted\.key/);
  assert.match(await scopeOf(src, '1'), /variable\.language\.special/);
});

test('substitutions and calls', async () => {
  const src = 'echo $(circumference --diameter 2)';
  assert.match(await scopeOf(src, 'circumference'), /entity\.name\.function\.call/);
  assert.match(await scopeOf(src, '--diameter'), /constant\.other\.option/);
  assert.match(await scopeOf(src, '2'), /meta\.embedded\.substitution/);
});

test('statement calls and macros', async () => {
  const src = 'deploy --env prod\n@throw 1 "bad"\nf()\n  @prt -s red "$@"';
  assert.match(await scopeOf(src, 'deploy'), /entity\.name\.function\.call/);
  assert.match(await scopeOf(src, '@'), /entity\.name\.function\.macro\.scriptopotamus/);
  assert.match(await scopeOf(src, 'throw'), /entity\.name\.function\.macro\.scriptopotamus/);
  assert.match(await scopeOf(src, 'prt'), /entity\.name\.function\.macro\.scriptopotamus/);
  assert.doesNotMatch(await scopeOf(src, '@', 2), /macro/);
});

test('shebang and comments', async () => {
  const src = '#!/usr/bin/env scriptopotamus\necho $# # count';
  assert.match(await scopeOf(src, '#!/usr/bin/env scriptopotamus'), /comment\.line\.shebang/);
  assert.match(await scopeOf(src, '#', 0), /variable\.language\.special/);
  assert.match(await scopeOf(src, ' count'), /comment\.line\.number-sign/);
});
