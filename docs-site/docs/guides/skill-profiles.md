---
title: Skill profiles
description: Save and apply reusable named sets of skill IDs.
---

# Skill profiles

A skill profile snapshots the current project's installed skill IDs so the same
set can be applied elsewhere.

## Create a profile

```bash
cd configured-project
agentbriefer skill profile create secure-rust-backend
```

The profile is stored under the platform configuration directory, commonly:

```text
~/.config/agentbriefer/skill-profiles/secure-rust-backend.yaml
```

## List profiles

```bash
agentbriefer skill profile list
```

## Apply a profile

```bash
cd another-configured-project
agentbriefer skill profile apply secure-rust-backend
```

Apply **replaces** the target project's entire `skills` list. Agentbriefer then
materializes the new set, removes stale skill directories, and synchronizes all
configured outputs.

:::caution

Apply is not additive. Review the profile and the target project's existing
skill list before applying it.

:::

Like developer profiles, skill profiles are copied snapshots rather than live
links. Changing one does not mutate projects that previously applied it.
