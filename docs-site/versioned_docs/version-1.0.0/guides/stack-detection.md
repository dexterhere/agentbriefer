---
title: Stack detection
description: Understand detected languages, manifests, dependencies, limitations, and editable prefilled values.
---

# Stack detection

During `init`, Agentbriefer looks for supported manifest files directly in the
current directory. A successful detector can prefill:

- primary language;
- package manager;
- framework;
- database or data-access library;
- testing tools;
- up to 20 key direct dependencies;
- the manifest used as evidence.

The complete ecosystem list is in the [detection matrix](../reference/detection-matrix.md).

## Dependency categorization

Each detector has prioritized framework, testing, and database lookup lists.
Recognized packages are assigned to those fields; remaining direct dependencies
are alphabetized and capped at 20.

```text
axum + sqlx + insta + serde
  ↓
framework: axum
database: sqlx
testing_tools: [insta]
key_dependencies: [serde]
```

## Detection never blocks init

Malformed, absent, or unreadable manifests produce an empty detection result.
The wizard still runs and accepts manual answers.

## Polyglot repositories

Detectors run in a fixed order and the first matching ecosystem wins. Detection
is not recursive and does not merge multiple applications in a monorepo.

For a Rust API with a nested JavaScript frontend, running `init` at the repository
root may detect only the root manifest. Correct the prefilled values or maintain
separate configurations at the appropriate project roots.

:::tip[Detection is a starting point 🧭]

Press Enter to accept an accurate prefill. Edit it whenever the manifest does
not reflect the context you want agents to prioritize.

:::

## Detection and skill recommendations

`agentbriefer skill list --recommended` runs detection for the current session
and shows skills whose compatible-stack tags intersect the detected tags.
A stack-agnostic skill is always recommended.

Matching is an **any-tag** intersection. A skill tagged `rust`, `axum`, and
`postgres` may be recommended by a Rust project even when Axum and PostgreSQL
were not detected. Review the skill with `skill info` before adding it.
