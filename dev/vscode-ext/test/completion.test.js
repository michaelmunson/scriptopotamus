'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { analyze } = require('../src/analyzer');
const { complete } = require('../src/completion');

const SOURCE = [
  'PI<const float>=3.14',
  'name<str>="x"',
  'circumference(',
  '  --diameter<float> | --radius<float>',
  '  [--verbose]',
  ')',
  '  for part in 1 2',
  '    echo $part',
  '  -> $PI',
  '.build(target)',
  '  echo $target',
];

function at(lines, text, after = []) {
  const source = [...lines, text, ...after];
  return complete(analyze(source.join('\n')), text, lines.length, text.length);
}

function labels(lines, text, after) {
  const found = at(lines, text, after);
  return found ? found.items.map((item) => item.label) : null;
}

test('macros complete after @', () => {
  const found = at(SOURCE, '  @pr');
  assert.equal(found.from, 3);
  assert.deepEqual(found.items.map((i) => i.label), ['throw', 'prt', 'input', 'rootdir', 'import']);
  assert.equal(found.items.find((i) => i.label === 'prt').insertText, 'prt "${1:message}"');
  assert.deepEqual(labels(SOURCE, 'cd @'), ['throw', 'prt', 'input', 'rootdir', 'import']);
  assert.notDeepEqual(labels(SOURCE, 'user@ho'), ['throw', 'prt', 'input', 'rootdir', 'import']);
});

test('variables and params complete after $', () => {
  const inside = SOURCE.slice(0, 9);
  const found = at(inside, '  echo ${di');
  assert.equal(found.from, '  echo ${'.length);
  const items = labels(inside, '  echo $');
  assert.deepEqual(items.slice(0, 3), ['diameter', 'radius', 'verbose']);
  assert.ok(items.includes('PI') && items.includes('name') && items.includes('part'));
  assert.ok(!labels(SOURCE.slice(0, 2), 'echo $').includes('diameter'));
  assert.equal(at(inside, '  echo $').items.find((i) => i.label === 'PI').kind, 'constant');
});

test('option references complete after $-', () => {
  const inside = SOURCE.slice(0, 9);
  const found = at(inside, '  echo $--ra');
  assert.equal(found.from, '  echo $'.length);
  assert.deepEqual(found.items.map((i) => i.label), ['--diameter', '--radius', '--verbose']);
});

test('types complete inside annotations', () => {
  assert.deepEqual(labels([], 'x<'), ['const', 'int', 'float', 'str', 'upper', 'lower', 'bool', 'list', 'dict']);
  assert.ok(!labels([], 'x<const ').includes('const'));
  assert.deepEqual(labels([], 'x<list '), ['int', 'float', 'str', 'upper', 'lower', 'bool']);
  assert.deepEqual(labels([], 'x<list int '), []);
  assert.equal(at([], 'x<dict int fl').from, 'x<dict int '.length);
  assert.deepEqual(labels(['f('], '  --opt<', [')', '  echo']), labels([], 'x<'));
  assert.deepEqual(labels(['f(', '  [--opt<str>]', ')'], '  echo a<'), null);
  assert.equal(at([], 'x<str {a b'), null);
  assert.equal(at([], 'x<int=4'), null);
});

test('function flags complete after a call', () => {
  assert.deepEqual(labels(SOURCE, 'circumference --'), ['--diameter', '--radius', '--verbose']);
  assert.deepEqual(labels(SOURCE, 'x=$(circumference --verbose -'), ['--diameter', '--radius']);
  assert.deepEqual(labels(SOURCE, '@input "Pick" --'), ['-v', '-s', '--select', '--multi', '--confirm']);
  assert.deepEqual(labels(SOURCE, 'unknown --'), []);
  assert.equal(at(SOURCE, 'circumference 3'), null);
});

test('statements complete keywords, functions, and variables', () => {
  const items = labels(SOURCE, 'ci');
  assert.ok(items.includes('circumference') && items.includes('build') && items.includes('@prt'));
  assert.ok(items.includes('if') && items.includes('for') && items.includes('PI'));
  assert.ok(items.includes('len'));
  const nested = labels(SOURCE, 'if ci');
  assert.ok(nested.includes('circumference') && !nested.includes('if') && !nested.includes('PI'));
  assert.ok(labels(SOURCE, 'echo a | ').includes('circumference'));
  assert.equal(at(SOURCE, 'if').items.find((i) => i.label === 'if').insertText, 'if ${1:condition}\n\t$0');
});

test('nothing completes in comments or single quotes', () => {
  assert.equal(at(SOURCE, 'echo hi # $'), null);
  assert.equal(at(SOURCE, "echo '$"), null);
  assert.ok(at(SOURCE, 'echo "$').items.length);
});
