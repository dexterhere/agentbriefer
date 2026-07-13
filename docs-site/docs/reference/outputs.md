---
title: Outputs
description: Supported AI-agent instruction formats and how Agentbriefer writes them.
---

# Outputs

Agentbriefer turns one project profile into native instruction files for four AI coding ecosystems.
Select only the tools your project uses, or keep all outputs to offer contributors a consistent
experience.

| Configuration value | Path | Intended consumer |
| --- | --- | --- |
| `agents-md` | `AGENTS.md` | Agents that implement the AGENTS.md convention |
| `claude-md` | `CLAUDE.md` | Claude Code |
| `cursor-rules` | `.cursor/rules/agentbriefer.mdc` | Cursor |
| `copilot-instructions` | `.github/copilot-instructions.md` | GitHub Copilot |

## Generate or synchronize

`agentbriefer generate` renders a complete Agentbriefer-owned file. It is the clearest choice for a
new output or a file the team intends to manage entirely from configuration.

`agentbriefer sync` updates the marked Agentbriefer section and preserves content outside it. Use it
when a file also contains carefully maintained human instructions.

```bash
# First creation
agentbriefer generate

# Subsequent update that preserves surrounding content
agentbriefer sync
```

## Nested paths

Agentbriefer creates required parent directories for Cursor and GitHub output paths. If an output
path is a symbolic link, it resolves and writes to the final target rather than replacing the link.

## Reviewing changes

Treat generated instructions as source-controlled artifacts:

```bash
agentbriefer doctor
agentbriefer sync
git diff -- agentbriefer.yaml AGENTS.md CLAUDE.md .cursor .github
```

This makes changes visible to reviewers and keeps agent behavior reproducible across machines.

## Tool-specific additions

If a platform needs unique guidance, add it outside the managed markers and use `sync`. Put shared
policy in `agentbriefer.yaml`, a reusable profile, or a skill so it does not drift between outputs.
