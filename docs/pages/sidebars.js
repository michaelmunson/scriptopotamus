/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docs: [
    {
      type: 'category',
      label: 'Start',
      collapsed: false,
      items: [
        'index',
        'installation',
        'getting-started',
        'cli',
        'examples',
      ],
    },
    {
      type: 'category',
      label: 'Language',
      collapsed: false,
      items: [
        'syntax',
        'variables',
        'values',
        'control-flow',
        'functions',
        'commands',
        'parameters',
        'usage',
        'inherit',
        'macros',
        'hooks',
        'imports',
        'standard-library',
        'bash',
      ],
    },
    {
      type: 'category',
      label: 'scrippolib',
      collapsed: false,
      items: [
        'scrippolib',
        'installing',
        'registry',
        'publishing',
      ],
    },
    {
      type: 'category',
      label: 'Packages and scripts',
      collapsed: false,
      items: [
        'packages',
        'scripts',
        'metadata',
        'layout',
      ],
    },
    {
      type: 'category',
      label: 'Tooling',
      collapsed: false,
      items: [
        'configuration',
        'shell',
        'editors',
        'compilation',
      ],
    },
  ],
};

module.exports = sidebars;
