---
id: quick-start
title: Quick start
sidebar_position: 4
description: Configure a project and generate AI-agent instructions in minutes.
---

# Quick start

This walkthrough configures one project, generates all four supported output
formats, adds a security skill, and verifies the result.

## 1. Install and enter your project

```bash
npm install -g agentbriefer
cd path/to/your-project
```

## 2. Run the setup wizard

```bash
agentbriefer init
```

When Agentbriefer recognizes a project manifest, it announces the detected
language and pre-fills stack questions. Press Enter to accept a value or edit it.

The wizard covers:

1. developer and explanation styles;
2. language, framework, database, package manager, tests, and dependencies;
3. project type and architecture expectations;
4. security, testing, and dependency policies;
5. custom instructions and stop rules;
6. output formats;
7. a review screen where every answer remains editable.

It writes `agentbriefer.yaml` in the current directory.

## 3. Generate instruction files

```bash
agentbriefer generate
```

With the default output selection, the project now contains:

```text
your-project/
├── agentbriefer.yaml
├── CLAUDE.md
├── AGENTS.md
├── .cursor/rules/agentbriefer.mdc
└── .github/copilot-instructions.md
```

## 4. Add a focused skill

```bash
agentbriefer skill list --recommended
agentbriefer skill add no-secrets-in-repo
```

Adding a skill updates `agentbriefer.yaml`, creates
`.agentbriefer/skills/no-secrets-in-repo/SKILL.md`, and synchronizes all
configured output files.

## 5. Verify the project

```bash
agentbriefer doctor
```

A clean project prints `No issues found.` Doctor reports warnings rather than
returning a failing status code.

## 6. Choose the right regeneration command

- Continue using `generate` if the outputs are entirely machine-owned.
- Switch to `sync` before adding manual notes outside Agentbriefer's managed
  markers.

```bash
agentbriefer sync
```

:::caution[Generate overwrites the whole file]

After you add manual content around a managed block, use `sync`. A later
`generate` does not preserve those additions.

:::

## Next steps

- Understand [generate versus sync](./guides/generate-vs-sync.md).
- Browse the [skill guide](./guides/skills.md).
- Review a complete [configuration reference](./reference/configuration.md).
- Explore real [use cases and examples](./examples/overview.md).
