'use strict';

const fs = require('fs');
const path = require('path');
const oniguruma = require('vscode-oniguruma');
const textmate = require('vscode-textmate');

const GRAMMAR_PATH = path.join(__dirname, '..', 'syntaxes', 'scriptopotamus.tmLanguage.json');
const WASM_PATH = require.resolve('vscode-oniguruma/release/onig.wasm');

let grammarPromise;

function loadGrammar() {
  if (!grammarPromise) {
    const onigLib = oniguruma.loadWASM(fs.readFileSync(WASM_PATH).buffer).then(() => ({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (s) => new oniguruma.OnigString(s),
    }));
    const registry = new textmate.Registry({
      onigLib,
      loadGrammar: async (scopeName) =>
        scopeName === 'source.scriptopotamus'
          ? textmate.parseRawGrammar(fs.readFileSync(GRAMMAR_PATH, 'utf8'), GRAMMAR_PATH)
          : null,
    });
    grammarPromise = registry.loadGrammar('source.scriptopotamus');
  }
  return grammarPromise;
}

async function tokenize(source) {
  const grammar = await loadGrammar();
  let stack = textmate.INITIAL;
  return source.split('\n').map((line) => {
    const result = grammar.tokenizeLine(line, stack);
    stack = result.ruleStack;
    return result.tokens.map((t) => ({
      text: line.slice(t.startIndex, t.endIndex),
      scopes: t.scopes.slice(1),
    }));
  });
}

module.exports = { tokenize };

if (require.main === module) {
  const file = process.argv[2];
  tokenize(fs.readFileSync(file, 'utf8')).then((lines) => {
    lines.forEach((tokens, i) => {
      const shown = tokens
        .filter((t) => t.text.trim())
        .map((t) => `${JSON.stringify(t.text)}=${t.scopes.map((s) => s.replace('.scriptopotamus', '')).join('>') || '-'}`);
      console.log(`${i + 1}: ${shown.join('  ')}`);
    });
  });
}
