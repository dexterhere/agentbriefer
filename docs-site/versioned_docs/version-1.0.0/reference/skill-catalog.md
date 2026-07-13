---
title: Bundled skill catalog
description: Reference for the seven focused skills included with Agentbriefer v1.
---

# Bundled skill catalog

Agentbriefer v1 bundles seven skills inside the binary. They require no separate download and are
installed into a project by ID.

| Skill ID | Category | Use it when |
| --- | --- | --- |
| `docker-compose-for-local-dev` | Backend | Local infrastructure should stay minimal, configurable, healthy, and free of committed secrets. |
| `rust-axum-layered-structure` | Backend | An Axum service needs light handler, service, and data boundaries without premature abstractions. |
| `css-first-motion` | Frontend | UI animation should prefer CSS and respect reduced-motion preferences. |
| `server-components-by-default` | Frontend | A Next.js or React project should keep client components small and intentional. |
| `sveltekit-component-structure` | Frontend | A SvelteKit app needs thin routes and well-placed components, stores, and API helpers. |
| `no-secrets-in-repo` | Security | Every stack should keep keys, tokens, and credentials out of tracked files. |
| `password-and-session-auth` | Security | A Rust/Axum/PostgreSQL backend owns password, cookie, session, or token behavior. |

## Browse and inspect

```bash
agentbriefer skill list
agentbriefer skill list --recommended
agentbriefer skill info no-secrets-in-repo
```

`--role` and `--recommended` filter only the displayed catalog. They never install a skill or change
the shared project configuration.

## Choose a focused set

Skills add specialized guidance to every selected output, so install only rules that repeatedly apply
to the repository.

```bash
agentbriefer skill add rust-axum-layered-structure
agentbriefer skill add no-secrets-in-repo
```

The IDs are stored in `agentbriefer.yaml`, materialized under `.agentbriefer/skills/`, and inlined
into generated instructions. See [Skills](../guides/skills.md) for the full lifecycle.

