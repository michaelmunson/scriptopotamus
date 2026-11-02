function siteUrl(value) {
  return (value || 'https://michaelmunson.github.io').replace(/\/$/, '');
}

function siteBaseUrl(value) {
  if (!value || value === '/') {
    return '/';
  }
  const withLeading = value.startsWith('/') ? value : `/${value}`;
  const withTrailing = withLeading.endsWith('/') ? withLeading : `${withLeading}/`;
  return withTrailing.replace(/\/{2,}/g, '/');
}

const url = siteUrl(process.env.DOCUSAURUS_URL);
const baseUrl = siteBaseUrl(process.env.DOCUSAURUS_BASE_URL);

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Scriptopotamus',
  tagline: 'scrippo compiles .scrippo to bash',
  favicon: 'img/favicon.svg',
  url,
  baseUrl,
  organizationName: 'michaelmunson',
  projectName: 'scriptopotamus',
  trailingSlash: true,
  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',
  markdown: {
    format: 'detect',
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          routeBasePath: '/',
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/michaelmunson/scriptopotamus/tree/main/docs/pages/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],
  themes: [
    [
      require.resolve('@easyops-cn/docusaurus-search-local'),
      {
        hashed: true,
        language: ['en'],
        indexDocs: true,
        indexBlog: false,
        indexPages: false,
        docsRouteBasePath: '/',
      },
    ],
  ],
  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        respectPrefersColorScheme: true,
      },
      navbar: {
        title: 'Scriptopotamus',
        logo: {
          alt: '',
          src: 'img/favicon.svg',
        },
        items: [
          {
            href: 'https://github.com/michaelmunson/scriptopotamus',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      prism: {
        additionalLanguages: ['lua'],
      },
      docs: {
        sidebar: {
          autoCollapseCategories: false,
        },
      },
    }),
};

module.exports = config;
