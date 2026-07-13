---
title: Profiles and sync
description: Reuse personal preferences and skill sets while preserving handwritten project notes.
---

# Profiles, skill sets, and manual notes

This workflow separates three concerns:

- personal communication preferences in a developer profile;
- reusable technical guidance in a skill profile;
- repository-specific notes outside a managed output block.

## Save personal preferences

```bash
agentbriefer profile create
# Name: practical-short
# Style: practical
# Explanation style: short
```

Future `init` runs can copy that profile without hiding the values from project
YAML.

## Save a skill set

After configuring a representative frontend project:

```bash
agentbriefer skill add server-components-by-default
agentbriefer skill add css-first-motion
agentbriefer skill add no-secrets-in-repo
agentbriefer skill profile create frontend-basics
```

Apply it to another configured project:

```bash
agentbriefer skill profile apply frontend-basics
```

Remember that apply replaces the complete skills list.

## Preserve handwritten instructions

Run sync once, then add notes outside the block:

```markdown title="AGENTS.md"
# Repository note

The `fixtures/legacy` directory is an external compatibility corpus. Do not
reformat it.

<!-- agentbriefer:managed:start -->
Generated project guidance...
<!-- agentbriefer:managed:end -->
```

Now configuration changes are safe to merge:

```bash
agentbriefer sync
agentbriefer doctor
```

The compatibility note survives while the managed section reflects current
policy and skills.
