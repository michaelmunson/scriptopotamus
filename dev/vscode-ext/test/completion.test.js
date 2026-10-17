'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
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

const NESTED = [
  '.version()',
  '  .get()',
  '    -> 1',
  '  .bump(version [--dry])',
  '    -> $version',
  '  .demo()',
  '    version get',
];

function at(lines, text, after = [], options = {}) {
  const source = [...lines, text, ...after];
  return complete(analyze(source.join('\n')), text, lines.length, text.length, options);
}

function labels(lines, text, after, options) {
  const found = at(lines, text, after, options);
  return found ? found.items.map((item) => item.label) : null;
}

test('macros complete after @', () => {
  const found = at(SOURCE, '  @im');
  assert.equal(found.from, 3);
  assert.deepEqual(found.items.map((i) => i.label), ['import']);
  assert.equal(found.items[0].insertText, 'import ${1:scrippo/std}');
  const defined = ['@say()', '  -> "hi"', '@'];
  assert.deepEqual(labels(defined.slice(0, 2), defined[2]), ['import', 'say']);
  assert.notDeepEqual(labels(SOURCE, 'user@ho'), ['import']);
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
  assert.deepEqual(labels([], 'x<'), ['const', 'int', 'float', 'str', 'bool', 'list', 'dict', 'any']);
  assert.ok(!labels([], 'x<const ').includes('const'));
  assert.deepEqual(labels([], 'x<const '), ['int', 'float', 'str', 'bool', 'list', 'dict', 'any']);
  assert.deepEqual(labels([], 'x<list '), []);
  assert.deepEqual(labels(['f('], '  --opt<', [')', '  echo']), labels([], 'x<'));
  assert.deepEqual(labels(['f(', '  [--opt<str>]', ')'], '  echo a<'), null);
  assert.equal(at([], 'x<str {a b'), null);
  assert.equal(at([], 'x<int=4'), null);
});

test('function flags complete after a call', () => {
  assert.deepEqual(labels(SOURCE, 'circumference --'), ['--diameter', '--radius', '--verbose']);
  assert.deepEqual(labels(SOURCE, 'x=$(circumference --verbose -'), ['--diameter', '--radius']);
  assert.deepEqual(labels(SOURCE, 'input "Pick" --'), []);
  assert.deepEqual(labels(['@import scrippo/std/io'], 'input "Pick" --'), ['-v', '--select', '--multi', '--confirm']);
  assert.deepEqual(labels(['@import scrippo/io'], 'prt -'), ['-s', '--style', '-j', '--join', '-n']);
  assert.deepEqual(labels(SOURCE, 'unknown --'), []);
  assert.equal(at(SOURCE, 'circumference 3'), null);
});

test('statements complete keywords, functions, and variables', () => {
  const items = labels(SOURCE, 'ci');
  assert.ok(items.includes('circumference') && items.includes('build') && items.includes('@import'));
  assert.ok(items.includes('if') && items.includes('for') && items.includes('PI'));
  assert.ok(!items.includes('prt') && !items.includes('list') && !items.includes('here'));
  const nested = labels(SOURCE, 'if ci');
  assert.ok(nested.includes('circumference') && !nested.includes('if') && !nested.includes('PI'));
  assert.ok(labels(SOURCE, 'echo a | ').includes('circumference'));
  assert.equal(at(SOURCE, 'if').items.find((i) => i.label === 'if').insertText, 'if ${1:condition}\n\t$0');
});

test('stdlib functions complete only after their package is imported', () => {
  assert.ok(!labels(SOURCE, 'pr').includes('prt'));
  const all = labels(['@import scrippo/std'], 'pr');
  assert.ok(all.includes('prt') && all.includes('throw') && all.includes('input'));
  assert.ok(all.includes('list') && all.includes('dict') && all.includes('str') && all.includes('here'));
  const io = labels(['@import scrippo/std/io'], 't');
  assert.ok(io.includes('throw') && io.includes('prt') && !io.includes('list') && !io.includes('here'));
  const types = labels(['@import "scrippo/types" # types only'], 'l');
  assert.ok(types.includes('list') && !types.includes('prt') && !types.includes('here'));
  const nested = labels(['f()', '  @import scrippo/std'], '  pr');
  assert.ok(!nested.includes('prt'));
});

test('packages complete after @import', () => {
  const found = at(SOURCE, '@import ');
  assert.equal(found.from, '@import '.length);
  assert.deepEqual(
    found.items.filter((item) => item.detail !== 'local file').map((item) => item.label),
    [
      'scrippo/std',
      'scrippo/std/io',
      'scrippo/std/types',
      'scrippo/std/path',
      'scrippo/io',
      'scrippo/types',
      'scrippo/path',
      'scrippo/*',
    ],
  );
  assert.equal(at(SOURCE, '@import scrippo/std/').from, '@import '.length);
  assert.equal(at(SOURCE, '@import "scrippo/io').items.find((item) => item.label === 'scrippo/io').insertText, 'scrippo/io"');
});

test('nested commands complete after their parent', () => {
  assert.deepEqual(labels(NESTED, 'version '), ['get', 'bump', 'demo']);
  const partial = at(NESTED, 'version g');
  assert.equal(partial.from, 'version '.length);
  assert.deepEqual(partial.items.map((i) => i.label), ['get', 'bump', 'demo']);
  assert.ok(!labels(NESTED, 'ver').includes('get'));
  assert.deepEqual(labels(NESTED, 'version bump --'), ['--dry']);
  assert.equal(at(NESTED, 'version get '), null);
});

test('local .scrippo files complete after @import', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'scrippo-import-'));
  try {
    fs.writeFileSync(path.join(dir, 'util.scrippo'), '-> 1\n');
    fs.mkdirSync(path.join(dir, 'lib'));
    fs.writeFileSync(path.join(dir, 'lib', 'helper.scrippo'), '-> 2\n');
    const found = at([], '@import ./', [], { importDir: dir });
    assert.deepEqual(
      found.items.map((item) => item.label),
      ['./lib/helper.scrippo', './util.scrippo'],
    );
    assert.deepEqual(labels([], '@import util', [], { importDir: dir }), ['./util.scrippo']);
    assert.ok(labels([], '@import ', [], { importDir: dir }).includes('./util.scrippo'));
    assert.ok(labels([], '@import ', [], { importDir: dir }).includes('scrippo/std'));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test('nothing completes in comments or single quotes', () => {
  assert.equal(at(SOURCE, 'echo hi # $'), null);
  assert.equal(at(SOURCE, "echo '$"), null);
  assert.ok(at(SOURCE, 'echo "$').items.length);
});
