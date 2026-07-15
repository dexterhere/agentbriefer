---
title: CI integration
description: Fail your pipeline when generated instruction files drift from agentbriefer.yaml.
---

# CI integration

`agentbriefer doctor` exits non-zero when a configured output is missing or
out of sync with `agentbriefer.yaml`. That makes it a natural CI gate: add it
to your workflow and drift gets caught in review, not discovered later by an
agent acting on stale instructions.

## Add it to your workflow

```yaml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  doctor:
    name: agentbriefer doctor
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Run agentbriefer doctor
        uses: dexterhere/agentbriefer/.github/actions/doctor@v1.0.0
        with:
          version: '1.0.0'
          # fix: 'true'
```

`permissions: contents: read` is all this job needs — the action requires no
secrets and never writes back to your repository.

## What `--fix` does and doesn't do

Set `fix: 'true'` on the action (or run `agentbriefer doctor --fix` locally)
to have doctor resync any missing or stale output before it reports. It:

- Rewrites the configured output files (`CLAUDE.md`, `AGENTS.md`, Cursor
  rules, Copilot instructions) in the checkout, using the same logic as
  `agentbriefer sync`.
- Never touches `.agentbriefer/skills/` or your configured skill list.

It does **not**:

- Commit or push the fixed files back to your branch. In CI, `--fix` only
  changes files inside that run's ephemeral workspace — if the job succeeds
  because `--fix` resolved the drift, your repository itself is still out of
  sync until someone runs `agentbriefer sync` locally and commits the
  result. Treat a `--fix`-driven pass as a signal to sync locally, not as a
  substitute for it.

## FAQ

### My CI started failing after upgrading — why?

`agentbriefer doctor` used to always exit `0`, regardless of what it found.
As of the version noted in the [CHANGELOG](https://github.com/dexterhere/agentbriefer/blob/main/CHANGELOG.md#unreleased),
it exits non-zero when it finds drift — a configured output that's missing or
no longer matches a fresh render. This is an intentional breaking change:
doctor moved from an advisory local linter to a CI-enforceable check.

If you're not ready to make this blocking, pass `--no-fail` (or set
`no-fail: 'true'` if you're invoking the CLI directly rather than through the
composite action) to restore the previous always-succeed behavior while you
migrate. Otherwise, run `agentbriefer sync` locally, commit the result, and
the check will pass.
