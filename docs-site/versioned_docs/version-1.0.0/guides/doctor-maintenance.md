---
title: Doctor and maintenance
description: Diagnose conflicting policy, missing output, drift, and catalog mismatches.
---

# Doctor and maintenance

Run the read-only project linter after configuration changes, Agentbriefer
upgrades, or unexpected generated output:

```bash
agentbriefer doctor
```

## What doctor checks

- `dependency_policy: allow` combined with `security_level: strict`;
- `testing_level: light` combined with `security_level: strict`;
- an empty primary language;
- configured skill IDs missing from the bundled catalog;
- configured output files that do not exist;
- generated files that differ from a fresh render.

For synced files, drift is measured only inside the managed block. Manual notes
outside it do not make the file stale.

## Findings and exit behavior

Doctor prints warnings for every finding and `No issues found.` when clean.
Findings currently do not produce a non-zero exit code.

```bash
agentbriefer doctor
# Use the report for visibility; do not assume warnings fail the process.
```

## Maintenance cycle

```bash
# Upgrade the CLI using your installation method.
npm install -g agentbriefer@latest

# Refresh bundled skill content and generated files.
agentbriefer skill update

# Confirm project state.
agentbriefer doctor
```

If an unknown skill ID appears after working with a teammate, upgrade first.
Remove the ID only when the project no longer intends to use that skill.
