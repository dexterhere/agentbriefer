import type {Config} from '@docusaurus/types';
import type {Options, ThemeConfig} from '@docusaurus/preset-classic';
import {themes as prismThemes} from 'prism-react-renderer';

const config: Config = {
  title: 'Agentbriefer',
  tagline: 'Configure how AI coding agents think, code, test, and stop.',
  favicon: 'img/agentbriefer-icon.svg',
  url: 'https://docsagentbriefer.vercel.app',
  baseUrl: '/',
  organizationName: 'dexterhere',
  projectName: 'agentbriefer',
  trailingSlash: false,
  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',
  onDuplicateRoutes: 'throw',
  markdown: {
    mermaid: true,
    format: 'detect',
  },
  themes: ['@docusaurus/theme-mermaid'],
  plugins: [
    [
      '@easyops-cn/docusaurus-search-local',
      {
        hashed: true,
        indexDocs: true,
        indexBlog: false,
        docsRouteBasePath: '/docs',
        language: ['en'],
        highlightSearchTermsOnTargetPage: true,
        searchResultLimits: 8,
        searchResultContextMaxLength: 60,
      },
    ],
    [
      '@signalwire/docusaurus-plugin-llms-txt',
      {
        siteTitle: 'Agentbriefer documentation',
        siteDescription:
          'Versioned documentation for configuring AI coding-agent behavior with Agentbriefer.',
        depth: 2,
        onRouteError: 'throw',
        content: {
          enableLlmsFullTxt: true,
          includeDocs: true,
          includeVersionedDocs: true,
          includePages: false,
          includeBlog: false,
        },
      },
    ],
  ],
  presets: [
    [
      'classic',
      {
        docs: {
          routeBasePath: 'docs',
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/dexterhere/agentbriefer/edit/main/docs-site/',
          showLastUpdateAuthor: true,
          showLastUpdateTime: true,
          lastVersion: '1.0.0',
          versions: {
            current: {
              label: 'Next',
              path: 'next',
              banner: 'unreleased',
              badge: false,
            },
            '1.0.0': {
              label: 'v1.0',
              path: '',
              banner: 'none',
              badge: true,
            },
          },
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
        sitemap: {
          changefreq: 'weekly',
          priority: 0.5,
          ignorePatterns: ['/docs/next/**'],
        },
      } satisfies Options,
    ],
  ],
  themeConfig: {
    image: 'img/agentbriefer-social-card.svg',
    metadata: [
      {
        name: 'description',
        content:
          'Documentation for Agentbriefer, the CLI that creates consistent project instructions for AI coding agents.',
      },
    ],
    colorMode: {
      defaultMode: 'light',
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'agentbriefer',
      logo: {
        alt: 'Agentbriefer logo',
        src: 'img/agentbriefer-icon.svg',
      },
      items: [
        {type: 'docSidebar', sidebarId: 'userSidebar', position: 'left', label: 'Docs'},
        {to: '/docs/examples/overview', label: 'Examples', position: 'left'},
        {to: '/docs/reference/commands', label: 'Reference', position: 'left'},
        {
          type: 'docsVersionDropdown',
          position: 'right',
          dropdownActiveClassDisabled: true,
        },
        {
          href: 'https://github.com/dexterhere/agentbriefer',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            {label: 'Get started', to: '/docs/quick-start'},
            {label: 'Installation', to: '/docs/installation'},
            {label: 'Command reference', to: '/docs/reference/commands'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'GitHub', href: 'https://github.com/dexterhere/agentbriefer'},
            {label: 'npm', href: 'https://www.npmjs.com/package/agentbriefer'},
            {label: 'Security', href: 'https://github.com/dexterhere/agentbriefer/security/policy'},
          ],
        },
        {
          title: 'AI access',
          items: [
            {label: 'llms.txt', href: 'https://docsagentbriefer.vercel.app/llms.txt'},
            {label: 'llms-full.txt', href: 'https://docsagentbriefer.vercel.app/llms-full.txt'},
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Agentbriefer. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['bash', 'powershell', 'toml', 'yaml', 'rust'],
    },
  } satisfies ThemeConfig,
};

export default config;
