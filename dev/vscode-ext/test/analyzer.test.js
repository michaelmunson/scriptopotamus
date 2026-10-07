'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const { analyze } = require('../src/analyzer');

const REPO = path.join(__dirname, '..', '..', '..');

function lint(lines) {
  return analyze(lines.join('\n')).diagnostics.map((d) => ({
    line: d.range.start.line + 1,
    col: d.range.start.character + 1,
    severity: d.severity,
    code: d.code,
    message: d.message,
  }));
}

function codes(lines) {
  return lint(lines).map((d) => `${d.line}:${d.code}`);
}

test('vision example has no errors', () => {
  const source = fs.readFileSync(path.join(REPO, 'tests', 'vision.local.scrippo'), 'utf8');
  const errors = analyze(source).diagnostics.filter((d) => d.severity === 'error');
  assert.deepEqual(errors, []);
});

test('simple example is clean', () => {
  const source = fs.readFileSync(path.join(REPO, 'docs', 'examples', 'simple.scrippo'), 'utf8');
  assert.deepEqual(analyze(source).diagnostics, []);
});

test('block closers are errors', () => {
  assert.deepEqual(codes(['if $x', '  echo a', 'fi']), ['3:block-closer']);
  assert.deepEqual(codes(['for i in 1 2', '  echo $i', 'done']), ['3:block-closer']);
});

test('elif and else must line up with their if', () => {
  assert.deepEqual(codes(['if $x', '  echo a', '  else', '  echo b']), ['3:orphan-branch']);
  assert.deepEqual(codes(['else', '  echo b']), ['1:orphan-branch']);
});

test('else must be alone on its line', () => {
  const [d] = lint(['if $x', '  echo a', 'else # note', '  echo b']);
  assert.equal(d.code, 'else-trailing');
  assert.match(d.message, /not even a comment/);
  assert.equal(lint(['if $x', '  echo a', 'else $y', '  echo b'])[0].code, 'else-trailing');
});

test('comments after conditions are flagged', () => {
  assert.deepEqual(codes(['if $x # why', '  echo a']), ['1:trailing-comment']);
  assert.deepEqual(codes(['for i in 1 2 # why', '  echo $i']), ['1:trailing-comment']);
});

test('conditions are required', () => {
  assert.deepEqual(codes(['if', '  echo a']), ['1:missing-condition']);
  assert.deepEqual(codes(['for i', '  echo a']), ['1:bad-for']);
});

test('trailing whitespace is flagged because the compiler counts it as indentation', () => {
  const [d] = lint(['if $x   ', '  echo a']);
  assert.equal(d.code, 'trailing-whitespace');
  assert.equal(d.col, 6);
});

test('indentation style and consistency', () => {
  assert.deepEqual(codes(['if $x', '  echo a', 'if $y', '\techo b']), ['4:indent-style']);
  assert.deepEqual(codes(['x=1', '  y=2']), ['2:indentation']);
  assert.deepEqual(codes(['if $x', '    echo a', '  echo b']), ['3:indentation']);
});

test('empty blocks are flagged', () => {
  assert.deepEqual(codes(['if $x', 'echo a']), ['1:empty-block']);
  assert.deepEqual(codes(['f(a)', 'echo a']), ['1:empty-block']);
  assert.deepEqual(codes(['f(a) -> $a']), []);
});

test('unknown and malformed types', () => {
  assert.deepEqual(codes(['x<integer>=1']), ['1:type']);
  assert.deepEqual(codes(['x<int const>=1']), ['1:type']);
  assert.deepEqual(codes(['x<const>=1']), ['1:type']);
  assert.deepEqual(codes(['x<int str>=1']), ['1:type']);
  assert.deepEqual(codes(['x<list list>']), ['1:type']);
  assert.deepEqual(codes(['x<int (1 2 x)>']), ['1:type']);
  assert.deepEqual(codes(['x<dict int str>=(a=1)', 'y<list int>', 'z<const upper>=HI']), []);
});

test('literal assignments are type checked like the compiler does', () => {
  const [d] = lint(['my_num<int>="asd"']);
  assert.equal(d.code, 'type-mismatch');
  assert.equal(d.message, 'my_num: expected int, received "asd"');
  assert.deepEqual(codes(['n<int>=5', 'n=abc']), ['2:type-mismatch']);
  assert.deepEqual(codes(['c<str (a b c)>=d']), ['1:type-mismatch']);
  assert.deepEqual(codes(['f<float>=$1', 'b<bool>=true', 'n<int>=$a + 1']), []);
});

test('collection values must match collection types', () => {
  assert.deepEqual(codes(['n<int>=(1 2)']), ['1:type-mismatch']);
  assert.deepEqual(codes(['l<list int>=(1 two 3)']), ['1:type-mismatch']);
  assert.deepEqual(codes(['d<dict int str>=(1 2)']), ['1:type-mismatch']);
});

test('const variables cannot be reassigned', () => {
  const [d] = lint(['PI<const float>=3.14', 'PI=3']);
  assert.equal(d.code, 'const-reassign');
  assert.match(d.message, /line 1/);
});

test('spaces around = are flagged', () => {
  assert.deepEqual(codes(['x = 5']), ['1:spaced-assignment']);
  assert.deepEqual(codes(['x= 5']), ['1:spaced-assignment']);
  assert.deepEqual(codes(['if $x == 5', '  echo a']), []);
});

