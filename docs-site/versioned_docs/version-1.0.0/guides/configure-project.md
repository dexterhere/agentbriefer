---
title: Configure a project
description: Use init, detected stack values, project policies, custom instructions, and stop rules.
---

# Configure a project

`agentbriefer init` is the only interactive project command. It turns a guided
conversation into a typed YAML configuration.

## Before the prompts

Run it from the project root:

```bash
agentbriefer init
```

If `agentbriefer.yaml` already exists, the wizard asks before overwriting it and
defaults to leaving it untouched. Agentbriefer then detects a supported manifest
and loads any saved developer profiles.

## Stack answers

Detected values prefill language, framework, database, package manager, testing
tools, and key dependencies. They remain editable because repositories can be
polyglot or use a tool the detector does not recognize.

Comma-separated testing tools and dependencies are normalized into YAML lists.
Optional fields can be left blank.

## Behavioral policy

The structured settings are deliberately concrete:

| Setting | What it changes |
|---|---|
| Developer style | Scope, tradeoffs, teaching, maintainability, or security posture |
| Explanation style | Short, beginner-friendly, detailed, or tradeoff-focused reporting |
| Project type | Backend, frontend, CLI, full-stack, library, or docs priorities |
| Security level | Caution around auth, secrets, validation, permissions, and data access |
| Testing level | When tests are optional, practical, or required |
| Dependency policy | Whether agents may add, must explain, or must ask first |
| Architecture style | Flat, lightly layered, feature-based, or advanced existing patterns |

Every value is rendered with an agent-facing description; generated files do
not rely on an unexplained label such as `practical` or `strict`.

## Custom instructions

Custom instructions hold project-specific guidance that does not fit a dropdown:

```text
Run `make fixtures` before integration tests. Never edit generated files under
src/protocol/. Keep public error codes backward compatible.
```

The wizard opens `$VISUAL` or `$EDITOR` when available, falling back to the
platform editor. Closing without saving keeps the previous value.

## Stop rules

Stop rules describe boundaries where an agent must pause or avoid continuing:

```yaml
stop_rules:
  - Stop before changing CI/CD configuration.
  - Ask before modifying a public API.
  - Do not access production credentials.
```

When no stop rules are configured, generated output still tells the agent to
stop after the requested change is complete and verified.

## Review and save

The final screen shows every answer. Select a field to edit it using its current
value as the default, or select **Save and finish**. The wizard writes YAML only
after this review.

:::note

`init` starts with no installed skills. Add them after saving with
`agentbriefer skill list` and `agentbriefer skill add <id>`.

:::
