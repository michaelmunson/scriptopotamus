'use strict';

const fs = require('fs');
const path = require('path');
const oniguruma = require('vscode-oniguruma');
const textmate = require('vscode-textmate');

const WASM_PATH = require.resolve('vscode-oniguruma/release/onig.wasm');

const grammarPromises = new Map();

function loadGrammar(scope = 'source.scriptopotamus') {
  if (!grammarPromises.has(scope)) {
    const onigLib = oniguruma.loadWASM(fs.readFileSync(WASM_PATH).buffer).then(() => ({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (s) => new oniguruma.OnigString(s),
    }));
    const contributions = require('../package.json').contributes.grammars;
    const registry = new textmate.Registry({
      onigLib,
      getInjections: (scopeName) => contributions
        .filter((g) => g.injectTo?.includes(scopeName)).map((g) => g.scopeName),
      loadGrammar: async (scopeName) => {
        if (scopeName === 'text.html.markdown') {
          const hostPath = process.env.SCRIPPO_TEST_MARKDOWN_GRAMMAR
            || path.join(__dirname, 'fixtures', 'markdown.tmLanguage.json');
          return textmate.parseRawGrammar(fs.readFileSync(hostPath, 'utf8'), hostPath);
        }
        const contribution = contributions.find((g) => g.scopeName === scopeName);
        if (!contribution) return null;
        const grammarPath = path.join(__dirname, '..', contribution.path);
        return textmate.parseRawGrammar(fs.readFileSync(grammarPath, 'utf8'), grammarPath);
      },
    });
    grammarPromises.set(scope, registry.loadGrammar(scope));
  }
  return grammarPromises.get(scope);
}

async function tokenize(source, scope) {
  const grammar = await loadGrammar(scope);
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
