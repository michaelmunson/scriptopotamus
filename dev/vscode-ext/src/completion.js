'use strict';

const { describeParam, scanShell, SCALARS, COLLECTIONS } = require('./analyzer');
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

function codeblock(code) {
  return `\`\`\`scriptopotamus\n${code}\n\`\`\``;
}

function result(prefix, typed, items) {
  return { from: prefix.length - (typed || '').length, items };
}

function macroItems(withAt) {
  return Object.entries(docs.BUILTINS)
    .filter(([, builtin]) => builtin.macro)
    .map(([name, builtin]) => ({
      label: withAt ? `@${name}` : name,
      kind: 'macro',
      detail: `@${name}`,
      documentation: `${codeblock(builtin.signature)}\n\n${builtin.description}`,
      insertText: withAt ? `@${builtin.snippet}` : builtin.snippet,
      snippet: true,
      filterText: name,
    }));
}

function functionItems(analysis) {
  const items = analysis.callable.map((fn) => ({
    label: fn.name,
    kind: fn.kind,
    detail: fn.leaves.map(describeParam).join(' '),
    documentation: `${codeblock(fn.header)}\n\n${fn.kind === 'command' ? 'Command' : 'Function'} defined on line ${fn.line + 1}`,
  }));
  for (const [name, builtin] of Object.entries(docs.BUILTINS)) {
    if (builtin.macro || analysis.callable.some((fn) => fn.name === name)) continue;
    items.push({ label: name, kind: 'function', detail: 'builtin', documentation: `${codeblock(builtin.signature)}\n\n${builtin.description}` });
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
      if (items.has(leaf.name)) continue;
      const role = leaf.kind === 'opt' ? 'option' : 'argument';
      items.set(leaf.name, {
        label: leaf.name,
        kind: 'param',
        detail: describeParam(leaf),
        documentation: `${leaf.effectiveOptional ? 'Optional' : 'Required'} ${role} of \`${fn.name || '.()'}\``,
        sortText: `0${leaf.name}`,
      });
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
      if (leaf.kind !== 'opt' || items.has(leaf.flag)) continue;
      items.set(leaf.flag, {
        label: leaf.flag,
        kind: 'flag',
        detail: describeParam(leaf),
        documentation: `Option of \`${fn.name || '.()'}\`, also available as \`$${leaf.name}\``,
      });
    }
  }
  return [...items.values()];
}

function typeItems(before) {
  const words = before.filter((w) => w !== 'const');
  const [base, ...args] = words;
  let names = [];
  if (!base) names = [...(before.includes('const') ? [] : ['const']), ...SCALARS, ...COLLECTIONS];
  else if ((base === 'list' && !args.length) || (base === 'dict' && args.length < 2)) names = SCALARS;
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

function flagItems(analysis, command, used) {
  if (command.startsWith('@')) {
    const builtin = docs.BUILTINS[command.slice(1)];
    if (!builtin || !builtin.macro) return [];
    return builtin.flags.map((flag) => ({ label: flag, kind: 'flag', detail: command }));
  }
  const fn = analysis.callable.find((f) => f.name === command);
  if (!fn) return [];
  return fn.leaves
    .filter((leaf) => leaf.kind === 'opt' && (leaf.repeat || !used.includes(leaf.flag)))
    .map((leaf) => ({
      label: leaf.flag,
      kind: 'flag',
      detail: describeParam(leaf),
      documentation: `${leaf.effectiveOptional ? 'Optional' : 'Required'} option of \`${fn.name}\``,
    }));
}

function commandCompletion(analysis, prefix, mask) {
  const cut = Math.max(-1, ...[...mask.matchAll(RE.separator)].map((m) => m.index));
  const segment = prefix.slice(cut + 1).trimStart();
  const lead = RE.statementLead.exec(segment)[0];
  const words = segment.slice(lead.length).split(/\s+/);
  const current = words.pop();
  if (!words.length) {
    if (current && !RE.name.test(current)) return null;
    const items = [...functionItems(analysis), ...macroItems(true)];
    if (cut < 0 && !lead) items.push(...keywordItems(), ...variableItems(analysis, -1));
    return result(prefix, current, items);
  }
  if (!current.startsWith('-')) return null;
  return result(prefix, current, flagItems(analysis, words[0], words.slice(1)));
}

function complete(analysis, lineText, line, character) {
  const prefix = lineText.slice(0, character);
  const scanned = scanShell(prefix);
  if (scanned.comments.length || (scanned.unterminated && scanned.unterminated.kind === 'sq')) return null;
  let m;
  if ((m = RE.macro.exec(prefix))) return result(prefix, m[1], macroItems(false));
  if ((m = RE.optionRef.exec(prefix))) return result(prefix, m[1], optionRefItems(analysis, line));
  if ((m = RE.variable.exec(prefix))) return result(prefix, m[1], variableItems(analysis, line));
  return typeCompletion(analysis, prefix, line) || commandCompletion(analysis, prefix, scanned.mask);
}

module.exports = { complete };
