'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { registerDocumentationComments } = require('../src/documentation-comments');

test('gray decorations cover documentation comments and refresh on edits and editor changes', () => {
  let text = '#? docs\n.run(\n  path #? path docs\n)\n  echo "#? string" # normal\n  #? body docs';
  const document = {
    languageId: 'scriptopotamus',
    getText: () => text,
    positionAt: (offset) => offset,
  };
  let ranges;
  const editor = { document, setDecorations: (_, value) => { ranges = value; } };
  let onEdit;
  let onVisible;
  let options;
  const vscode = {
    ThemeColor: class { constructor(id) { this.id = id; } },
    Range: class { constructor(start, end) { this.start = start; this.end = end; } },
    window: {
      visibleTextEditors: [editor],
      createTextEditorDecorationType: (value) => { options = value; return { dispose() {} }; },
      onDidChangeVisibleTextEditors: (fn) => { onVisible = fn; return { dispose() {} }; },
    },
    workspace: {
      onDidChangeTextDocument: (fn) => { onEdit = fn; return { dispose() {} }; },
    },
  };
  const context = { subscriptions: [] };
  registerDocumentationComments(vscode, context);
  const decorated = () => ranges.map(({ start, end }) => text.slice(start, end));
  assert.equal(options.color.id, 'scriptopotamus.documentationCommentForeground');
  assert.deepEqual(decorated(), ['#? docs', '#? path docs', '#? body docs']);
  text = '# regular\n#? changed';
  onEdit({ document });
  assert.deepEqual(decorated(), ['#? changed']);
  text = '#? opened';
  onVisible([editor]);
  assert.deepEqual(decorated(), ['#? opened']);
  document.languageId = 'plaintext';
  onEdit({ document });
  assert.deepEqual(ranges, []);
  assert.equal(context.subscriptions.length, 3);
});
