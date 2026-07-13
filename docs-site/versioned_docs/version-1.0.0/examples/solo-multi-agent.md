---
title: Solo multi-agent workflow
description: Keep Claude Code, Cursor, Copilot, and AGENTS.md-compatible tools aligned.
---

# Solo developer, several agents

You use different AI tools for different tasks but want the same scope,
dependency, testing, and explanation expectations everywhere.

## Configure once

```bash
cd my-project
agentbriefer init
agentbriefer generate
```

Choose all four outputs and a practical, concise profile:

```yaml title="agentbriefer.yaml"
developer:
  style: practical
  explanation_style: short
project:
  project_type: full-stack-app
  stack:
    language: typescript
    framework: next
    testing_tools:
      - vitest
      - playwright
    package_manager: npm
    key_dependencies:
      - react
  security_level: standard
  testing_level: practical
  dependency_policy: explain-first
  architecture_style: feature-based
stop_rules:
  - Ask before changing a public API contract.
skills:
  - server-components-by-default
  - css-first-motion
outputs:
  - claude-md
  - agents-md
  - cursor-rules
  - copilot-instructions
```

## Result

- Claude Code reads `CLAUDE.md`.
- compatible agent runners discover `AGENTS.md`.
- Cursor loads its always-applied `.mdc` rule.
- GitHub Copilot reads repository-wide custom instructions.

All four receive the same decision loop, project policy, workflow loops, stop
rules, and installed skill bodies.

## Ongoing changes

Edit `agentbriefer.yaml`, then run:

```bash
agentbriefer sync
agentbriefer doctor
```

This keeps tool syntax separate while project intent stays unified.
