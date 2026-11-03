'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { tokenize } = require('./tokenize');

const markdown = (source) => tokenize(source, 'text.html.markdown');
const scopes = (tokens, text) => {
  const token = tokens.find((t) => t.text === text);
  assert.ok(token, `missing token ${JSON.stringify(text)}`);
  return token.scopes.join(' ');
};

test('Markdown fences embed the scrippo grammar for both language names and fence styles', async () => {
  for (const fence of ['```', '~~~', '````']) {
    for (const language of ['scrippo', 'scriptopotamus', 'Scrippo']) {
      const lines = await markdown([
        `${fence}${language}`, 'name<str>="Ada"', '-> "$name"', fence, '# Heading',
      ].join('\n'));
      assert.match(scopes(lines[1], 'str'), /support\.type\.primitive/);
      assert.match(scopes(lines[1], 'Ada'), /meta\.embedded\.block\.scriptopotamus.*string\.quoted/);
      assert.match(scopes(lines[2], '->'), /keyword\.control\.echo/);
      assert.match(lines[4].flatMap((t) => t.scopes).join(' '), /markup\.heading\.markdown/);
      assert.ok(lines[3].every((t) => !t.scopes.includes('meta.embedded.block.scriptopotamus')));
    }
  }
});

test('Markdown closing fences terminate unfinished scrippo strings', async () => {
  const lines = await markdown('```scrippo\nmessage="unfinished\n```\n# Heading');
  assert.match(scopes(lines[1], 'unfinished'), /string\.quoted/);
  assert.match(lines[3].flatMap((t) => t.scopes).join(' '), /markup\.heading\.markdown/);
});

test('other fences and Markdown prose do not receive scrippo highlighting', async () => {
  const lines = await markdown([
    'name<str>="prose"', '`name<str>="inline"`',
    '```text', '```scrippo', 'name<str>="plain"', '```',
    '```scrippo-other', 'name<str>="other"', '```',
  ].join('\n'));
  assert.ok(lines.flat().every((t) => t.scopes.every((s) => !s.includes('scriptopotamus'))));
});

test('Markdown injection registers the embedded language', () => {
  const grammars = require('../package.json').contributes.grammars;
  const injection = grammars.find((g) => g.injectTo?.includes('text.html.markdown'));
  assert.equal(injection?.embeddedLanguages['meta.embedded.block.scriptopotamus'], 'scriptopotamus');
});
