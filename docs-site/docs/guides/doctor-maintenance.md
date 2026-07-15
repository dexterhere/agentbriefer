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

By default, `agentbriefer doctor` exits non-zero when it reports **drift** —
a configured output that hasn't been generated yet, or a generated file that
no longer matches a fresh render. Conflicting settings, missing fields, and
unknown skill IDs are reported but do not affect the exit code on their own.

```bash
agentbriefer doctor
# Exits non-zero if any output is missing or stale. Safe to use as a CI gate.
```

Two flags change this behavior:

- `--fix` resyncs any missing or stale output (the same write path as
  `agentbriefer sync`) before doctor reports, so a single run can repair and
  then report a clean result.
- `--no-fail` always exits `0`, regardless of findings — use it if you want
  doctor's report without making it a blocking check.

`--json` emits a machine-readable report instead of the human-readable
listing, for scripting or CI summaries:

```bash
agentbriefer doctor --json
```

See [CI integration](./ci-integration.md) for wiring doctor into a pipeline
as a blocking check.

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
