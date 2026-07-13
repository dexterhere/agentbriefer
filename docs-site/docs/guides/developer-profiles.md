---
title: Developer profiles
description: Reuse personal development and explanation styles across projects.
---

# Developer profiles

A developer profile stores the two preferences that usually follow a person
rather than a repository:

```yaml
style: practical
explanation_style: tradeoff-based
```

## Create a profile

```bash
agentbriefer profile create
```

Choose a simple name, developer style, and explanation style. Existing names
require overwrite confirmation.

Profiles are stored in the platform configuration directory, commonly:

```text
~/.config/agentbriefer/profiles/<name>.yaml
```

## List profiles

```bash
agentbriefer profile list
```

## Use a profile during init

When profiles exist, `agentbriefer init` offers them before asking the two
developer questions. The selected values are copied into the project YAML.

## Switch an existing project

```bash
cd your-project
agentbriefer profile switch
agentbriefer sync
```

Switch replaces the project's `developer` section and records the source profile
name in `extends`. It does not automatically regenerate output files, so run
`sync` afterward.

## Copy semantics

Profiles are snapshots, not live inheritance. Updating a saved profile does not
retroactively change projects that used it. This keeps `agentbriefer.yaml`
self-contained and reviewable by teammates who do not have your profile.
