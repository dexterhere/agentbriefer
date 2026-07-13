---
title: Stack-specific skills
description: Use detected stack tags to discover relevant skills for Rust, Next.js, and SvelteKit.
---

# Stack detection and skills

Recommendation narrows the bundled catalog; it does not install anything.

```bash
agentbriefer skill list --recommended
agentbriefer skill info <id>
agentbriefer skill add <id>
```

## Rust and Axum

Given a `Cargo.toml` containing Axum and SQLx, review:

```bash
agentbriefer skill info rust-axum-layered-structure
agentbriefer skill info password-and-session-auth
agentbriefer skill add rust-axum-layered-structure
```

Add the authentication skill only when the project actually owns password and
session behavior. Recommendations use any matching tag, so human review matters.

## Next.js and React

```bash
agentbriefer skill list --role frontend --recommended
agentbriefer skill add server-components-by-default
agentbriefer skill add css-first-motion
```

This keeps client boundaries intentional and prefers CSS motion with reduced
motion support.

## SvelteKit

```bash
agentbriefer skill info sveltekit-component-structure
agentbriefer skill add sveltekit-component-structure
```

The skill keeps reusable components and stores under `src/lib` while route files
remain focused on routing and page composition.

## Stack-agnostic security

`no-secrets-in-repo` has no compatible-stack restriction and appears in every
recommendation set:

```bash
agentbriefer skill add no-secrets-in-repo
```
