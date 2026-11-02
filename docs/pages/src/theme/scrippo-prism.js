const keywords =
  /\b(?:if|elif|else|for|while|case|in|fi|done|then|do|esac)\b|->/;

const builtins =
  /(?<![\w$-])(?:echo|printf|read|cd|source|shift|local|declare|export|readonly|unset|set|eval|exec|test|trap|wait|mapfile|let|return|exit|break|continue|prt|input|throw|list|dict|str|here)(?![\w-])/;

function registerScrippo(Prism) {
  if (Prism.languages.scrippo) {
    return;
  }

  const variable = /\$\{[^}\n]+\}|\$--?[\w-]+|\$\w+\[[^\]]*\]|\$\w+/;

  const interpolation = [
    {
      pattern: /\$\(\([\s\S]*?\)\)/,
      inside: {
        number: /\b\d+(?:\.\d+)?\b/,
        operator: /[-+*/%]/,
        variable,
        punctuation: /[()]/,
      },
    },
    {
      pattern: /\$\([^)\n]*\)/,
      inside: {
        variable,
        number: /\b\d+(?:\.\d+)?\b/,
        punctuation: /[()]/,
      },
    },
  ];

  const string = [
    {
      pattern: /\$(?:'(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*")/,
      greedy: true,
    },
    {
      pattern: /'(?:\\.|[^'\\])*'/,
      greedy: true,
    },
    {
      pattern: /"(?:\\.|[^"\\])*"/,
      greedy: true,
      inside: {
        interpolation,
        variable,
      },
    },
  ];

  Prism.languages.scrippo = {
    comment: [
      {
        pattern: /^#!.*/,
        greedy: true,
      },
      {
        pattern: /(^|[\s;])#\?.*/,
        lookbehind: true,
        greedy: true,
        alias: 'documentation',
      },
      {
        pattern: /(^|[\s;])#.*/,
        lookbehind: true,
        greedy: true,
      },
    ],
    type: {
      pattern: /<[^>\n]*>/,
      inside: {
        keyword: /\bconst\b/,
        builtin:
          /\b(?:int|integer|float|str|string|bool|boolean|any|list|dict|array)\b/,
        string,
        number: /\b\d+(?:\.\d+)+\b|\b\d+\b/,
        operator: /=/,
        punctuation: /[{}()<>]/,
      },
    },
    string,
    interpolation,
    function: [
      {
        pattern: /(^[ \t]*)\.(?:[A-Za-z_][\w-]*)?(?=[ \t]*\()/m,
        lookbehind: true,
      },
      {
        pattern: /@[A-Za-z_]\w*/,
      },
      {
        pattern: /(^[ \t]*)[A-Za-z_][\w-]*(?=[ \t]*\()/m,
        lookbehind: true,
      },
    ],
    keyword: keywords,
    boolean: /\b(?:true|false)\b/,
    builtin: builtins,
    variable,
    number: /\b\d+(?:\.\d+)+\b|\b\d+\b/,
    'attr-name': {
      pattern:
        /(^|[\s[(])--?(?!eq\b|ne\b|lt\b|le\b|gt\b|ge\b|nt\b|ot\b|ef\b)[A-Za-z][\w-]*/,
      lookbehind: true,
    },
    operator: [
      /&&|\|\|/,
      {
        pattern: /(^|[\s])(?:-eq|-ne|-lt|-le|-gt|-ge|-nt|-ot|-ef)\b/,
        lookbehind: true,
      },
      /=~|==|!=|<=|>=/,
      {
        pattern: /(^|[\s])[-+*/%](?=\s)/,
        lookbehind: true,
      },
      /=/,
      /\|/,
      /\.{3}/,
      /\*(?=[A-Za-z_])/,
    ],
    punctuation: /[()[\]{}.,;]/,
  };

  Prism.languages.scriptopotamus = Prism.languages.scrippo;
}

module.exports = registerScrippo;
