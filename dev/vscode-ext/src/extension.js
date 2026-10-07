'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const vscode = require('vscode');
const { analyze, describeParam } = require('./analyzer');
const compiler = require('./compiler');
const docs = require('./docs');

const LANGUAGE_ID = 'scriptopotamus';
const LINT_SOURCE = 'scriptopotamus';
const COMPILER_SOURCE = 'scriptopotamus compiler';
const LINT_DELAY = 200;
const COMPILE_DELAY = 800;
const WORD = /->|\$?--?[A-Za-z_][A-Za-z0-9_-]*|\$?[A-Za-z_][A-Za-z0-9_]*/;

const SEVERITY = {
  error: vscode.DiagnosticSeverity.Error,
  warning: vscode.DiagnosticSeverity.Warning,
  info: vscode.DiagnosticSeverity.Information,
  hint: vscode.DiagnosticSeverity.Hint,
};

const SYMBOL_KIND = {
  function: vscode.SymbolKind.Function,
  command: vscode.SymbolKind.Module,
  variable: vscode.SymbolKind.Variable,
  constant: vscode.SymbolKind.Constant,
  arg: vscode.SymbolKind.Variable,
  opt: vscode.SymbolKind.Property,
};

function toRange(r) {
  return new vscode.Range(r.start.line, r.start.character, r.end.line, r.end.character);
}

function tokenRange(token) {
  return new vscode.Range(token.line, token.col, token.endLine, token.end);
}

function settings(doc) {
  return vscode.workspace.getConfiguration('scriptopotamus', doc ? doc.uri : undefined);
}

function isScrippo(doc) {
  return doc.languageId === LANGUAGE_ID;
}

function importDirOf(doc) {
  return doc.uri.scheme === 'file' ? path.dirname(doc.uri.fsPath) : undefined;
}

