import clsx from 'clsx';
import Heading from '@theme/Heading';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

const features = [
  {
    title: 'One config, every agent',
    description: 'Generate Claude, AGENTS.md, Cursor, and Copilot instructions from one reviewable project policy.',
    to: '/docs/how-it-works',
  },
  {
    title: 'Stack-aware setup',
    description: 'Detect language, framework, database, test tooling, package manager, and key dependencies across 28 ecosystems.',
    to: '/docs/guides/stack-detection',
  },
  {
    title: 'Reusable skills',
    description: 'Install focused practices for architecture, frontend structure, security, authentication, and local infrastructure.',
    to: '/docs/guides/skills',
  },
  {
    title: 'Safe synchronization',
    description: 'Regenerate managed guidance while preserving the notes your team keeps outside Agentbriefer markers.',
    to: '/docs/guides/generate-vs-sync',
  },
  {
    title: 'Personal profiles',
    description: 'Carry your preferred development and explanation style from project to project without hiding team policy.',
    to: '/docs/guides/developer-profiles',
  },
  {
    title: 'Built-in diagnosis',
    description: 'Find inconsistent policies, missing outputs, stale files, and unavailable skill IDs before they surprise the team.',
    to: '/docs/guides/doctor-maintenance',
  },
];

export default function Home() {
  return (
    <Layout title="Documentation" description="Learn how to configure consistent AI coding-agent behavior with Agentbriefer.">
      <main>
        <header className="hero--agentbriefer">
          <div className="container">
            <p className="hero__eyebrow">Agentbriefer CLI · v1.0</p>
            <Heading as="h1" className="hero__title">
              Give every coding agent the same project brief.
            </Heading>
            <p className="hero__subtitle">
              Configure how AI agents think, code, test, and stop—once. Agentbriefer turns that policy into the instruction files your tools already read.
            </p>
            <div className="hero__command" aria-label="Recommended installation command">
              npm install -g agentbriefer
            </div>
            <div className="hero__actions">
              <Link className={clsx('button', 'button--primary', 'button--lg')} to="/docs/quick-start">
                Get started
              </Link>
              <Link className={clsx('button', 'button--secondary', 'button--lg')} href="https://github.com/dexterhere/agentbriefer">
                View on GitHub
              </Link>
            </div>
          </div>
        </header>
        <section className="feature-grid" aria-label="Agentbriefer capabilities">
          {features.map((feature) => (
            <Link className="feature-card" to={feature.to} key={feature.title}>
              <Heading as="h2">{feature.title}</Heading>
              <p>{feature.description}</p>
            </Link>
          ))}
        </section>
      </main>
    </Layout>
  );
}
