---
title: Team workflow
description: Share project policy while keeping personal filters and profiles separate.
---

# Team workflow

Agentbriefer separates shared repository policy from personal conveniences.

## Commit shared state

Commit `agentbriefer.yaml`. It contains the complete developer/project policy,
output selection, stop rules, custom instructions, and installed skill IDs.

Teams may also commit generated instruction files so agents receive guidance
immediately after checkout. If the team chooses not to commit them, document
`agentbriefer sync` as a setup step.

Materialized `.agentbriefer/skills/` files are reproducible from the config and
CLI catalog. Decide as a team whether readable copies should be committed.

## Keep personal state personal

Developer profiles and skill profiles live in the user's configuration directory
and are not part of the repository. Applying either writes explicit values back
to project YAML, keeping the shared result visible.

`skill list --role` and `--recommended` are browsing filters only. Two teammates
can use different filters without changing installed project state.

## Suggested review process

When `agentbriefer.yaml` changes, review it like application configuration:

- Does a policy conflict with the project's risk level?
- Is a new stop rule precise enough to act on?
- Does an installed skill fit the actual stack?
- Will changing outputs add or remove tool-specific files?
- Were generated files synchronized in the same change?

## Example pull-request check

```bash
cargo test --all-targets       # project-specific checks, when applicable
agentbriefer sync
agentbriefer doctor
git diff -- agentbriefer.yaml CLAUDE.md AGENTS.md .cursor .github
```

`agentbriefer doctor` exits non-zero when it finds drift (a missing or stale
generated output), so this check fails the process on its own — you don't
need to review its output manually to catch drift. Conflicting-settings,
missing-field, and unknown-skill findings are still warnings that won't fail
the command, so it's worth skimming the output for those.

See [CI integration](./ci-integration.md) for running this same check as a
required GitHub Actions job instead of a local pre-PR step.
