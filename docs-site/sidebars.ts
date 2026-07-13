import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  userSidebar: [
    {
      type: 'category',
      label: 'Start here',
      collapsed: false,
      items: ['introduction', 'how-it-works', 'installation', 'quick-start'],
    },
    {
      type: 'category',
      label: 'Core guides',
      items: [
        'guides/configure-project',
        'guides/generate-vs-sync',
        'guides/stack-detection',
        'guides/developer-profiles',
        'guides/skills',
        'guides/skill-profiles',
        'guides/doctor-maintenance',
        'guides/team-workflow',
      ],
    },
    {
      type: 'category',
      label: 'Use cases & examples',
      items: ['examples/overview', 'examples/solo-multi-agent', 'examples/team-security', 'examples/stack-skills', 'examples/profiles-and-sync'],
    },
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/commands',
        'reference/configuration',
        'reference/outputs',
        'reference/detection-matrix',
        'reference/skill-catalog',
        'reference/files-and-safety',
      ],
    },
    'troubleshooting',
    {
      type: 'category',
      label: 'Contributing',
      items: [
        'contributing/architecture',
        'contributing/development',
        'contributing/extending-agentbriefer',
        'contributing/testing-and-release',
        'contributing/documentation',
      ],
    },
    'release-notes/v1',
  ],
};

export default sidebars;
