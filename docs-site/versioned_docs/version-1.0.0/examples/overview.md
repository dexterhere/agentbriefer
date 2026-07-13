---
title: Example library
description: Choose a complete Agentbriefer workflow for your project or team.
---

# Use cases and examples

These examples start from a real goal and show the resulting configuration and
workflow—not just isolated command syntax.

| Goal | Example |
|---|---|
| Keep several coding assistants aligned | [Solo multi-agent workflow](./solo-multi-agent.md) |
| Establish strict team security boundaries | [Team security policy](./team-security.md) |
| Combine stack detection with focused skills | [Stack-specific skills](./stack-skills.md) |
| Reuse preferences and preserve manual notes | [Profiles and sync](./profiles-and-sync.md) |

## A useful baseline

Most projects can begin with:

```yaml
developer:
  style: practical
  explanation_style: short
project:
  project_type: library
  stack:
    language: rust
    package_manager: cargo
  security_level: standard
  testing_level: practical
  dependency_policy: explain-first
  architecture_style: simple
stop_rules: []
skills:
  - no-secrets-in-repo
```

Then change only settings that express a real project constraint. More strict is
not automatically more useful; the best brief is the one the team will follow
and review.
