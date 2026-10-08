'use strict';

const cp = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

const ERROR_LINE = /^(ast|compile): line (\d+): (.*)$/;
const COMPILER_RELATIVE_PATH = path.join('src', 'compiler.bash');

function parseErrors(stderr) {
  const errors = [];
  for (const line of stderr.split('\n')) {
    const m = ERROR_LINE.exec(line.trim());
    if (m) errors.push({ stage: m[1], line: Math.max(0, Number(m[2]) - 1), message: m[3] });
  }
  return errors;
}

function findCompiler(start, stop) {
  const root = path.resolve(stop);
  let dir = path.resolve(start);
  const relative = path.relative(root, dir);
  if (relative.startsWith('..') || path.isAbsolute(relative)) dir = root;
  for (;;) {
    const candidate = path.join(dir, COMPILER_RELATIVE_PATH);
    if (fs.existsSync(candidate)) return candidate;
    if (dir === root) return null;
    dir = path.dirname(dir);
  }
}

function bashMajorVersion(bash) {
  return new Promise((resolve) => {
    cp.execFile(bash, ['-c', 'echo "${BASH_VERSINFO[0]}"'], { timeout: 5000 }, (error, stdout) => {
      resolve(error ? null : Number(stdout.trim()) || null);
    });
  });
}

function compile({ bash, compilerPath, source, importDir, sourceFile, timeout = 20000 }) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'scrippo-'));
  const file = path.join(dir, 'input.scrippo');
  fs.writeFileSync(file, source);
  const env = { ...process.env };
  if (importDir) env.SCRIPPO_IMPORT_DIR = importDir;
  if (sourceFile) env.SCRIPPO_SOURCE = sourceFile;
  let child;
  let cancelled = false;
  const promise = new Promise((resolve) => {
    child = cp.execFile(
      bash,
      [compilerPath, file],
      { cwd: path.dirname(compilerPath), env, timeout, maxBuffer: 16 * 1024 * 1024 },
      (error, stdout, stderr) => {
        fs.rmSync(dir, { recursive: true, force: true });
        resolve({
          ok: !error,
          cancelled,
          timedOut: Boolean(error && error.killed && !cancelled),
          spawnError: error && typeof error.code === 'string' ? error.message : null,
          stdout,
          stderr,
          errors: parseErrors(stderr),
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
  const first = result.stderr.split('\n').find((line) => line.trim());
  return first ? first.trim() : 'the compiler exited with an error';
}

module.exports = { compile, parseErrors, findCompiler, bashMajorVersion, describeFailure, COMPILER_RELATIVE_PATH };
