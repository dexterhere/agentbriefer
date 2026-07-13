---
title: Team security policy
description: Make authentication, secrets, dependencies, and test expectations explicit.
---

# Security-first backend team

The team maintains a backend with authentication and wants agents to treat
security-relevant changes conservatively.

```yaml title="agentbriefer.yaml"
developer:
  style: security-first
  explanation_style: tradeoff-based
project:
  project_type: backend-api
  stack:
    language: rust
    framework: axum
    database: sqlx
    testing_tools:
      - cargo-test
    package_manager: cargo
    key_dependencies:
      - argon2
      - tower-http
  security_level: strict
  testing_level: strict
  dependency_policy: ask-first
  architecture_style: simple-layered
stop_rules:
  - Stop before changing session lifetime or cookie policy.
  - Ask before adding a new authentication dependency.
  - Never weaken validation to make a test pass.
skills:
  - rust-axum-layered-structure
  - password-and-session-auth
  - no-secrets-in-repo
outputs:
  - claude-md
  - agents-md
```

## Apply and verify

```bash
agentbriefer skill add rust-axum-layered-structure
agentbriefer skill add password-and-session-auth
agentbriefer skill add no-secrets-in-repo
agentbriefer doctor
```

Strict security with `ask-first` dependencies and strict testing is internally
consistent. Doctor would warn if the same project allowed dependencies freely or
used light testing.

## Team review points

- Commit policy changes with the related architecture decision.
- Review skill additions as executable guidance, not decorative metadata.
- Keep secrets and environment-specific values out of YAML and generated files.
- Use stop rules for decisions requiring human ownership, not for ordinary work.
