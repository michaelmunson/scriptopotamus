'use strict';

const assert = require('node:assert/strict');
const cp = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { analyze } = require('../src/analyzer');
const compiler = require('../src/compiler');

const REPO = path.join(__dirname, '..', '..', '..');
const COMPILER = path.join(REPO, 'src', 'compiler.bash');

function hasTool(command) {
  return cp.spawnSync('bash', ['-c', command]).status === 0;
}

const skip =
  !fs.existsSync(COMPILER) || !hasTool('(( BASH_VERSINFO[0] >= 4 ))') || !hasTool('command -v jq')
    ? 'needs the compiler, bash 4+, and jq'
    : false;

async function run(lines) {
  return compiler.compile({ bash: 'bash', compilerPath: COMPILER, source: lines.join('\n') }).promise;
}

test('parseErrors reads ast and compile errors', () => {
  assert.deepEqual(compiler.parseErrors('ast: line 3: oops\nnoise\ncompile: line 12: bad\n'), [
    { stage: 'ast', line: 2, message: 'oops' },
    { stage: 'compile', line: 11, message: 'bad' },
  ]);
});

test('findCompiler walks up to the workspace root', () => {
  assert.equal(compiler.findCompiler(path.join(REPO, 'docs', 'examples'), REPO), COMPILER);
  assert.equal(compiler.findCompiler(path.join(REPO, 'docs'), path.join(REPO, 'docs')), null);
});

test('compiles the vision example', { skip }, async () => {
  const result = await run(fs.readFileSync(path.join(REPO, 'tests', 'vision.local.scrippo'), 'utf8').split('\n'));
  assert.equal(result.ok, true, result.stderr);
  assert.match(result.stdout, /^#!\/usr\/bin\/env bash/);
});

test('macros expand inline and @import resolves from importDir', { skip }, async () => {
  const result = await compiler.compile({
    bash: 'bash',
    compilerPath: COMPILER,
    source: ['@import "simple.scrippo"', '@throw 2 "bad"'].join('\n'),
    importDir: path.join(REPO, 'docs', 'examples'),
  }).promise;
  assert.equal(result.ok, true, result.stderr);
  assert.match(result.stdout, /function circumference/);
  assert.match(result.stdout, /^exit 2$/m);
});

test('commands from imports are dispatched and duplicates are compile errors', { skip }, async () => {
  const importDir = fs.mkdtempSync(path.join(os.tmpdir(), 'scrippo-test-'));
  fs.writeFileSync(path.join(importDir, 'deploy.scrippo'), '.deploy(env)\n  -> "$env"\n');
  const ok = await compiler.compile({ bash: 'bash', compilerPath: COMPILER, source: '@import "deploy.scrippo"', importDir }).promise;
  const dup = await compiler.compile({ bash: 'bash', compilerPath: COMPILER, source: '@import "deploy.scrippo"\n.deploy()\n  -> 1', importDir }).promise;
  fs.rmSync(importDir, { recursive: true, force: true });
  assert.equal(ok.ok, true, ok.stderr);
  assert.match(ok.stdout, /^ {2}deploy\)$/m);
  assert.deepEqual(dup.errors.map((e) => [e.line, e.message]), [[1, "command 'deploy' already defined"]]);
});

test('unknown macros are compile errors', { skip }, async () => {
  const result = await run(['f()', '  @nope 1']);
  assert.deepEqual(result.errors.map((e) => [e.line, e.message]), [[1, "unknown macro '@nope'"]]);
});

test('trailing whitespace breaks else in the compiler and is flagged at its cause', { skip }, async () => {
  const lines = ['if $x   ', '  echo a', 'else', '  echo b'];
  const result = await run(lines);
  assert.deepEqual(result.errors.map((e) => e.line), [2]);
  const [d] = analyze(lines.join('\n')).diagnostics;
  assert.equal(d.code, 'trailing-whitespace');
  assert.equal(d.range.start.line, 0);
});

const agreeing = {
  'block closer': ['if $x', '  echo a', 'fi'],
  'orphan else': ['if $x', '  echo a', '  else', '  echo b'],
  'else with comment': ['if $x', '  echo a', 'else # c', '  echo b'],
  'type mismatch': ['n<int>="asd"'],
  'const reassignment': ['PI<const float>=3.14', 'PI=3'],
  'invalid parameter': ['f(9a)', '  echo'],
  'missing bracket': ['f(a [b)', '  echo'],
  'brace after params': ['f(a) {', '  echo $a', '}'],
};

for (const [name, lines] of Object.entries(agreeing)) {
  test(`compiler and linter agree: ${name}`, { skip }, async () => {
    const result = await run(lines);
    assert.equal(result.ok, false, 'compiler should fail');
    assert.equal(result.errors.length, 1, result.stderr);
    const lintLines = analyze(lines.join('\n')).diagnostics.map((d) => d.range.start.line);
    assert.ok(lintLines.length > 0, 'linter should report something');
    assert.ok(
      lintLines.some((line) => Math.abs(line - result.errors[0].line) <= 1),
      `compiler line ${result.errors[0].line + 1} vs linter lines ${lintLines.map((l) => l + 1)}`,
    );
  });
}
