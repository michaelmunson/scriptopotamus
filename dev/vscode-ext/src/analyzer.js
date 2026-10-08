'use strict';

const SCALARS = ['int', 'float', 'str', 'upper', 'lower', 'bool'];
const COLLECTIONS = ['list', 'dict'];
const TYPE_WORDS = new Set([...SCALARS, ...COLLECTIONS, 'const']);
const PATTERNS = {
  int: /^-?[0-9]+$/,
  float: /^-?[0-9]*\.?[0-9]+$/,
  bool: /^(true|false)$/,
};
const PARAM_PUNCTUATION = new Set(['[', ']', '|']);
const NAME = '[A-Za-z_][A-Za-z0-9_]*';
const FUNCTION_NAME = '[A-Za-z_][A-Za-z0-9_-]*';

const RE = {
  if: /^if\s+(.+)$/d,
  elif: /^elif\s+(.+)$/d,
  orphan: /^(elif|else)(?=\s|$)/,
  for: new RegExp(`^for\\s+(${NAME})\\s+in\\s+(.+)$`, 'd'),
  while: /^while\s+(.+)$/d,
  case: /^case\s+(.+?)(?:\s+in)?$/d,
  terminated: /^(.*[^;])?(;;&|;;|;&)$/,
  terminator: /^(;;&|;;|;&)$/,
  echo: /^->\s*(.*)$/d,
  function: new RegExp(`^(?:\\.|\\.?${FUNCTION_NAME})\\(`),
  declaration: new RegExp(`^(${NAME})(<([^>]*)>)$`, 'd'),
  typedAssignment: new RegExp(`^(${NAME})(<([^>]*)>)\\s*=\\s*(.*)$`, 'd'),
  assignment: new RegExp(`^(${NAME})=(.*)$`, 'd'),
  param: /^(\*|-{0,2})([A-Za-z_][A-Za-z0-9_-]*)(?:<([^>]*)>)?(\.\.\.)?$/,
  literals: /^([^({]*)[({](.*)[)}]$/,
  list: /^\((.*)\)$/,
  math: /^[^"'()]+\s[-+*/%]\s[^"'()]+$/,
  closer: /^(fi|done|then|do|esac)(?=[\s;]|$)/,
  bareCondition: /^(if|while|case)$/,
  badFor: /^for(?=\s|$)/,
  spacedAssignment: new RegExp(`^(${NAME})(<[^>]*>)?\\s+=(?!=)`),
  optionRef: /\$(--?)([A-Za-z_][A-Za-z0-9_-]*)/g,
  comment: /(^|\s)#.*$/,
  heredoc: /<<(?!<)/,
};

function position(line, character) {
  return { line, character };
}

function span(line, start, end, endLine = line) {
  return { start: position(line, start), end: position(endLine, end) };
}

function tokenRange(token) {
  return span(token.line, token.col, token.end, token.endLine);
}

function words(input) {
  const out = [];
  let word = '';
  let start = -1;
  let quote = '';
  for (let i = 0; i < input.length; i++) {
    const c = input[i];
    if (quote) {
      word += c;
      if (c === quote) quote = '';
    } else if (c === '"' || c === "'") {
      if (start < 0) start = i;
      word += c;
      quote = c;
    } else if (/\s/.test(c)) {
      if (word) out.push({ text: word, index: start });
      word = '';
      start = -1;
    } else {
      if (start < 0) start = i;
      word += c;
    }
  }
  if (word) out.push({ text: word, index: start });
  return out;
}

function splitWords(input) {
  return words(input).map((w) => w.text);
}

function scanShell(text) {
  const mask = text.split('');
  const stack = [];
  const comments = [];
  const blank = (k) => {
    if (k < mask.length && mask[k] !== '\n') mask[k] = ' ';
  };
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    const top = stack.length ? stack[stack.length - 1].kind : null;
    if (top === 'sq') {
      blank(i);
      if (c === "'") stack.pop();
      i++;
      continue;
    }
    if (top === 'dq') {
      if (c === '\\') {
        blank(i);
        blank(i + 1);
        i += 2;
      } else if (c === '"') {
        blank(i);
        stack.pop();
        i++;
      } else if (c === '$' && text[i + 1] === '(') {
        stack.push({ kind: 'sub', index: i });
        i += 2;
      } else if (c === '`') {
        stack.push({ kind: 'bt', index: i });
        i++;
      } else {
        blank(i);
        i++;
      }
      continue;
    }
    if (c === '\\') {
      i += 2;
    } else if (c === '#' && (i === 0 || /\s/.test(text[i - 1]))) {
      let end = text.indexOf('\n', i);
      if (end < 0) end = text.length;
      for (let k = i; k < end; k++) blank(k);
      comments.push([i, end]);
      i = end;
    } else if (c === "'" || c === '"') {
      blank(i);
      stack.push({ kind: c === "'" ? 'sq' : 'dq', index: i });
      i++;
    } else if (c === '`') {
      if (top === 'bt') stack.pop();
      else stack.push({ kind: 'bt', index: i });
      i++;
    } else if (c === '$' && text[i + 1] === '(') {
      stack.push({ kind: 'sub', index: i });
      i += 2;
    } else if (c === '(' && (top === 'sub' || top === 'paren')) {
      stack.push({ kind: 'paren', index: i });
      i++;
    } else if (c === ')' && (top === 'sub' || top === 'paren')) {
      stack.pop();
      i++;
    } else {
      i++;
    }
  }
  const open = stack.find((s) => s.kind === 'sq' || s.kind === 'dq');
  return { mask: mask.join(''), comments, unterminated: open || null };
}

function parseType(raw) {
  let typePart = raw;
  let literals = null;
  const m = RE.literals.exec(raw);
  if (m) {
    typePart = m[1];
    literals = splitWords(m[2]);
  }
  const datatype = typePart.split(/\s+/).filter(Boolean);
  const description = datatype.join(' ') + (literals ? ` (${literals.join(' ')})` : '');
  return {
    raw,
    datatype,
    literals,
    base: datatype.find((w) => w !== 'const') || '',
    isConst: datatype.includes('const'),
    description,
  };
}

function classifyValue(raw) {
  const m = RE.list.exec(raw);
  if (m) {
    const items = splitWords(m[1]);
    if (items.length && items.every((w) => /^[^=]+=/.test(w))) return { kind: 'dict', items };
    return { kind: 'list', items };
  }
  if (RE.math.test(raw)) return { kind: 'math' };
  return { kind: 'scalar' };
}

function literalValue(raw) {
  const m = /^"([^"$`\\]*)"$/.exec(raw) || /^'([^']*)'$/.exec(raw);
  if (m) return m[1];
  if (/^[a-zA-Z0-9_.+-]+$/.test(raw)) return raw;
  return null;
}

function hasTypeTest(info) {
  return Boolean(info.literals || PATTERNS[info.base]);
}

function accepts(value, info) {
  if (info.literals) return info.literals.includes(value);
  const pattern = PATTERNS[info.base];
  return !pattern || pattern.test(value);
}

function leavesOf(params, groupRepeat = false, groupOptional = false, out = []) {
  for (const p of params) {
    if (p.kind === 'group') {
      leavesOf(p.params, groupRepeat || p.repeat, groupOptional || p.optional, out);
    } else if (p.kind === 'exclusive') {
      leavesOf(p.options, groupRepeat, groupOptional, out);
    } else if (p.kind === 'arg' || p.kind === 'opt' || p.kind === 'rest') {
      const isBool = p.datatype.datatype.length === 1 && p.datatype.datatype[0] === 'bool';
      p.array = p.kind === 'rest' || ((p.repeat || groupRepeat) && !isBool);
      p.effectiveOptional = p.optional || groupOptional;
      out.push(p);
    }
  }
  return out;
}

class Analyzer {
  constructor(source) {
    this.lines = source.split(/\r?\n/).map((raw) => {
      const text = raw.trim();
      return { raw, text, indent: raw.length - raw.trimStart().length };
    });
    this.idx = 0;
    this.diagnostics = [];
    this.functions = [];
    this.callable = new Map();
    this.variables = new Map();
    this.types = new Map();
    this.definedFunctions = new Map();
  }

  run() {
    this.body = this.parseBlock(-1);
    this.collectFunctions(this.body, []);
    this.checkWhitespace();
    this.check(this.body, { fnChain: [], inBlock: false });
    this.diagnostics.sort(
      (a, b) => a.range.start.line - b.range.start.line || a.range.start.character - b.range.start.character,
    );
    return this;
  }

  report(severity, range, message, code) {
    this.diagnostics.push({ severity, range, message, code });
  }

  lineRange(i) {
    const line = this.lines[i];
    return span(i, line.indent, line.indent + line.text.length);
  }

  keywordRange(i, keyword) {
    const line = this.lines[i];
    return span(i, line.indent, line.indent + keyword.length);
  }

  peek() {
    while (this.idx < this.lines.length) {
      const line = this.lines[this.idx];
      if (line.text && !line.text.startsWith('#')) return line;
      this.idx++;
    }
    return null;
  }

  parseBlock(parentIndent) {
    const nodes = [];
    let blockIndent = null;
    let line;
    while ((line = this.peek()) && line.indent > parentIndent) {
      if (blockIndent === null) {
        blockIndent = line.indent;
      } else if (line.indent !== blockIndent) {
        const message =
          line.indent > blockIndent
            ? 'Unexpected indentation: the statement above does not open a block'
            : `Inconsistent indentation: expected ${blockIndent} columns to match the rest of this block`;
        this.report('warning', this.lineRange(this.idx), message, 'indentation');
      }
      nodes.push(this.parseStatement(nodes[nodes.length - 1]));
    }
    return nodes;
  }

  parseStatement(prev) {
    const i = this.idx;
    const line = this.lines[i];
    const t = line.text;
    let m;
    if ((m = RE.if.exec(t))) return this.parseIf(m);
    if ((m = RE.orphan.exec(t))) return this.parseOrphan(m[1], prev);
    if ((m = RE.for.exec(t))) return this.parseLoop('for', m);
    if ((m = RE.while.exec(t))) return this.parseLoop('while', m);
    if ((m = RE.case.exec(t))) return this.parseCase(m);
    if ((m = RE.echo.exec(t))) {
      this.idx++;
      return { kind: 'echo', line: i, value: m[1], col: line.indent + m.indices[1][0] };
    }
    if (RE.function.test(t)) return this.parseFunction();
    if ((m = RE.declaration.exec(t))) {
      this.idx++;
      return {
        kind: 'declaration',
        line: i,
        name: m[1],
        nameRange: span(i, line.indent, line.indent + m[1].length),
        datatype: parseType(m[3]),
        typeRange: span(i, line.indent + m.indices[2][0], line.indent + m.indices[2][1]),
      };
    }
    if ((m = RE.typedAssignment.exec(t))) {
      this.idx++;
      return {
        kind: 'assignment',
        line: i,
        name: m[1],
        nameRange: span(i, line.indent, line.indent + m[1].length),
        datatype: parseType(m[3]),
        typeRange: span(i, line.indent + m.indices[2][0], line.indent + m.indices[2][1]),
        value: m[4],
        valueCol: line.indent + m.indices[4][0],
      };
    }
    if ((m = RE.assignment.exec(t))) {
      this.idx++;
      return {
        kind: 'assignment',
        line: i,
        name: m[1],
        nameRange: span(i, line.indent, line.indent + m[1].length),
        datatype: null,
        value: m[2],
        valueCol: line.indent + m.indices[2][0],
      };
    }
    return this.parseBash();
  }

  parseIf(match) {
    const start = this.idx;
    const indent = this.lines[start].indent;
    const branches = [];
    const addBranch = (keyword, m) => {
      const lineIdx = this.idx;
      const col = this.lines[lineIdx].indent + m.indices[1][0];
      this.idx++;
      branches.push({ keyword, condition: m[1], line: lineIdx, col, body: this.parseBlock(indent) });
    };
    addBranch('if', match);
    let line;
    let m;
    while ((line = this.peek()) && line.indent === indent && (m = RE.elif.exec(line.text))) addBranch('elif', m);
    let elseBranch = null;
    if ((line = this.peek()) && line.indent === indent && line.text === 'else') {
      const lineIdx = this.idx;
      this.idx++;
      elseBranch = { keyword: 'else', line: lineIdx, body: this.parseBlock(indent) };
    }
    return { kind: 'if', line: start, indent, branches, else: elseBranch };
  }

  parseOrphan(keyword, prev) {
    const i = this.idx;
    const line = this.lines[i];
    const followsIf = prev && prev.kind === 'if' && prev.indent === line.indent && !prev.else;
    let message = `'${keyword}' without matching 'if': it must be indented to the same level as its 'if'`;
    let code = 'orphan-branch';
    if (followsIf && line.text === 'elif') {
      message = "'elif' needs a condition";
    } else if (followsIf && keyword === 'else' && /^else\s+#/.test(line.text)) {
      message = "Nothing may follow 'else' on its line, not even a comment";
      code = 'else-trailing';
    } else if (followsIf && keyword === 'else') {
      message = "'else' takes no condition: did you mean 'elif'?";
      code = 'else-trailing';
    }
    this.report('error', this.lineRange(i), message, code);
    this.idx++;
    return { kind: 'orphan', line: i, indent: line.indent, body: this.parseBlock(line.indent) };
  }

  parseLoop(kind, m) {
    const i = this.idx;
    const line = this.lines[i];
    this.idx++;
    const node = { kind, line: i, indent: line.indent };
    if (kind === 'for') {
      node.var = m[1];
      node.varRange = span(i, line.indent + m.indices[1][0], line.indent + m.indices[1][1]);
      node.items = m[2];
      node.col = line.indent + m.indices[2][0];
    } else {
      node.condition = m[1];
      node.col = line.indent + m.indices[1][0];
    }
    node.body = this.parseBlock(line.indent);
    return node;
  }

  parseCase(m) {
    const i = this.idx;
    const line = this.lines[i];
    this.idx++;
    const node = { kind: 'case', line: i, indent: line.indent, subject: m[1], col: line.indent + m.indices[1][0], arms: [] };
    let arm;
    while ((arm = this.peek()) && arm.indent > line.indent) node.arms.push(this.parseCaseArm());
    return node;
  }

  parseCaseArm() {
    const i = this.idx;
    const line = this.lines[i];
    const t = line.text;
    let quote = '';
    let depth = 0;
    let close = -1;
    for (let k = 0; k < t.length && close < 0; k++) {
      const c = t[k];
      if (quote) {
        if (c === quote) quote = '';
      } else if (c === '"' || c === "'") {
        quote = c;
      } else if (c === '(') {
        depth++;
      } else if (c === ')' && depth) {
        depth--;
      } else if (c === ')') {
        close = k;
      }
    }
    if (close <= 0) {
      const message = RE.terminator.test(t)
        ? `Unexpected '${t}': end the arm's line with it or indent it under the arm`
        : "Expected 'pattern)' to start a case arm";
      this.report('error', this.lineRange(i), message, 'case-arm');
      this.idx++;
      return { line: i, pattern: null, body: this.parseBlock(line.indent) };
    }

    const arm = { line: i, pattern: t.slice(0, close), body: [] };
    const after = t.slice(close + 1);
    const col = line.indent + close + 1 + after.length - after.trimStart().length;
    let rest = after.trim();
    const terminated = RE.terminated.exec(rest);
    if (terminated) rest = (terminated[1] || '').trimEnd();
    if (rest) {
      this.lines[i] = { raw: line.raw, text: rest, indent: col };
      arm.body.push(this.parseStatement());
    } else {
      this.idx++;
    }
    arm.body.push(...this.parseBlock(line.indent));
    const last = arm.body[arm.body.length - 1];
    if (last && last.kind === 'bash' && last.lines.length === 1 && RE.terminator.test(last.text)) arm.body.pop();
    return arm;
  }

  parseBash() {
    const i = this.idx;
    const indent = this.lines[i].indent;
    const lines = [i];
    this.idx++;
    let line;
    while ((line = this.peek()) && line.indent > indent) {
      lines.push(this.idx);
      this.idx++;
    }
    return { kind: 'bash', line: i, indent, lines, text: this.lines[i].text };
  }

  parseFunction() {
    const start = this.idx;
    const line = this.lines[start];
    const indent = line.indent;
    const paren = line.text.indexOf('(');
    let name = line.text.slice(0, paren);
    let kind = 'function';
    let nameCol = indent;
    if (name.startsWith('.')) {
      kind = 'command';
      name = name.slice(1);
      nameCol++;
    }
    const nameRange = name ? span(start, nameCol, nameCol + name.length) : span(start, indent, nameCol);

    const chars = [];
    let depth = 1;
    let lineIdx = start;
    let segCol = indent + paren + 1;
    let seg = line.text.slice(paren + 1);
    let tail = '';
    let tailCol = 0;
    let closed = false;
    for (;;) {
      const comment = RE.comment.exec(seg);
      if (comment) seg = seg.slice(0, comment.index);
      for (let k = 0; k < seg.length; k++) {
        const ch = seg[k];
        if (ch === '(') depth++;
        if (ch === ')') depth--;
        if (depth === 0) {
          tail = seg.slice(k + 1);
          tailCol = segCol + k + 1;
          closed = true;
          break;
        }
        chars.push({ ch, line: lineIdx, col: segCol + k });
      }
      if (closed || lineIdx + 1 >= this.lines.length) break;
      chars.push({ ch: ' ', line: lineIdx, col: segCol + seg.length });
      lineIdx++;
      segCol = 0;
      seg = this.lines[lineIdx].raw;
    }

    let params = [];
    let inline = null;
    if (!closed) {
      this.report('error', span(start, indent + paren, indent + paren + 1), `Unclosed '(' in ${name}(...)`, 'unclosed-paren');
      lineIdx = start;
    } else {
      const lead = tail.length - tail.trimStart().length;
      tail = tail.trim();
      tailCol += lead;
      const echo = RE.echo.exec(tail);
      if (echo) {
        inline = { kind: 'echo', line: lineIdx, value: echo[1], col: tailCol + echo.indices[1][0] };
      } else if (tail) {
        const message = tail.startsWith('{')
          ? `Unexpected '{' after ${name}(...): function bodies are defined by indentation, not braces`
          : `Unexpected '${tail}' after ${name}(...): only '-> <value>' may follow the parameters`;
        this.report('error', span(lineIdx, tailCol, tailCol + tail.length), message, 'function-tail');
      }
      const tokens = this.tokenizeParams(chars);
      const state = { pos: 0 };
      params = this.parseParamSequence(tokens, state);
      if (state.pos < tokens.length) {
        const token = tokens[state.pos];
        this.report('error', tokenRange(token), `Unexpected '${token.text}' in parameters`, 'param');
      }
    }

    this.idx = lineIdx + 1;
    const body = this.parseBlock(indent);
    if (inline) body.unshift(inline);
    const header = this.lines
      .slice(start, lineIdx + 1)
      .map((l) => l.raw.slice(Math.min(indent, l.indent)).trimEnd())
      .join('\n');
    return { kind, name, line: start, indent, closeLine: lineIdx, nameRange, params, leaves: leavesOf(params), body, header };
  }

  tokenizeParams(chars) {
    const tokens = [];
    let word = null;
    let inType = null;
    const extend = (c) => {
      word.text += c.ch;
      word.end = c.col + 1;
      word.endLine = c.line;
    };
    const begin = (c) => ({ text: '', line: c.line, col: c.col, end: c.col, endLine: c.line });
    const flush = () => {
      if (word) tokens.push(word);
      word = null;
    };
    for (const c of chars) {
      if (inType) {
        extend(c);
        if (c.ch === '>') inType = null;
      } else if (c.ch === '<') {
        if (!word && tokens.length && !PARAM_PUNCTUATION.has(tokens[tokens.length - 1].text)) word = tokens.pop();
        if (!word) word = begin(c);
        extend(c);
        inType = c;
      } else if (PARAM_PUNCTUATION.has(c.ch)) {
        flush();
        word = begin(c);
        extend(c);
        flush();
      } else if (/\s/.test(c.ch)) {
        flush();
      } else {
        if (!word) word = begin(c);
        extend(c);
      }
    }
    if (inType) this.report('error', span(inType.line, inType.col, inType.col + 1), "Unclosed '<' in parameter type", 'param');
    flush();
    return tokens;
  }

  parseParamSequence(tokens, state) {
    const items = [];
    while (state.pos < tokens.length && tokens[state.pos].text !== ']') items.push(this.parseParamAlternatives(tokens, state));
    return items;
  }

  parseParamAlternatives(tokens, state) {
    const options = [this.parseParamItem(tokens, state)];
    while (state.pos < tokens.length && tokens[state.pos].text === '|') {
      state.pos++;
      options.push(this.parseParamItem(tokens, state));
    }
    return options.length > 1 ? { kind: 'exclusive', options } : options[0];
  }

  parseParamItem(tokens, state) {
    const token = tokens[state.pos];
    if (!token) {
      const last = tokens[tokens.length - 1];
      this.report('error', tokenRange(last), `Expected a parameter after '${last.text}'`, 'param');
      return { kind: 'invalid' };
    }
    state.pos++;
    if (token.text !== '[') return this.parseParam(token);
    const params = this.parseParamSequence(tokens, state);
    if (state.pos < tokens.length && tokens[state.pos].text === ']') {
      state.pos++;
    } else {
      this.report('error', tokenRange(token), "Missing ']' for this '['", 'param');
    }
    let repeat = false;
    if (state.pos < tokens.length && tokens[state.pos].text === '...') {
      repeat = true;
      state.pos++;
    }
    if (params.length === 1 && (params[0].kind === 'arg' || params[0].kind === 'opt' || params[0].kind === 'rest')) {
      return { ...params[0], optional: true, repeat: params[0].repeat || repeat };
    }
    return { kind: 'group', optional: true, repeat, params, token };
  }

  parseParam(token) {
    const m = RE.param.exec(token.text);
    if (!m) {
      const message =
        token.text === '|' || token.text === ']'
          ? `Expected a parameter before '${token.text}'`
          : `Invalid parameter '${token.text}': expected name, -f, --flag, or *name, optionally followed by <type> and ...`;
      this.report('error', tokenRange(token), message, 'param');
      return { kind: 'invalid', token };
    }
    const [, prefix, rawName, rawType, dots] = m;
    const isOpt = prefix.startsWith('-');
    const isRest = prefix === '*';
    let typeText = rawType || '';
    let defaultValue = null;
    const eq = typeText.indexOf('=');
    if (eq >= 0) {
      defaultValue = typeText.slice(eq + 1);
      typeText = typeText.slice(0, eq);
    }
    return {
      kind: isRest ? 'rest' : isOpt ? 'opt' : 'arg',
      name: rawName.replace(/-/g, '_'),
      flag: isOpt ? prefix + rawName : null,
      optional: false,
      repeat: Boolean(dots),
      datatype: parseType(typeText.trim() ? typeText : isOpt ? 'bool' : 'str'),
      default: defaultValue,
      token,
    };
  }

  collectFunctions(nodes, chain) {
    for (const node of nodes) {
      if (node.kind === 'function' || node.kind === 'command') {
        node.parent = chain[chain.length - 1] || null;
        this.functions.push(node);
        if (node.kind === 'function' || !chain.length) this.callable.set(node.name, node);
        this.collectFunctions(node.body, [...chain, node]);
      } else {
        for (const child of childBlocks(node)) this.collectFunctions(child, chain);
      }
    }
  }

  checkWhitespace() {
    let style = null;
    this.lines.forEach((line, i) => {
      if (!line.text || line.text.startsWith('#')) return;
      const lead = line.raw.slice(0, line.raw.length - line.raw.trimStart().length);
      if (lead.includes('\t') && lead.includes(' ')) {
        this.report('warning', span(i, 0, lead.length), 'Mixed tabs and spaces: the compiler counts a tab as a single column', 'indent-style');
      } else if (lead) {
        const current = lead[0] === '\t' ? 'tabs' : 'spaces';
        if (!style) {
          style = { kind: current, line: i };
        } else if (current !== style.kind) {
          this.report(
            'warning',
            span(i, 0, lead.length),
            `Indented with ${current}, but line ${style.line + 1} uses ${style.kind}: the compiler counts a tab as a single column`,
            'indent-style',
          );
        }
      }
      const trimmed = line.raw.trimEnd().length;
      if (trimmed < line.raw.length) {
        this.report(
          'warning',
          span(i, trimmed, line.raw.length),
          'Trailing whitespace: the compiler counts it as indentation, which can break block nesting',
          'trailing-whitespace',
        );
      }
    });
  }

  check(nodes, scope) {
    for (const node of nodes) this.checkNode(node, scope);
  }

  checkNode(node, scope) {
    switch (node.kind) {
      case 'if':
        for (const branch of node.branches) {
          this.checkCondition(branch.line, branch.col, branch.condition, scope, branch.keyword);
          this.checkBody(branch, branch.keyword, scope);
        }
        if (node.else) this.checkBody(node.else, 'else', scope);
        break;
      case 'for':
        this.checkCondition(node.line, node.col, node.items, scope, 'for');
        this.checkBody(node, 'for', scope);
        break;
      case 'while':
        this.checkCondition(node.line, node.col, node.condition, scope, 'while');
        this.checkBody(node, 'while', scope);
        break;
      case 'case':
        this.checkCondition(node.line, node.col, node.subject, scope, 'case');
        if (!node.arms.length) {
          this.report('warning', this.keywordRange(node.line, 'case'), "Empty 'case' block: indent the 'pattern)' arms that belong to it", 'empty-block');
        }
        for (const arm of node.arms) this.check(arm.body, { ...scope, inBlock: true });
        break;
      case 'echo':
        this.checkText([{ line: node.line, col: node.col, text: node.value }], scope, { calls: 'substitution' });
        break;
      case 'declaration':
        this.validateType(node.datatype, node.typeRange);
        this.types.set(node.name, node.datatype);
        this.defineVariable(node);
        break;
      case 'assignment':
        this.checkAssignment(node, scope);
        break;
      case 'function':
      case 'command':
        this.checkFunction(node, scope);
        break;
      case 'bash':
        this.checkBash(node, scope);
        break;
      case 'orphan':
        this.check(node.body, scope);
        break;
    }
  }

  checkBody(owner, keyword, scope) {
    if (!owner.body.length) {
      this.report('warning', this.keywordRange(owner.line, keyword), `Empty '${keyword}' block: indent the lines that belong to it`, 'empty-block');
    }
    this.check(owner.body, { ...scope, inBlock: true });
  }

  checkCondition(line, col, text, scope, keyword) {
    const target = { for: 'item list', case: 'value' }[keyword] || 'condition';
    this.checkText([{ line, col, text }], scope, {
      calls: 'substitution',
      comment: `Comments after '${keyword}' are compiled into the ${target} and break it: move the comment to its own line`,
    });
  }

  checkAssignment(node, scope) {
    let info = node.datatype;
    if (info) {
      this.validateType(info, node.typeRange);
      this.types.set(node.name, info);
    } else {
      info = this.types.get(node.name) || null;
      if (info && info.isConst) {
        const declared = this.variables.get(node.name);
        const where = declared ? ` (declared on line ${declared.line + 1})` : '';
        this.report('error', node.nameRange, `Cannot reassign const '${node.name}'${where}`, 'const-reassign');
      }
      if (/^\s/.test(node.value)) {
        this.report(
          'warning',
          span(node.line, node.valueCol - 1, node.valueCol + node.value.length - node.value.trimStart().length),
          `Space after '=': this runs '${node.value.trim().split(/\s+/)[0]}' as a command with ${node.name} set to an empty string`,
          'spaced-assignment',
        );
      }
    }
    this.defineVariable(node);

    const valueRange = span(node.line, node.valueCol, node.valueCol + node.value.length);
    if (info) {
      const value = classifyValue(node.value);
      if ((value.kind === 'list' || value.kind === 'dict') && SCALARS.includes(info.base)) {
        this.report('warning', valueRange, `A ${value.kind} is assigned to '${node.name}', which is declared as <${info.description}>`, 'type-mismatch');
      } else if (info.base === 'list' && value.kind === 'dict') {
        this.report('warning', valueRange, `Key=value pairs are assigned to '${node.name}', which is declared as a list`, 'type-mismatch');
      } else if (info.base === 'dict' && value.kind === 'list' && value.items.length) {
        this.report('warning', valueRange, `Dict values must be key=value pairs, e.g. (a=1 b=2)`, 'type-mismatch');
      } else if (info.base === 'list' && value.kind === 'list') {
        this.checkListItems(node, info);
      } else if (value.kind === 'scalar' && hasTypeTest(info)) {
        const literal = literalValue(node.value);
        if (literal !== null && !accepts(literal, info)) {
          this.report('error', valueRange, `${node.name}: expected ${info.description}, received ${node.value}`, 'type-mismatch');
        }
      }
    }
    this.checkText([{ line: node.line, col: node.valueCol, text: node.value }], scope, { calls: 'substitution' });
  }

  checkListItems(node, info) {
    const element = info.datatype.filter((w) => w !== 'const')[1];
    const pattern = PATTERNS[element];
    if (!pattern) return;
    const inner = node.value.slice(1, -1);
    for (const item of words(inner)) {
      const literal = literalValue(item.text);
      if (literal !== null && !pattern.test(literal)) {
        const start = node.valueCol + 1 + item.index;
        this.report('warning', span(node.line, start, start + item.text.length), `'${item.text}' is not a valid ${element}`, 'type-mismatch');
      }
    }
  }

  checkFunction(node, scope) {
    const label = functionLabel(node);
    if (node.kind === 'command' && scope.inBlock) {
      this.report(
        'warning',
        node.nameRange,
        `Command '${label}' is inside an if/for/while/case block and will be ignored by the compiler: define commands at the top level or directly inside another function`,
        'nested-command',
      );
    }
    const key = [...scope.fnChain.map((f) => f.name), label].join(' ');
    const prior = this.definedFunctions.get(key);
    if (prior) {
      this.report('warning', node.nameRange, `'${label}' is already defined on line ${prior.line + 1}`, 'duplicate-function');
    } else {
      this.definedFunctions.set(key, node);
    }
    this.checkParams(node);
    if (!node.body.length) {
      this.report('warning', node.nameRange, `'${label}' has an empty body: indent the lines that belong to it`, 'empty-block');
    }
    this.check(node.body, { fnChain: [...scope.fnChain, node], inBlock: false });
  }

  checkParams(node) {
    const seen = new Map();
    let rest = null;
    for (const leaf of node.leaves) {
      const range = tokenRange(leaf.token);
      const prior = seen.get(leaf.name);
      if (prior) {
        this.report('error', range, `Duplicate parameter '${leaf.name}' (already declared on line ${prior.token.line + 1})`, 'duplicate-param');
      } else {
        seen.set(leaf.name, leaf);
      }
      this.validateType(leaf.datatype, range);
      if (leaf.default !== null) {
        if (leaf.array) {
          this.report('warning', range, 'Defaults are ignored for repeatable parameters', 'param-default');
        } else {
          const literal = literalValue(leaf.default);
          if (literal !== null && hasTypeTest(leaf.datatype) && !accepts(literal, leaf.datatype)) {
            this.report('error', range, `Default ${leaf.default} is not a valid ${leaf.datatype.description}`, 'param-default');
          }
        }
      }
      if (leaf.kind === 'arg') {
        if (rest) {
          this.report(
            'warning',
            range,
            `'${leaf.name}' comes after the repeatable argument '${rest.name}...' and will never receive its own value`,
            'param-order',
          );
        } else if (leaf.array) {
          rest = leaf;
        }
      }
      if (!(leaf.kind === 'opt' && leaf.datatype.base === 'bool') && !leaf.array) {
        this.types.set(leaf.name, leaf.datatype);
      }
    }
    const walk = (params) => {
      for (const p of params) {
        if (p.kind === 'group') {
          if (!p.params.length) this.report('warning', tokenRange(p.token), 'Empty optional group', 'param');
          walk(p.params);
        } else if (p.kind === 'exclusive') {
          walk(p.options);
        }
      }
    };
    walk(node.params);
    const stars = node.leaves.filter((leaf) => leaf.kind === 'rest');
    if (stars.length > 1) {
      for (const leaf of stars.slice(1)) {
        this.report('error', tokenRange(leaf.token), 'Only one *param is allowed', 'param');
      }
    } else if (stars.length === 1 && node.leaves[node.leaves.length - 1] !== stars[0]) {
      this.report('error', tokenRange(stars[0].token), `'*${stars[0].name}' must be the last parameter`, 'param-order');
    }
  }

  checkBash(node, scope) {
    const i = node.line;
    const t = node.text;
    let m;
    if ((m = RE.closer.exec(t))) {
      this.report('error', this.keywordRange(i, m[1]), `Unexpected '${m[1]}': blocks are closed by indentation, so remove this line`, 'block-closer');
    } else if ((m = RE.bareCondition.exec(t))) {
      this.report('error', this.lineRange(i), `'${m[1]}' needs a ${m[1] === 'case' ? 'value to match' : 'condition'}`, 'missing-condition');
    } else if (RE.badFor.test(t)) {
      this.report('error', this.lineRange(i), "Expected 'for <name> in <items>'", 'bad-for');
    } else if ((m = RE.spacedAssignment.exec(t))) {
      this.report(
        'warning',
        this.lineRange(i),
        `Spaces around '=' make this run '${m[1]}' as a command: write ${m[1]}${m[2] || ''}=value`,
        'spaced-assignment',
      );
    }
    const segments = node.lines.map((idx) => ({ line: idx, col: this.lines[idx].indent, text: this.lines[idx].text }));
    const joined = segments.map((s) => s.text).join('\n');
    this.checkText(segments, scope, { calls: 'command', quotes: !RE.heredoc.test(joined) });
  }

  checkText(segments, scope, opts) {
    const text = segments.map((s) => s.text).join('\n');
    const at = (offset) => {
      let o = offset;
      for (const s of segments) {
        if (o <= s.text.length) return position(s.line, s.col + o);
        o -= s.text.length + 1;
      }
      const last = segments[segments.length - 1];
      return position(last.line, last.col + last.text.length);
    };
    const rangeAt = (a, b) => ({ start: at(a), end: at(b) });
    const scanned = scanShell(text);

    if (opts.quotes !== false && scanned.unterminated) {
      const { kind, index } = scanned.unterminated;
      this.report('warning', rangeAt(index, index + 1), `Unterminated ${kind === 'sq' ? 'single' : 'double'} quote`, 'unterminated-quote');
    }
    if (opts.comment && scanned.comments.length) {
      const [start, end] = scanned.comments[0];
      this.report('warning', rangeAt(start, end), opts.comment, 'trailing-comment');
    }
    const inComment = (index) => scanned.comments.some(([a, b]) => index >= a && index < b);

    for (const m of text.matchAll(RE.optionRef)) {
      if (inComment(m.index)) continue;
      const [full, dashes, rawName] = m;
      const name = rawName.replace(/-/g, '_');
      const range = rangeAt(m.index, m.index + full.length);
      if (!scope.fnChain.length) {
        if (dashes === '--') this.report('warning', range, `'${full}' refers to an option, but is used outside of a function`, 'unknown-option-ref');
        continue;
      }
      const known = scope.fnChain.some((fn) => fn.leaves.some((leaf) => leaf.name === name));
      if (!known) {
        const fn = scope.fnChain[scope.fnChain.length - 1];
        this.report('warning', range, `'${full}' does not match any parameter of '${fn.name}'`, 'unknown-option-ref');
      }
    }

    this.checkCalls(text, scanned.mask, rangeAt, opts.calls === 'command');
  }

  checkCalls(text, mask, rangeAt, allowStart) {
    for (const m of mask.matchAll(new RegExp(FUNCTION_NAME, 'g'))) {
      const fn = this.callable.get(m[0]);
      if (!fn || !fn.leaves.length) continue;
      const before = mask[m.index - 1];
      if (before !== undefined && /[\w$.\-{/]/.test(before)) continue;
      const after = mask[m.index + m[0].length];
      if (after !== undefined && !/[\s;&|)`]/.test(after)) continue;
      const prev = mask.slice(0, m.index).replace(/[ \t]+$/, '');
      const prevChar = prev[prev.length - 1];
      const commandPosition = prevChar === undefined ? allowStart : /[;&|(`\n]/.test(prevChar);
      if (!commandPosition) continue;
      const argsStart = m.index + m[0].length;
      const term = mask.slice(argsStart).search(/[;&|)`\n]/);
      const argsEnd = term < 0 ? mask.length : argsStart + term;
      this.checkCallArgs(fn, text, argsStart, argsEnd, rangeAt);
    }
  }

  checkCallArgs(fn, text, start, end, rangeAt) {
    const flags = new Map();
    for (const leaf of fn.leaves) if (leaf.kind === 'opt') flags.set(leaf.flag, leaf);
    const args = words(text.slice(start, end)).map((w) => ({ text: w.text, index: start + w.index }));
    for (let k = 0; k < args.length; k++) {
      const arg = args[k];
      if (arg.text === '--') break;
      if (!/^-./.test(arg.text) || /[$`"']/.test(arg.text)) continue;
      const range = rangeAt(arg.index, arg.index + arg.text.length);
      const eq = /^(--?[A-Za-z_][A-Za-z0-9_-]*)=/.exec(arg.text);
      if (eq && flags.has(eq[1])) {
        this.report('warning', range, `Write '${eq[1]} <value>' instead of '${arg.text}': '=' is not supported`, 'unknown-option');
        continue;
      }
      const leaf = flags.get(arg.text);
      if (!leaf) {
        if (fn.leaves.some((item) => item.kind === 'rest')) continue;
        this.report('warning', range, `Unknown option '${arg.text}' for '${fn.name}'`, 'unknown-option');
        continue;
      }
      if (leaf.datatype.base === 'bool' && !leaf.array) continue;
      if (leaf.repeat) {
        while (k + 1 < args.length && !/^-./.test(args[k + 1].text)) k++;
      } else {
        k++;
      }
    }
  }

  validateType(info, range) {
    const words = info.datatype;
    if (!words.length) return this.report('error', range, 'Missing type', 'type');
    const unknown = words.find((w) => !TYPE_WORDS.has(w));
    if (unknown) {
      return this.report(
        'error',
        range,
        `Unknown type '${unknown}': expected ${[...SCALARS, ...COLLECTIONS].join(', ')}, optionally prefixed with const`,
        'type',
      );
    }
    const rest = words[0] === 'const' ? words.slice(1) : words;
    const [base, ...args] = rest;
    if (rest.includes('const')) return this.report('error', range, "'const' must come first, e.g. <const int>", 'type');
    if (!base) return this.report('error', range, "'const' needs a type, e.g. <const str>", 'type');
    if (SCALARS.includes(base) && args.length) return this.report('error', range, `'${base}' does not take type arguments`, 'type');
    if (base === 'list' && args.length > 1) return this.report('error', range, "'list' takes one element type, e.g. <list int>", 'type');
    if (base === 'dict' && args.length > 2) return this.report('error', range, "'dict' takes at most two types, e.g. <dict int str>", 'type');
    const nested = args.find((a) => !SCALARS.includes(a));
    if (nested) return this.report('error', range, `'${nested}' cannot be used as an element type: use a primitive type`, 'type');
    if (!info.literals) return;
    if (!SCALARS.includes(base)) return this.report('error', range, 'Allowed values can only be listed for primitive types', 'type');
    if (!info.literals.length) return this.report('warning', range, 'Empty list of allowed values', 'type');
    const pattern = PATTERNS[base];
    if (!pattern) return;
    for (const literal of info.literals) {
      const value = literalValue(literal) ?? literal;
      if (!pattern.test(value)) this.report('error', range, `Allowed value '${literal}' is not a valid ${base}`, 'type');
    }
  }

  defineVariable(node) {
    if (this.variables.has(node.name)) return;
    this.variables.set(node.name, {
      name: node.name,
      line: node.line,
      range: node.nameRange,
      datatype: node.datatype || null,
    });
  }
}

function childBlocks(node) {
  if (node.kind === 'if') return [...node.branches.map((b) => b.body), ...(node.else ? [node.else.body] : [])];
  if (node.kind === 'for' || node.kind === 'while' || node.kind === 'orphan') return [node.body];
  if (node.kind === 'case') return node.arms.map((arm) => arm.body);
  return [];
}

function endLine(node) {
  let last = node.kind === 'bash' ? node.lines[node.lines.length - 1] : node.closeLine ?? node.line;
  if (node.kind === 'case') for (const arm of node.arms) last = Math.max(last, arm.line);
  const blocks = node.kind === 'function' || node.kind === 'command' ? [node.body] : childBlocks(node);
  for (const block of blocks) {
    if (block.length) last = Math.max(last, endLine(block[block.length - 1]));
  }
  return last;
}

function functionLabel(node) {
  if (node.kind !== 'command') return node.name;
  return node.name ? `.${node.name}` : '.()';
}

function describeParam(leaf) {
  let text = leaf.kind === 'rest' ? `*${leaf.name}` : leaf.flag || leaf.name;
  if (leaf.kind === 'arg' ? leaf.datatype.raw !== 'str' : leaf.datatype.raw !== 'bool') text += `<${leaf.datatype.description}>`;
  if (leaf.repeat) text += '...';
  return leaf.optional ? `[${text}]` : text;
}

function buildSymbols(analyzer, nodes) {
  const symbols = [];
  const seen = new Set();
  const fullRange = (node) => {
    const last = endLine(node);
    return span(node.line, 0, analyzer.lines[last].raw.length, last);
  };
  const visit = (list) => {
    for (const node of list) {
      if (node.kind === 'function' || node.kind === 'command') {
        const params = node.leaves.map((leaf) => ({
          name: leaf.kind === 'rest' ? `*${leaf.name}` : leaf.flag || leaf.name,
          detail: leaf.datatype.description,
          kind: leaf.kind,
          range: tokenRange(leaf.token),
          selectionRange: tokenRange(leaf.token),
          children: [],
        }));
        symbols.push({
          name: functionLabel(node),
          detail: node.leaves.map(describeParam).join(' '),
          kind: node.kind,
          range: fullRange(node),
          selectionRange: node.nameRange,
          children: [...params, ...buildSymbols(analyzer, node.body)],
        });
      } else if ((node.kind === 'declaration' || node.kind === 'assignment') && !seen.has(node.name)) {
        seen.add(node.name);
        symbols.push({
          name: node.name,
          detail: node.datatype ? `<${node.datatype.description}>` : '',
          kind: node.datatype && node.datatype.isConst ? 'constant' : 'variable',
          range: fullRange(node),
          selectionRange: node.nameRange,
          children: [],
        });
      } else {
        for (const block of childBlocks(node)) visit(block);
      }
    }
  };
  visit(nodes);
  return symbols;
}

function analyze(source) {
  const analyzer = new Analyzer(source).run();
  const functions = analyzer.functions.map((fn) => ({ fn, start: fn.line, end: endLine(fn) }));
  return {
    diagnostics: analyzer.diagnostics,
    symbols: buildSymbols(analyzer, analyzer.body),
    functions: analyzer.functions,
    variables: analyzer.variables,
    findFunction(name) {
      return analyzer.callable.get(name) || analyzer.functions.find((fn) => fn.name === name) || null;
    },
    functionChainAt(line) {
      return functions
        .filter((f) => f.start <= line && line <= f.end)
        .sort((a, b) => b.start - a.start)
        .map((f) => f.fn);
    },
  };
}

module.exports = { analyze, describeParam, scanShell, splitWords, SCALARS, COLLECTIONS };