function activate(context) {
  const output = vscode.window.createOutputChannel('Scriptopotamus');
  const lintCollection = vscode.languages.createDiagnosticCollection(LINT_SOURCE);
  const compilerCollection = vscode.languages.createDiagnosticCollection(COMPILER_SOURCE);
  const analyses = new Map();
  const timers = new Map();
  const jobs = new Map();
  const bashVersions = new Map();
  const warned = new Set();

  function analysisFor(doc) {
    const key = doc.uri.toString();
    const cached = analyses.get(key);
    if (cached && cached.version === doc.version) return cached.result;
    const result = analyze(doc.getText());
    analyses.set(key, { version: doc.version, result });
    return result;
  }

  function lint(doc) {
    if (!settings(doc).get('lint.enable', true)) {
      lintCollection.delete(doc.uri);
      return;
    }
    lintCollection.set(
      doc.uri,
      analysisFor(doc).diagnostics.map((d) => {
        const diagnostic = new vscode.Diagnostic(toRange(d.range), d.message, SEVERITY[d.severity]);
        diagnostic.source = LINT_SOURCE;
        diagnostic.code = d.code;
        if (d.code === 'trailing-whitespace') diagnostic.tags = [vscode.DiagnosticTag.Unnecessary];
        return diagnostic;
      }),
    );
  }

  function notify(message, key, interactive) {
    output.appendLine(message);
    if (!interactive && warned.has(key)) return;
    warned.add(key);
    vscode.window.showWarningMessage(message, 'Open Settings').then((choice) => {
      if (choice) vscode.commands.executeCommand('workbench.action.openSettings', 'scriptopotamus.compiler');
    });
  }

  function resolveCompilerPath(doc) {
    const folder = vscode.workspace.getWorkspaceFolder(doc.uri);
    let configured = settings(doc).get('compiler.path', '').trim();
    if (configured) {
      configured = configured
        .replace(/\$\{workspaceFolder\}/g, folder ? folder.uri.fsPath : '')
        .replace(/^~(?=$|[\\/])/, os.homedir());
      if (!path.isAbsolute(configured) && folder) configured = path.join(folder.uri.fsPath, configured);
      return { path: configured, configured: true };
    }
    if (folder && doc.uri.scheme === 'file') {
      const found = compiler.findCompiler(path.dirname(doc.uri.fsPath), folder.uri.fsPath);
      if (found) return { path: found, configured: false };
    }
    for (const f of vscode.workspace.workspaceFolders || []) {
      const found = compiler.findCompiler(f.uri.fsPath, f.uri.fsPath);
      if (found) return { path: found, configured: false };
    }
    return null;
  }

  async function compilerSetup(doc, interactive) {
    if (!vscode.workspace.isTrusted) {
      if (interactive) notify('The Scriptopotamus compiler only runs in trusted workspaces.', 'trust', true);
      return null;
    }
    const resolved = resolveCompilerPath(doc);
    if (!resolved) {
      const message = `Could not find the Scriptopotamus compiler (${compiler.COMPILER_RELATIVE_PATH}) in this workspace. Set scriptopotamus.compiler.path to use it.`;
      if (interactive) notify(message, 'missing-compiler', true);
      else output.appendLine(message);
      return null;
    }
    if (!fs.existsSync(resolved.path)) {
      notify(`Scriptopotamus compiler not found at ${resolved.path}`, `missing:${resolved.path}`, interactive);
      return null;
    }
    const bash = settings(doc).get('compiler.bashPath', 'bash').trim() || 'bash';
    if (!bashVersions.has(bash)) bashVersions.set(bash, compiler.bashMajorVersion(bash));
    const major = await bashVersions.get(bash);
    if (!major || major < 4) {
      const message = major
        ? `The Scriptopotamus compiler needs bash 4 or newer, but '${bash}' is bash ${major}. Set scriptopotamus.compiler.bashPath to a newer bash.`
        : `Could not run '${bash}'. Set scriptopotamus.compiler.bashPath to a bash 4+ executable.`;
      notify(message, `bash:${bash}`, interactive);
      return null;
    }
    return { bash, compilerPath: resolved.path };
  }

  function publishCompilerResult(doc, result) {
    if (result.ok) {
      compilerCollection.delete(doc.uri);
      return;
    }
    if (!result.errors.length) {
      output.appendLine(`Compiler failed for ${doc.uri.fsPath || doc.uri.toString()}:\n${result.stderr || compiler.describeFailure(result)}`);
      const diagnostic = new vscode.Diagnostic(
        new vscode.Range(0, 0, 0, doc.lineAt(0).text.length),
        `Scriptopotamus compiler failed: ${compiler.describeFailure(result)}`,
        vscode.DiagnosticSeverity.Warning,
      );
      diagnostic.source = COMPILER_SOURCE;
      compilerCollection.set(doc.uri, [diagnostic]);
      return;
    }
    const lintErrorLines = new Set(
      settings(doc).get('lint.enable', true)
        ? analysisFor(doc).diagnostics.filter((d) => d.severity === 'error').map((d) => d.range.start.line)
        : [],
    );
    compilerCollection.set(
      doc.uri,
      result.errors
        .filter((e) => !lintErrorLines.has(e.line))
        .map((e) => {
          const line = doc.lineAt(Math.min(e.line, doc.lineCount - 1));
          const range = new vscode.Range(line.lineNumber, line.firstNonWhitespaceCharacterIndex, line.lineNumber, line.text.length);
          const diagnostic = new vscode.Diagnostic(range, e.message, vscode.DiagnosticSeverity.Error);
          diagnostic.source = COMPILER_SOURCE;
          diagnostic.code = e.stage;
          return diagnostic;
        }),
    );
  }

  function cancelJob(doc) {
    const key = doc.uri.toString();
    const job = jobs.get(key);
    if (job) job.cancel();
    jobs.delete(key);
  }

  async function runCompiler(doc) {
    cancelJob(doc);
    if (settings(doc).get('compiler.run', 'onSave') === 'off') {
      compilerCollection.delete(doc.uri);
      return;
    }
    const setup = await compilerSetup(doc, false);
    if (!setup || doc.isClosed) {
      compilerCollection.delete(doc.uri);
      return;
    }
    const key = doc.uri.toString();
    const version = doc.version;
    const job = compiler.compile({ ...setup, source: doc.getText(), importDir: importDirOf(doc) });
    jobs.set(key, job);
    const result = await job.promise;
    if (jobs.get(key) === job) jobs.delete(key);
    if (result.cancelled || doc.isClosed || doc.version !== version) return;
    publishCompilerResult(doc, result);
  }

  function schedule(doc, kind, delay, fn) {
    const key = `${kind}:${doc.uri.toString()}`;
    clearTimeout(timers.get(key));
    timers.set(
      key,
      setTimeout(() => {
        timers.delete(key);
        if (!doc.isClosed) fn(doc);
      }, delay),
    );
  }

  function refresh(doc) {
    if (!isScrippo(doc)) return;
    lint(doc);
    runCompiler(doc);
  }

  function forget(doc) {
    const key = doc.uri.toString();
    for (const kind of ['lint', 'compile']) {
      clearTimeout(timers.get(`${kind}:${key}`));
      timers.delete(`${kind}:${key}`);
    }
    cancelJob(doc);
    analyses.delete(key);
    lintCollection.delete(doc.uri);
    compilerCollection.delete(doc.uri);
  }

  function findParam(chain, word) {
    const bare = word.replace(/^\$/, '');
    for (const fn of chain) {
      for (const leaf of fn.leaves) {
        if (bare.startsWith('-')) {
          const asName = bare.replace(/^-+/, '').replace(/-/g, '_');
          if (leaf.flag === bare || (word.startsWith('$') && leaf.name === asName)) return { fn, leaf };
        } else if (leaf.name === bare) {
          return { fn, leaf };
        }
      }
    }
    return null;
  }

  function insideTypeAnnotation(doc, pos) {
    const before = doc.lineAt(pos.line).text.slice(0, pos.character);
    return before.lastIndexOf('<') > before.lastIndexOf('>');
  }

  function wordAt(doc, pos) {
    const range = doc.getWordRangeAtPosition(pos, WORD);
    return range ? { range, word: doc.getText(range) } : null;
  }

  function describe(doc, pos, { word, range }) {
    const analysis = analysisFor(doc);
    const md = new vscode.MarkdownString();
    if (word === '->') return md.appendMarkdown(docs.KEYWORDS['->']);
    if (docs.TYPES[word] && insideTypeAnnotation(doc, pos)) return md.appendMarkdown(`**${word}**: ${docs.TYPES[word]}`);

    const param = findParam(analysis.functionChainAt(pos.line), word);
    if (param) {
      const { fn, leaf } = param;
      md.appendCodeblock(describeParam(leaf), LANGUAGE_ID);
      const role = leaf.kind === 'opt' ? 'option' : 'argument';
      md.appendMarkdown(`${leaf.effectiveOptional ? 'Optional' : 'Required'} ${role} of \`${fn.name}\`, available as \`$${leaf.name}\``);
      if (leaf.default !== null) md.appendMarkdown(`, defaults to \`${leaf.default}\``);
      return md;
    }

    const bare = word.replace(/^\$/, '');
    if (!word.startsWith('$')) {
      const fn = analysis.findFunction(bare);
      if (fn) {
        md.appendCodeblock(fn.header, LANGUAGE_ID);
        return md.appendMarkdown(`${fn.kind === 'command' ? 'Command' : 'Function'} defined on line ${fn.line + 1}`);
      }
    }
    const variable = analysis.variables.get(bare);
    if (variable) {
      md.appendCodeblock(variable.datatype ? `${bare}<${variable.datatype.description}>` : bare, LANGUAGE_ID);
      return md.appendMarkdown(`${variable.datatype && variable.datatype.isConst ? 'Constant' : 'Variable'} declared on line ${variable.line + 1}`);
    }
    if (word.startsWith('$')) return null;
    if (docs.BUILTINS[word]) {
      md.appendCodeblock(docs.BUILTINS[word].signature, LANGUAGE_ID);
      return md.appendMarkdown(`Builtin: ${docs.BUILTINS[word].description}`);
    }
    if (docs.KEYWORDS[word] && doc.lineAt(pos.line).firstNonWhitespaceCharacterIndex === range.start.character) {
      return md.appendMarkdown(docs.KEYWORDS[word]);
    }
    return null;
  }

  const hoverProvider = {
    provideHover(doc, pos) {
      const found = wordAt(doc, pos);
      if (!found) return null;
      const md = describe(doc, pos, found);
      return md ? new vscode.Hover(md, found.range) : null;
    },
  };

  const definitionProvider = {
    provideDefinition(doc, pos) {
      const found = wordAt(doc, pos);
      if (!found) return null;
      const analysis = analysisFor(doc);
      const param = findParam(analysis.functionChainAt(pos.line), found.word);
      if (param) return new vscode.Location(doc.uri, tokenRange(param.leaf.token));
      const bare = found.word.replace(/^\$/, '');
      const fn = !found.word.startsWith('$') && analysis.findFunction(bare);
      if (fn) return new vscode.Location(doc.uri, toRange(fn.nameRange));
      const variable = analysis.variables.get(bare);
      if (variable) return new vscode.Location(doc.uri, toRange(variable.range));
      return null;
    },
  };

  const symbolProvider = {
    provideDocumentSymbols(doc) {
      const toSymbol = (s) => {
        const symbol = new vscode.DocumentSymbol(s.name, s.detail, SYMBOL_KIND[s.kind], toRange(s.range), toRange(s.selectionRange));
        symbol.children = s.children.map(toSymbol);
        return symbol;
      };
      return analysisFor(doc).symbols.map(toSymbol);
    },
  };

  function fix(title, diagnostic, build) {
    const action = new vscode.CodeAction(title, vscode.CodeActionKind.QuickFix);
    action.diagnostics = [diagnostic];
    action.edit = new vscode.WorkspaceEdit();
    action.isPreferred = true;
    build(action.edit);
    return action;
  }

  function moveCommentAbove(doc, diagnostic, uri, edit) {
    const line = doc.lineAt(diagnostic.range.start.line);
    const indent = line.text.slice(0, line.firstNonWhitespaceCharacterIndex);
    const comment = line.text.slice(diagnostic.range.start.character).trim();
    const code = line.text.slice(0, diagnostic.range.start.character).trimEnd();
    edit.replace(uri, line.range, `${indent}${comment}\n${code}`);
  }

  const codeActionProvider = {
    provideCodeActions(doc, _range, context) {
      const actions = [];
      for (const d of context.diagnostics) {
        if (d.source !== LINT_SOURCE) continue;
        const line = doc.lineAt(d.range.start.line);
        if (d.code === 'trailing-whitespace') {
          actions.push(fix('Remove trailing whitespace', d, (edit) => edit.delete(doc.uri, d.range)));
        } else if (d.code === 'block-closer') {
          actions.push(fix(`Remove '${line.text.trim().split(/[\s;]/)[0]}'`, d, (edit) => edit.delete(doc.uri, line.rangeIncludingLineBreak)));
        } else if (d.code === 'trailing-comment') {
          actions.push(fix('Move comment to its own line', d, (edit) => moveCommentAbove(doc, d, doc.uri, edit)));
        } else if (d.code === 'else-trailing' && /^\s*else\s+#/.test(line.text)) {
          const start = line.text.indexOf('#');
          const commentDiagnostic = { range: new vscode.Range(line.lineNumber, start, line.lineNumber, line.text.length) };
          actions.push(fix('Move comment to its own line', d, (edit) => moveCommentAbove(doc, commentDiagnostic, doc.uri, edit)));
        } else if (d.code === 'spaced-assignment') {
          const fixed = line.text.replace(/^(\s*[A-Za-z_][A-Za-z0-9_]*(?:<[^>]*>)?)\s*=\s*/, '$1=');
          actions.push(fix('Remove spaces around =', d, (edit) => edit.replace(doc.uri, line.range, fixed)));
        }
      }
      return actions;
    },
  };

  async function showCompiled() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || !isScrippo(editor.document)) {
      vscode.window.showInformationMessage('Open a Scriptopotamus file to compile it.');
      return;
    }
    const doc = editor.document;
    const setup = await compilerSetup(doc, true);
    if (!setup) return;
    const version = doc.version;
    const result = await vscode.window.withProgress(
      { location: vscode.ProgressLocation.Window, title: 'Compiling Scriptopotamus' },
      () => compiler.compile({ ...setup, source: doc.getText(), importDir: importDirOf(doc) }).promise,
    );
    if (!doc.isClosed && doc.version === version) publishCompilerResult(doc, result);
    if (!result.ok) {
      const first = result.errors[0];
      const message = first ? `line ${first.line + 1}: ${first.message}` : compiler.describeFailure(result);
      vscode.window.showErrorMessage(`Scriptopotamus compile failed, ${message}`);
      return;
    }
    const compiled = await vscode.workspace.openTextDocument({ language: 'shellscript', content: result.stdout });
    await vscode.window.showTextDocument(compiled, { viewColumn: vscode.ViewColumn.Beside, preview: true, preserveFocus: true });
  }

  const selector = { language: LANGUAGE_ID };
  context.subscriptions.push(
    output,
    lintCollection,
    compilerCollection,
    vscode.languages.registerHoverProvider(selector, hoverProvider),
    vscode.languages.registerDefinitionProvider(selector, definitionProvider),
    vscode.languages.registerDocumentSymbolProvider(selector, symbolProvider),
    vscode.languages.registerCodeActionsProvider(selector, codeActionProvider, {
      providedCodeActionKinds: [vscode.CodeActionKind.QuickFix],
    }),
    vscode.commands.registerCommand('scriptopotamus.showCompiled', showCompiled),
    vscode.workspace.onDidOpenTextDocument(refresh),
    vscode.workspace.onDidCloseTextDocument(forget),
    vscode.workspace.onDidChangeTextDocument((event) => {
      const doc = event.document;
      if (!isScrippo(doc) || !event.contentChanges.length) return;
      schedule(doc, 'lint', LINT_DELAY, lint);
      cancelJob(doc);
      if (settings(doc).get('compiler.run', 'onSave') === 'onType') schedule(doc, 'compile', COMPILE_DELAY, runCompiler);
      else compilerCollection.delete(doc.uri);
    }),
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (isScrippo(doc) && settings(doc).get('compiler.run', 'onSave') !== 'off') runCompiler(doc);
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (!event.affectsConfiguration('scriptopotamus')) return;
      bashVersions.clear();
      warned.clear();
      vscode.workspace.textDocuments.forEach(refresh);
    }),
    vscode.workspace.onDidGrantWorkspaceTrust(() => vscode.workspace.textDocuments.forEach(refresh)),
    {
      dispose() {
        timers.forEach(clearTimeout);
        jobs.forEach((job) => job.cancel());
      },
    },
  );

  vscode.workspace.textDocuments.forEach(refresh);
}

function deactivate() {}

module.exports = { activate, deactivate };
