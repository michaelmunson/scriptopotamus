'use strict';

const { scanShell } = require('./analyzer');

function registerDocumentationComments(vscode, context) {
  const decoration = vscode.window.createTextEditorDecorationType({
    color: new vscode.ThemeColor('scriptopotamus.documentationCommentForeground'),
  });

  function update(editor) {
    const doc = editor.document;
    if (doc.languageId !== 'scriptopotamus') {
      editor.setDecorations(decoration, []);
      return;
    }
    const text = doc.getText();
    const ranges = scanShell(text).comments
      .filter(([start]) => text.startsWith('#?', start))
      .map(([start, end]) => new vscode.Range(doc.positionAt(start), doc.positionAt(end)));
    editor.setDecorations(decoration, ranges);
  }

  context.subscriptions.push(
    decoration,
    vscode.window.onDidChangeVisibleTextEditors((editors) => editors.forEach(update)),
    vscode.workspace.onDidChangeTextDocument(({ document }) => {
      vscode.window.visibleTextEditors
        .filter((editor) => editor.document === document)
        .forEach(update);
    }),
  );
  vscode.window.visibleTextEditors.forEach(update);
}

module.exports = { registerDocumentationComments };
