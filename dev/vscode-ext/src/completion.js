'use strict';

const fs = require('fs');
const path = require('path');
const { describeParam, paramNames, scanShell, TYPE_NAMES } = require('./analyzer');
const docs = require('./docs');

const KEYWORD_SNIPPETS = {
  if: 'if ${1:condition}\n\t$0',
  elif: 'elif ${1:condition}\n\t$0',
  else: 'else\n\t$0',
  for: 'for ${1:item} in ${2:items}\n\t$0',
  while: 'while ${1:condition}\n\t$0',
  case: 'case ${1:value}\n\t${2:pattern})\n\t\t$0',
};

const RE = {
  macro: /(?:^|[^\w$])@([A-Za-z_]\w*)?$/,
  optionRef: /\$(--?[A-Za-z0-9_-]*)$/,
  variable: /\$\{?([A-Za-z_]\w*)?$/,
  declarationType: /^\s*[A-Za-z_]\w*<([^<>]*)$/,
  paramType: /(?:^|[\s(\[|])(?:\*|-{0,2})[A-Za-z_][\w-]*<([^<>]*)$/,
  statementLead: /^(?:(?:if|elif|while)\s+)?(?:!\s+)?/,
  separator: /[;&|(`]/g,
  name: /^[A-Za-z_][\w-]*$/,
};

const SKIP_DIRS = new Set(['.git', 'node_modules', 'target', 'dist', '.scrippo']);

function codeblock(code) {
  return `\`\`\`scriptopotamus\n${code}\n\`\`\``;
}

function result(prefix, typed, items) {
  return { from: prefix.length - (typed || '').length, items };
}

function macroItems(analysis, withAt) {
  const items = Object.entries(docs.BUILTINS)
    .filter(([, builtin]) => builtin.macro)
    .map(([name, builtin]) => macroItem(name, builtin, withAt));
  for (const macro of analysis.macros || []) {
    if (items.some((item) => item.filterText === macro.name)) continue;
    items.push(
      macroItem(
        macro.name,
        {
          signature: `@${macro.name}()`,
          description: `Macro defined on line ${macro.line + 1}`,
          snippet: macro.name,
        },
        withAt,
      ),
    );
  }
  return items;
}

function macroItem(name, builtin, withAt) {
  return {
    label: withAt ? `@${name}` : name,
    kind: 'macro',
    detail: `@${name}`,
    documentation: `${codeblock(builtin.signature)}\n\n${builtin.description}`,
    insertText: withAt ? `@${builtin.snippet}` : builtin.snippet,
    snippet: true,
    filterText: name,
  };
}

function importedModules(analysis) {
  const modules = new Set();
  for (const raw of analysis.imports || []) {
    for (const name of docs.modulesForImport(raw)) modules.add(name);
  }
  return modules;
}

function callableItem(fn) {
  return {
    label: fn.name,
    kind: fn.kind,
    detail: fn.leaves.map(describeParam).join(' '),
    documentation: `${codeblock(fn.header)}\n\n${fn.kind === 'command' ? 'Command' : 'Function'} defined on line ${fn.line + 1}`,
  };
}

function functionItems(analysis) {
  const imported = importedModules(analysis);
  const items = analysis.callable.map(callableItem);
  for (const [name, builtin] of Object.entries(docs.BUILTINS)) {
    if (builtin.macro || !imported.has(builtin.pkg) || analysis.callable.some((fn) => fn.name === name)) continue;
    items.push({ label: name, kind: 'function', detail: `scrippo/std/${builtin.pkg}`, documentation: `${codeblock(builtin.signature)}\n\n${builtin.description}` });
  }
  return items;
}

function keywordItems() {
  return Object.entries(KEYWORD_SNIPPETS).map(([name, snippet]) => ({
    label: name,
    kind: 'keyword',
    documentation: docs.KEYWORDS[name],
    insertText: snippet,
    snippet: true,
  }));
}

function variableItems(analysis, line) {
  const items = new Map();
  for (const fn of analysis.functionChainAt(line)) {
    for (const leaf of fn.leaves) {
      const role = leaf.kind === 'opt' ? 'option' : 'argument';
      for (const name of paramNames(leaf)) {
        if (items.has(name)) continue;
        items.set(name, {
          label: name,
          kind: 'param',
          detail: describeParam(leaf),
          documentation: `${leaf.effectiveOptional ? 'Optional' : 'Required'} ${role} of \`${fn.name || '.()'}\``,
          sortText: `0${name}`,
        });
      }
    }
  }
  for (const [name, variable] of analysis.variables) {
    if (items.has(name)) continue;
    const isConst = Boolean(variable.datatype && variable.datatype.isConst);
    items.set(name, {
      label: name,
      kind: isConst ? 'constant' : 'variable',
      detail: variable.datatype ? `<${variable.datatype.description}>` : '',
      documentation: `${isConst ? 'Constant' : 'Variable'} declared on line ${variable.line + 1}`,
      sortText: `1${name}`,
    });
  }
  return [...items.values()];
}

function optionRefItems(analysis, line) {
  const items = new Map();
  for (const fn of analysis.functionChainAt(line)) {
    for (const leaf of fn.leaves) {
      if (leaf.kind !== 'opt') continue;
      for (const flag of leaf.flags.length ? leaf.flags : [leaf.flag]) {
        if (items.has(flag)) continue;
        items.set(flag, {
          label: flag,
          kind: 'flag',
          detail: describeParam(leaf),
          documentation: `Option of \`${fn.name || '.()'}\`, also available as \`$${flag.replace(/^-+/, '').replace(/-/g, '_')}\``,
        });
      }
    }
  }
  return [...items.values()];
}

function typeItems(before) {
  const words = before.filter((w) => w !== 'const');
  if (words.length) return [];
  const names = [...(before.includes('const') ? [] : ['const']), ...TYPE_NAMES];
  return names.map((name) => ({ label: name, kind: 'type', documentation: docs.TYPES[name] }));
}

function typeCompletion(analysis, prefix, line) {
  const inHeader = analysis.functions.some((fn) => fn.line <= line && line <= fn.closeLine);
  const m = RE.declarationType.exec(prefix) || (inHeader && RE.paramType.exec(prefix));
  if (!m || /[={(]/.test(m[1])) return null;
  const words = m[1].split(/\s+/);
  const current = words.pop();
  return result(prefix, current, typeItems(words.filter(Boolean)));
}

function flagItemsFor(fn, used) {
  if (!fn) return [];
  return fn.leaves
    .filter((leaf) => leaf.kind === 'opt')
    .flatMap((leaf) => {
      const flags = leaf.flags.length ? leaf.flags : [leaf.flag];
      if (!leaf.repeat && flags.some((flag) => used.includes(flag))) return [];
      return flags
        .filter((flag) => leaf.repeat || !used.includes(flag))
        .map((flag) => ({
          label: flag,
          kind: 'flag',
          detail: describeParam(leaf),
          documentation: `${leaf.effectiveOptional ? 'Optional' : 'Required'} option of \`${fn.name}\``,
        }));
    });
}

function flagItems(analysis, command, used) {
  const resolved = analysis.resolveCommandPath([command]);
  if (resolved) return flagItemsFor(resolved.node, used);
  const builtin = docs.BUILTINS[command];
  if (!builtin || builtin.macro || !importedModules(analysis).has(builtin.pkg)) return [];
  return builtin.flags.filter((flag) => !used.includes(flag)).map((flag) => ({ label: flag, kind: 'flag', detail: command }));
}

function walkScrippoFiles(root) {
  const out = [];
  const visit = (dir, rel) => {
    let entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      if (entry.name.startsWith('.') && entry.name !== '.') continue;
      if (entry.isDirectory()) {
        if (SKIP_DIRS.has(entry.name)) continue;
        visit(path.join(dir, entry.name), rel ? `${rel}/${entry.name}` : entry.name);
      } else if (entry.isFile() && entry.name.endsWith('.scrippo')) {
        const fileRel = rel ? `${rel}/${entry.name}` : entry.name;
        out.push(`./${fileRel}`);
      }
    }
  };
  visit(root, '');
  return out.sort();
}

function matchesImportTyped(candidate, typed) {
  if (!typed) return true;
  if (candidate.startsWith(typed)) return true;
  if (typed.startsWith('./')) return candidate.startsWith(typed);
  return candidate.slice(2).startsWith(typed) || candidate.includes(`/${typed}`);
}

function localImportItems(importDir, typed, quote) {
  if (!importDir) return [];
  return walkScrippoFiles(importDir)
    .filter((file) => matchesImportTyped(file, typed))
    .map((file, index) => ({
      label: file,
      kind: 'module',
      detail: 'local file',
      documentation: `Import \`${file}\` relative to this script.`,
      insertText: quote ? `${file}${quote}` : file,
      sortText: `1${String(index).padStart(4, '0')}`,
    }));
}

function importCompletion(prefix, { importDir } = {}) {
  const m = /@import\s+(['"]?)([^\s'"]*)$/.exec(prefix);
  if (!m) return null;
  const quote = m[1];
  const typed = m[2];
  const items = [];
  const showPackages = !typed || typed.startsWith('scrippo') || 'scrippo'.startsWith(typed);
  if (showPackages) {
    for (const [index, pkg] of docs.PACKAGES.entries()) {
      items.push({
        label: pkg.path,
        kind: 'module',
        detail: pkg.detail,
        documentation: pkg.description,
        insertText: quote ? `${pkg.path}${quote}` : pkg.path,
        sortText: `0${String(index).padStart(2, '0')}`,
      });
    }
  }
  items.push(...localImportItems(importDir, typed, quote));
  if (!items.length) return null;
  return result(prefix, typed, items);
}

function commandCompletion(analysis, prefix, mask) {
  const cut = Math.max(-1, ...[...mask.matchAll(RE.separator)].map((m) => m.index));
  const segment = prefix.slice(cut + 1).trimStart();
  const lead = RE.statementLead.exec(segment)[0];
  const words = segment.slice(lead.length).split(/\s+/);
  const current = words.pop();
  if (!words.length) {
    if (current && !RE.name.test(current)) return null;
    const items = [...functionItems(analysis), ...macroItems(analysis, true)];
    if (cut < 0 && !lead) items.push(...keywordItems(), ...variableItems(analysis, -1));
    return result(prefix, current, items);
  }

  const pathInfo = analysis.resolveCommandPath(words);
  if (pathInfo) {
    const used = words.slice(pathInfo.depth);
    if (current.startsWith('-')) {
      return result(prefix, current, flagItemsFor(pathInfo.node, used));
    }
    if (pathInfo.depth === words.length) {
      const children = (pathInfo.node.children || []).filter((child) => child.kind === 'command' && child.name);
      if (children.length) return result(prefix, current, children.map(callableItem));
    }
    return null;
  }

  if (!current.startsWith('-')) return null;
  return result(prefix, current, flagItems(analysis, words[0], words.slice(1)));
}

function complete(analysis, lineText, line, character, options = {}) {
  const prefix = lineText.slice(0, character);
  const scanned = scanShell(prefix);
  if (scanned.comments.length || (scanned.unterminated && scanned.unterminated.kind === 'sq')) return null;
  let m;
  if ((m = RE.macro.exec(prefix))) return result(prefix, m[1], macroItems(analysis, false));
  if ((m = RE.optionRef.exec(prefix))) return result(prefix, m[1], optionRefItems(analysis, line));
  if ((m = RE.variable.exec(prefix))) return result(prefix, m[1], variableItems(analysis, line));
  return importCompletion(prefix, options) || typeCompletion(analysis, prefix, line) || commandCompletion(analysis, prefix, scanned.mask);
}

module.exports = { complete };
