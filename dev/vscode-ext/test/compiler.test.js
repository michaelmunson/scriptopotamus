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

test('@import globs expand to every matching file', { skip }, async () => {
  const importDir = fs.mkdtempSync(path.join(os.tmpdir(), 'scrippo-glob-'));
  const compileIn = (source, extra = {}) =>
    compiler.compile({ bash: 'bash', compilerPath: COMPILER, source, importDir, ...extra }).promise;
  try {
    fs.mkdirSync(path.join(importDir, 'sub'));
    fs.writeFileSync(path.join(importDir, 'a.bash'), '#!/usr/bin/env bash\necho from-a\n');
    fs.writeFileSync(path.join(importDir, 'c.scrippo'), '-> "from-c"\n');
    fs.writeFileSync(path.join(importDir, 'my file.bash'), 'echo from-space\n');
    fs.writeFileSync(path.join(importDir, 'sub', 'nested.bash'), 'echo nested\n');

    const all = await compileIn('@import ./* # imports all files');
    assert.equal(all.ok, true, all.stderr);
    assert.equal(all.stdout.split('#!/usr/bin/env bash').length - 1, 1);
    const fromA = all.stdout.indexOf('echo from-a');
    const fromC = all.stdout.indexOf('echo "from-c"');
    assert.ok(fromA >= 0 && fromC > fromA);
    assert.match(all.stdout, /echo from-space/);
    assert.doesNotMatch(all.stdout, /nested/);
    assert.deepEqual(analyze('@import ./* # imports all files').diagnostics, []);

    const scrippoOnly = await compileIn('@import "*.scrippo"');
    assert.equal(scrippoOnly.ok, true, scrippoOnly.stderr);
    assert.match(scrippoOnly.stdout, /echo "from-c"/);
    assert.doesNotMatch(scrippoOnly.stdout, /from-a/);

    const nested = await compileIn('@import ./sub/*.bash');
    assert.equal(nested.ok, true, nested.stderr);
    assert.match(nested.stdout, /echo nested/);

    const deep = await compileIn('@import "./**/*.bash"');
    assert.equal(deep.ok, true, deep.stderr);
    assert.match(deep.stdout, /echo from-a/);
    assert.match(deep.stdout, /echo nested/);
    assert.match(deep.stdout, /echo from-space/);

    const one = await compileIn('@import ./a.bash');
    assert.equal(one.ok, true, one.stderr);
    assert.match(one.stdout, /echo from-a/);
    assert.doesNotMatch(one.stdout, /from-c/);

    const twice = await compileIn('@import ./*.scrippo\n@import "c.scrippo"');
    assert.equal(twice.ok, true, twice.stderr);
    assert.equal(twice.stdout.split('echo "from-c"').length - 1, 1);

    const main = path.join(importDir, 'main.scrippo');
    fs.writeFileSync(main, '-> "before"\n@import ./*\n-> "after"\n');
    const self = cp.spawnSync('bash', [COMPILER, main], { encoding: 'utf8' });
    assert.equal(self.status, 0, self.stderr);
    assert.equal(self.stdout.split('echo "before"').length - 1, 1);
    assert.equal(self.stdout.split('echo "after"').length - 1, 1);
    assert.equal(self.stdout.split('echo from-a').length - 1, 1);

    const buffer = await compileIn('@import ./*\n-> "from-buffer"\n', { sourceFile: main });
    assert.equal(buffer.ok, true, buffer.stderr);
    assert.equal(buffer.stdout.split('echo "from-buffer"').length - 1, 1);
    assert.doesNotMatch(buffer.stdout, /echo "before"/);
    assert.match(buffer.stdout, /echo from-a/);

    const missing = await compileIn('@import ./*.nope');
    assert.deepEqual(missing.errors.map((e) => e.message), [`@import: no matches: ${importDir}/./*.nope`]);

    const dynamic = await compileIn('@import $path');
    assert.deepEqual(dynamic.errors.map((e) => e.message), ['@import: <path> must be a literal, received $path']);
  } finally {
    fs.rmSync(importDir, { recursive: true, force: true });
  }
});

test('trailing comments on macros are not arguments', { skip }, async () => {
  const dropped = await run(['@throw 1 "bad" # note']);
  assert.equal(dropped.ok, true, dropped.stderr);
  assert.match(dropped.stdout, /bad/);
  assert.doesNotMatch(dropped.stdout, /note/);
  const kept = await run(['@throw 1 "bad # note"']);
  assert.equal(kept.ok, true, kept.stderr);
  assert.match(kept.stdout, /bad # note/);
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

test('case arms compile with optional terminators', { skip }, async () => {
  const result = await run(['case $str', '\thello) -> "Hi" ;;', '\tworld)', '\t\t-> "Bye"', '\t*) ;&']);
  assert.equal(result.ok, true, result.stderr);
  assert.equal(
    result.stdout,
    [
      '#!/usr/bin/env bash',
      'case $str in',
      '  hello)',
      '    echo "Hi"',
      '    ;;',
      '  world)',
      '    echo "Bye"',
      '    ;;',
      '  *)',
      '    :',
      '    ;&',
      'esac',
      '',
    ].join('\n'),
  );
});

const agreeing = {
  'block closer': ['if $x', '  echo a', 'fi'],
  'esac': ['case $x', '  a) echo a', 'esac'],
  'case arm without pattern': ['case $x', '  echo a'],
  'case terminator on its own': ['case $x', '  a)', '    echo a', '  ;;'],
  'orphan else': ['if $x', '  echo a', '  else', '  echo b'],
  'else with comment': ['if $x', '  echo a', 'else # c', '  echo b'],
  'type mismatch': ['n<int>="asd"'],
  'const reassignment': ['PI<const float>=3.14', 'PI=3'],
  'invalid parameter': ['f(9a)', '  echo'],
  'star param not last': ['f(*rest a)', '  echo'],
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
