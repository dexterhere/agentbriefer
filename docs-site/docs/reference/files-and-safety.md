---
title: Files and safety
description: Files Agentbriefer reads and writes, preservation guarantees, and source-control guidance.
---

# Files and safety

Agentbriefer is local-first. It inspects project files, stores configuration and reusable data on your
machine, and writes the instruction outputs you select.

## Project files

| Path | Ownership |
| --- | --- |
| `agentbriefer.yaml` | Human-reviewed Agentbriefer configuration |
| `AGENTS.md` | Generated or synchronized output |
| `CLAUDE.md` | Generated or synchronized output |
| `.cursor/rules/agentbriefer.mdc` | Generated or synchronized output |
| `.github/copilot-instructions.md` | Generated or synchronized output |

Project manifests and lockfiles are read for stack detection. Agentbriefer does not modify them.

## User data

Installed skills and reusable developer or skill profiles live in the operating system's standard
per-user application data directory. The exact base directory follows the platform conventions
resolved for Agentbriefer, rather than assuming a hard-coded home path.

Back up custom skills and profiles if they are important organizational assets. Prefer keeping their
source in a version-controlled repository and installing from that source.

## Preservation rules

- `generate` owns the complete selected output.
- `sync` changes only Agentbriefer's marked section and retains surrounding content.
- Required parent directories are created when needed.
- Symbolic-link output paths are refused to avoid writing through an unexpected target.
- Invalid YAML, missing profiles, and unresolved skills are reported instead of silently rewriting
  intent.

An unknown skill is excluded from rendering and reported by validation. Run `doctor` before relying
on newly generated output.

## Source-control checklist

Commit `agentbriefer.yaml` and the generated files your team consumes. Do not commit private custom
skill content or profile data unless it is intentionally safe for every repository reader.

Before merging a policy change:

```bash
agentbriefer doctor
agentbriefer sync
git diff --check
```

Review the semantic change in the generated instructions—not just whether generation succeeded.
