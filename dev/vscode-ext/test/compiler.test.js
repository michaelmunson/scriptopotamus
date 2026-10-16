'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { analyze } = require('../src/analyzer');
const compiler = require('../src/compiler');

const REPO = path.join(__dirname, '..', '..', '..');
const COMPILER = compiler.findCompiler(REPO, REPO);

const skip = COMPILER ? false : 'needs a built scrippo binary (cargo build)';

async function run(lines, extra = {}) {
  return compiler.compile({ compilerPath: COMPILER, source: lines.join('\n'), ...extra }).promise;
}

test('parseErrors reads compiler diagnostics', () => {
  assert.deepEqual(compiler.parseErrors('Error: /tmp/input.scrippo:3: oops\nnoise\n/tmp/input.scrippo:12: bad\n'), [
    { stage: 'compile', line: 2, message: 'oops' },
    { stage: 'compile', line: 11, message: 'bad' },
  ]);
});

test('findCompiler walks up to the workspace root', () => {
  assert.equal(compiler.findCompiler(path.join(REPO, 'docs', 'examples'), REPO), COMPILER);
  assert.equal(compiler.findCompiler(path.join(REPO, 'docs'), path.join(REPO, 'docs')), null);
});

test('compiles the simple example', { skip }, async () => {
  const result = await run(fs.readFileSync(path.join(REPO, 'docs/examples/simple.scrippo'), 'utf8').split('\n'));
  assert.equal(result.ok, true, result.stderr);
  assert.match(result.stdout, /declare -r PI=3\.14/);
  assert.match(result.stdout, /function circumference/);
});

test('@import resolves stdlib and reports duplicate commands', { skip }, async () => {
  const ok = await run(['@import scrippo/std', 'prt -s green "hi"']);
  assert.equal(ok.ok, true, ok.stderr);
  assert.match(ok.stdout, /function prt/);
  const importDir = fs.mkdtempSync(path.join(os.tmpdir(), 'scrippo-dup-'));
  const main = path.join(importDir, 'main.scrippo');
  fs.writeFileSync(path.join(importDir, 'deploy.scrippo'), '.deploy(env)\n  -> "$env"\n');
  const dup = await run(['@import ./deploy.scrippo', '.deploy(env)', '  -> "$env"'], { sourceFile: main });
  fs.rmSync(importDir, { recursive: true, force: true });
  assert.equal(dup.ok, false, dup.stdout);
  assert.match(dup.errors.map((e) => e.message).join('\n'), /command 'deploy' already defined/);
});

test('@import globs expand from the source directory', { skip }, async () => {
  const importDir = fs.mkdtempSync(path.join(os.tmpdir(), 'scrippo-glob-'));
  const main = path.join(importDir, 'main.scrippo');
  try {
    fs.writeFileSync(path.join(importDir, 'a.bash'), 'echo from-a\n');
    fs.writeFileSync(path.join(importDir, 'c.scrippo'), '-> "from-c"\n');
    const all = await run(['@import ./*'], { sourceFile: main });
    assert.equal(all.ok, true, all.stderr);
    assert.match(all.stdout, /echo from-a/);
    assert.match(all.stdout, /echo "from-c"/);
    const missing = await run(['@import ./*.nope'], { sourceFile: main });
    assert.match(missing.errors.map((e) => e.message).join('\n'), /no matches/);
    const dynamic = await run(['@import $path'], { sourceFile: main });
    assert.match(dynamic.errors.map((e) => e.message).join('\n'), /must be a literal/);
  } finally {
    fs.rmSync(importDir, { recursive: true, force: true });
  }
});

test('unknown macros are compile errors', { skip }, async () => {
  const result = await run(['f()', '  @nope']);
  assert.equal(result.ok, false);
  assert.match(result.errors.map((e) => e.message).join('\n'), /unknown macro '@nope'/);
});

const agreeing = {
  'block closer': ['if $x', '  echo a', 'fi'],
  'esac': ['case $x', '  a) echo a', 'esac'],
  'case arm without pattern': ['case $x', '  echo a'],
  'orphan else': ['else', '  echo b'],
  'const reassignment': ['PI<const float>=3.14', 'PI=3'],
  'missing bracket': ['f(a [b)', '  echo'],
  'unknown type': ['n<upper>=1'],
};

for (const [name, lines] of Object.entries(agreeing)) {
  test(`compiler and linter agree: ${name}`, { skip }, async () => {
    const result = await run(lines);
    assert.equal(result.ok, false, result.stderr);
    assert.ok(result.errors.length >= 1, result.stderr);
    const lintLines = analyze(lines.join('\n')).diagnostics.map((d) => d.range.start.line);
    assert.ok(lintLines.length > 0, 'linter should report something');
    assert.ok(
      lintLines.some((line) => Math.abs(line - result.errors[0].line) <= 1),
      `compiler line ${result.errors[0].line + 1} (${result.errors[0].message}) vs linter lines ${lintLines.map((l) => l + 1)}`,
    );
  });
}
