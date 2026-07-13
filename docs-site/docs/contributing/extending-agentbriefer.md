---
title: Extending Agentbriefer
description: Add detectors, bundled skills, output formats, and configuration behavior safely.
---

# Extending Agentbriefer

Keep new functionality in the narrowest subsystem that owns it and add tests at the same boundary.

## Add a bundled skill

Create `skills/<category>/<id>/SKILL.md`. The directory name and frontmatter `id` must match.

```markdown title="skills/security/example-policy/SKILL.md"
---
id: example-policy
name: Example Policy
description: One sentence describing the behavior this skill enforces.
category: security
roles: [security, backend]
compatible_stacks: [rust, axum]
---
Write direct, actionable instructions for the coding agent here.
```

Use lowercase kebab-case IDs. `roles` are browsing tags. `compatible_stacks` participate in
case-insensitive any-tag recommendation matching; an empty list makes the skill stack-agnostic.

Then test the registry and update the public catalog:

```bash
cargo test skills
cargo test --test cli_integration
```

## Add or improve a detector

Implement an ecosystem module under `src/detect/`, expose it through `src/detect/mod.rs`, and preserve
the detector order intentionally because the first matching ecosystem wins. A detector should:

- return no match when its root manifest is absent;
- tolerate malformed input without preventing `init`;
- identify package manager, framework, database, tests, and key direct dependencies when evidence is
  reliable;
- keep dependency output deterministic and bounded.

Add focused unit fixtures and update the [detection matrix](../reference/detection-matrix.md).

## Add an output format

Add the `OutputFormat` enum value and path mapping, a top-level Tera template, prompt support, and
generate/sync/doctor coverage. Reuse shared template partials so behavioral policy does not drift
between tools. Document the consumer and output path.

## Change configuration

Schema changes affect YAML compatibility, templates, interactive prompts, profiles, snapshots, and
the documentation reference. For v1 maintenance releases, prefer optional fields with safe serde
defaults. Treat renamed or reinterpreted values as compatibility-sensitive changes.