test('function parameter syntax errors', () => {
  assert.deepEqual(codes(['f(a [b)', '  echo']), ['1:param']);
  assert.deepEqual(codes(['f(a ])', '  echo']), ['1:param']);
  assert.deepEqual(codes(['f(a | )', '  echo']), ['1:param']);
  assert.deepEqual(codes(['f(9a)', '  echo']), ['1:param']);
  assert.deepEqual(codes(['f(a', '  echo']), ['1:unclosed-paren']);
  assert.deepEqual(codes(['f(a) {', '  echo']), ['1:function-tail']);
  assert.deepEqual(codes(['f(--n<int="x">)', '  echo']), ['1:param-default']);
  assert.deepEqual(codes(['f(a a)', '  echo']), ['1:duplicate-param']);
  assert.deepEqual(codes(['f(rest... last)', '  echo']), ['1:param-order']);
  assert.deepEqual(codes(['f([])', '  echo']), ['1:param']);
});

test('multi-line parameters with comments parse like the compiler', () => {
  const source = [
    'f(',
    '  path # the path',
    '  [-s|--style <str>...]...',
    '  [--name<str="mike">]',
    '  [--mode<str (a b)>]',
    ')',
    '  echo $path $--style $--name',
  ];
  assert.deepEqual(codes(source), []);
  const fn = analyze(source.join('\n')).functions[0];
  assert.deepEqual(
    fn.leaves.map((l) => [l.kind, l.name, l.flag, l.datatype.description, l.effectiveOptional, l.repeat, l.array, l.default]),
    [
      ['arg', 'path', null, 'str', false, false, false, null],
      ['opt', 's', '-s', 'bool', true, false, false, null],
      ['opt', 'style', '--style', 'str', true, true, true, null],
      ['opt', 'name', '--name', 'str', true, false, false, '"mike"'],
      ['opt', 'mode', '--mode', 'str (a b)', true, false, false, null],
    ],
  );
});

test('option references must match a parameter', () => {
  assert.deepEqual(codes(['f(--a)', '  echo $--b']), ['2:unknown-option-ref']);
  assert.deepEqual(codes(['echo $--a']), ['1:unknown-option-ref']);
  assert.deepEqual(codes(['f(--some-flag)', '  echo $--some-flag', '  g(x)', '    echo $--some-flag']), []);
});

test('calls to user functions are checked for unknown options', () => {
  const fn = ['f(--a<int> [--flag])', '  echo $a'];
  assert.deepEqual(codes([...fn, 'f --a 1 --flag']), []);
  assert.deepEqual(codes([...fn, 'f --b 1']), ['3:unknown-option']);
  assert.deepEqual(codes([...fn, 'x=$(f --a 1 --nope)']), ['3:unknown-option']);
  assert.deepEqual(codes([...fn, 'f --a=1']), ['3:unknown-option']);
  assert.deepEqual(codes([...fn, 'echo "f --nope" \'f --nope\'']), []);
  assert.deepEqual(codes([...fn, 'f -- --nope']), []);
  assert.deepEqual(codes(['g()', '  echo', 'g --anything']), []);
});

test('commands nested in blocks are ignored by the compiler', () => {
  assert.deepEqual(codes(['if $x', '  .sub(a)', '    echo $a']), ['2:nested-command']);
  assert.deepEqual(codes(['.cmd(a)', '  .sub(b)', '    echo $b']), []);
});

test('duplicate functions are flagged', () => {
  assert.deepEqual(codes(['f(a) -> $a', 'f(a) -> $a']), ['2:duplicate-function']);
});

test('unterminated quotes are flagged', () => {
  assert.deepEqual(codes(['echo "hello']), ['1:unterminated-quote']);
  assert.deepEqual(codes(["echo don't"]), ['1:unterminated-quote']);
  assert.deepEqual(codes(["echo don't # it's balanced"]), []);
  assert.deepEqual(codes(['echo "a $(echo "b")"', 'echo hi # don\'t']), []);
  assert.deepEqual(codes(['echo "multi', '  line"']), []);
});

test('symbols outline functions, commands, params, and variables', () => {
  const source = fs.readFileSync(path.join(REPO, 'tests', 'vision.local.scrippo'), 'utf8');
  const { symbols } = analyze(source);
  const cmd = symbols.find((s) => s.name === '.my_cmd');
  assert.equal(cmd.kind, 'command');
  assert.deepEqual(cmd.children.map((c) => c.name), ['arg1', '--opt1', '.my_subcmd']);
  const pi = symbols.find((s) => s.name === 'PI');
  assert.equal(pi.kind, 'constant');
  assert.equal(pi.detail, '<const float>');
});

test('functionChainAt finds enclosing functions', () => {
  const analysis = analyze(['.a(x)', '  .b(y)', '    echo $y', '  echo $x', 'echo top'].join('\n'));
  assert.deepEqual(analysis.functionChainAt(2).map((f) => f.name), ['b', 'a']);
  assert.deepEqual(analysis.functionChainAt(3).map((f) => f.name), ['a']);
  assert.deepEqual(analysis.functionChainAt(4), []);
});
