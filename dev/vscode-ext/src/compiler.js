'use strict';

const cp = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

const ERROR_LINE = /^(?:Error:\s*)?(.*):(\d+): (.*)$/;
const COMPILER_CANDIDATES = [path.join('target', 'debug', 'scrippo'), path.join('target', 'release', 'scrippo')];

function parseErrors(stderr) {
  const errors = [];
  for (const line of stderr.split('\n')) {
    const m = ERROR_LINE.exec(line.trim());
    if (m && /^\d+$/.test(m[2])) errors.push({ stage: 'compile', line: Math.max(0, Number(m[2]) - 1), message: m[3] });
  }
  return errors;
}

function findCompiler(start, stop) {
  const root = path.resolve(stop);
  let dir = path.resolve(start);
  const relative = path.relative(root, dir);
  if (relative.startsWith('..') || path.isAbsolute(relative)) dir = root;
  for (;;) {
    for (const relativePath of COMPILER_CANDIDATES) {
      const candidate = path.join(dir, relativePath);
      if (fs.existsSync(candidate)) return candidate;
    }
    if (dir === root) return null;
    dir = path.dirname(dir);
  }
}

function compile({ compilerPath, source, sourceFile, timeout = 20000 }) {
  const besideSource = sourceFile ? path.dirname(sourceFile) : null;
  const dir = besideSource && fs.existsSync(besideSource) ? besideSource : fs.mkdtempSync(path.join(os.tmpdir(), 'scrippo-'));
  const removeDir = dir !== besideSource;
  const file = path.join(dir, `.scrippo-compile-${process.pid}-${Date.now()}.scrippo`);
  fs.writeFileSync(file, source);
  let child;
  let cancelled = false;
  const promise = new Promise((resolve) => {
    child = cp.execFile(
      compilerPath,
      ['-c', file],
      { cwd: path.dirname(compilerPath), timeout, maxBuffer: 16 * 1024 * 1024 },
      (error, stdout, stderr) => {
        fs.rmSync(file, { force: true });
        if (removeDir) fs.rmSync(dir, { recursive: true, force: true });
        resolve({
          ok: !error,
          cancelled,
          timedOut: Boolean(error && error.killed && !cancelled),
          spawnError: error && typeof error.code === 'string' ? error.message : null,
          stdout,
          stderr,
          errors: parseErrors(stderr || ''),
        });
      },
    );
  });
  const cancel = () => {
    cancelled = true;
    child.kill('SIGTERM');
  };
  return { promise, cancel };
}

function describeFailure(result) {
  if (result.spawnError) return result.spawnError;
  if (result.timedOut) return 'the compiler timed out';
  const first = (result.stderr || '').split('\n').find((line) => line.trim());
  return first ? first.trim() : 'the compiler exited with an error';
}

module.exports = { compile, parseErrors, findCompiler, describeFailure, COMPILER_CANDIDATES };
