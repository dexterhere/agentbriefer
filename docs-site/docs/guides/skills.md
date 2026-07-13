---
title: Skills
description: Discover, inspect, install, update, and remove focused Agentbriefer skills.
---

# Skills

Skills add focused practices that are more specific than Agentbriefer's general
project policy. They ship inside the CLI as validated `SKILL.md` documents.

## Browse the catalog

```bash
agentbriefer skill list
agentbriefer skill list --role frontend
agentbriefer skill list --recommended
agentbriefer skill list --role security --recommended
```

Role and recommendation filters affect only the current listing. They never
change the project's installed skills.

## Inspect before installing

```bash
agentbriefer skill info no-secrets-in-repo
```

Info prints the name, ID, description, category, roles, compatible-stack tags,
and full instruction body.

## Add a skill

```bash
agentbriefer skill add no-secrets-in-repo
```

This operation:

1. validates the ID against the bundled catalog;
2. appends the ID to `agentbriefer.yaml`;
3. materializes `.agentbriefer/skills/<id>/SKILL.md`;
4. synchronizes every configured agent output.

Adding an installed skill again is a harmless no-op. Unknown IDs fail with a
catalog hint.

## Update installed skill content

```bash
agentbriefer skill update
```

Update re-materializes configured skills and synchronizes outputs using the
catalog bundled in the currently installed CLI. It does not fetch a marketplace
or contact a server; upgrade Agentbriefer first to receive newer bundled content.

## Remove a skill

```bash
agentbriefer skill remove no-secrets-in-repo
```

Removal updates YAML, deletes stale materialized skill directories, and
synchronizes outputs. Removing an absent ID warns and succeeds.

## Source of truth

The IDs in `agentbriefer.yaml` are authoritative. Materialized `SKILL.md` files
contain a do-not-edit header and are replaced during update. Put project-specific
exceptions in `custom_instructions`, not inside generated skill files.

See the [v1 skill catalog](../reference/skill-catalog.md) for every bundled ID.
