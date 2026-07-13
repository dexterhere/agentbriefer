---
title: Documentation workflow
description: Author, version, validate, and deploy the Agentbriefer documentation.
---

# Documentation workflow

The Docusaurus site lives in `docs-site/` within the CLI repository so documentation can be reviewed
and versioned with the behavior it describes.

## Writing standards

- Lead with the user's goal and show commands that can be copied safely.
- Verify names, flags, paths, defaults, and exit behavior against the Rust source or `--help`.
- Use emoji sparingly as wayfinding, not as a replacement for labels. ✨
- Include realistic snippets without credentials, tokens, or private repository details.
- Link to the canonical reference instead of duplicating long schemas.
- Update both user and contributor material when a change affects both audiences.

## Local workflow

```bash
cd docs-site
npm install
npm start
```

Edit current documentation in `docs/`. Docusaurus reloads pages during development. Before commit:

```bash
npm run typecheck
npm run build
```

The build is configured to fail on broken links and anchors.

## Create a release snapshot

With the current docs describing the release:

```bash
npm run docusaurus docs:version 1.0.0
```

Then configure `1.0.0` as the latest stable version and current docs as **Next**. Keep release notes
inside the snapshot and continue future work only in `docs/`.

## Deployment

Vercel can deploy this static site from the monorepo by setting the project root to `docs-site`.
The repository includes `docs-site/vercel.json` and a maintainer runbook in `docs-site/DEPLOYMENT.md`.
Deploy previews should build every pull request; production should track the chosen release branch.

