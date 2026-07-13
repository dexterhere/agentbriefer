---
title: Configuration schema
description: Reference for agentbriefer.yaml and every supported configuration value.
---

# Configuration schema

`agentbriefer.yaml` is the source of truth for a project's generated instructions. Commit it so the
team reviews changes to agent behavior like any other development policy.

## Complete example

```yaml title="agentbriefer.yaml"
developer:
  style: practical
  explanation_style: tradeoff-based

project:
  project_type: backend-api
  stack:
    language: rust
    framework: axum
    database: postgresql
    testing_tools:
      - cargo test
    package_manager: cargo
    key_dependencies:
      - tokio
      - serde
  architecture_style: simple-layered
  security_level: strict
  testing_level: practical
  dependency_policy: explain-first

stop_rules:
  - Stop before changing the public API.
  - Stop before adding a production dependency.

skills:
  - secure-api
  - testing

custom_instructions: |
  Prefer small, reviewable patches.
  Keep database migrations backward compatible.

outputs:
  - agents-md
  - claude-md
  - cursor-rules
  - copilot-instructions
```

## Top-level fields

| Field | Required | Description |
| --- | --- | --- |
| `extends` | No | Reusable developer profile to inherit before applying project values. |
| `developer` | Yes | Preferred working and explanation styles. |
| `project` | Yes | Project type and detected or curated stack context. |
| `stop_rules` | No | Explicit conditions that require the agent to pause. Defaults to `[]`. |
| `skills` | No | Ordered names of bundled or installed skills. Defaults to `[]`. |
| `custom_instructions` | No | Project-specific guidance that does not fit another field. |
| `outputs` | No | Files to generate. Defaults to all supported outputs. |

Use only the documented field names. YAML syntax and invalid enum values are rejected; review the
generated result because unrecognized extra mapping fields are not part of the supported schema.

## Developer preferences

### `developer.style`

Choose one of:

- `minimal`
- `practical`
- `learning-focused`
- `enterprise`
- `security-first`

### `developer.explanation_style`

Choose one of:

- `short`
- `beginner-friendly`
- `detailed`
- `tradeoff-based`

## Project values

### `project.project_type`

Choose one of:

- `backend-api`
- `frontend-app`
- `cli-tool`
- `full-stack-app`
- `library`
- `documentation-heavy`

### `project.stack`

`language` is required. `framework`, `database`, `package_manager`, and `key_dependencies` are
optional context; `testing_tools` and `key_dependencies` default to empty lists.

Detection is a starting point, not a constraint. Keep product-specific details that a manifest
cannot infer, and remove incidental dependencies that should not influence agent decisions.

## Policy values

| Field | Supported values |
| --- | --- |
| `project.architecture_style` | `simple`, `simple-layered`, `feature-based`, `advanced` |
| `project.security_level` | `basic`, `standard`, `strict` |
| `project.testing_level` | `light`, `practical`, `strict` |
| `project.dependency_policy` | `allow`, `explain-first`, `ask-first` |

## Output values

| Value | Generated path |
| --- | --- |
| `agents-md` | `AGENTS.md` |
| `claude-md` | `CLAUDE.md` |
| `cursor-rules` | `.cursor/rules/agentbriefer.mdc` |
| `copilot-instructions` | `.github/copilot-instructions.md` |

See [Outputs](./outputs.md) for platform behavior and safe synchronization.

## Precedence

`extends` records which saved developer profile was copied into the project. It is informational:
generation does not resolve or merge the saved profile again. The concrete `developer` values in the
project are always authoritative. Skills are resolved in their configured order at render time.

After any manual edit, run:

```bash
agentbriefer doctor
agentbriefer sync
```
