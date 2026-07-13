---
title: Command reference
description: Complete reference for every Agentbriefer CLI command and option.
---

# Command reference

Agentbriefer uses the following command shape:

```bash
agentbriefer <COMMAND>
```

Use `agentbriefer help`, `agentbriefer <command> --help`, or
`agentbriefer skill <command> --help` for the reference shipped with your installed version.

The root command supports `-h, --help` and `-V, --version`. Run project commands from the directory
that contains `agentbriefer.yaml`.

## Core commands

### `init`

Interactively inspect a project and create `agentbriefer.yaml`.

```bash
agentbriefer init
```

The wizard proposes values based on detected manifests and lets you choose generated outputs,
developer preferences, stop rules, and skills.

### `generate`

Render configured AI instruction files from `agentbriefer.yaml`.

```bash
agentbriefer generate
```

Generation replaces the selected output files. Use [`sync`](#sync) when an existing file contains
human-maintained content that must remain outside Agentbriefer's managed section.

### `sync`

Refresh only the Agentbriefer-managed section in each configured output.

```bash
agentbriefer sync
```

When a file has no managed section, Agentbriefer appends one without deleting the existing content.

### `doctor`

Validate configuration, skills, profiles, output paths, and generated content.

```bash
agentbriefer doctor
```

Run this after configuration changes and in pull-request checks. Errors require attention; warnings
usually identify stale or incomplete project context.

### `profile list`

List saved developer profiles.

```bash
agentbriefer profile list
```

### `profile create`

Interactively save a reusable developer style and explanation style.

```bash
agentbriefer profile create
```

### `profile switch`

Interactively choose a saved profile and copy it into the current project.

```bash
agentbriefer profile switch
```

Switch updates `agentbriefer.yaml` but does not regenerate outputs. Run `agentbriefer sync` next.

## Skill commands

### `skill list`

List bundled and locally installed skills.

```bash
agentbriefer skill list
```

### `skill add <ID>`

Install a bundled skill into the current project.

```bash
agentbriefer skill add no-secrets-in-repo
```

### `skill remove <NAME>`

Remove an installed skill from the current project.

```bash
agentbriefer skill remove no-secrets-in-repo
```

Bundled skills remain part of the Agentbriefer binary and can be installed again later.

### `skill update`

Re-materialize all installed skills from the catalog in the current binary and synchronize outputs.

```bash
agentbriefer skill update
```

### `skill info <NAME>`

Show a skill's metadata and source.

```bash
agentbriefer skill info no-secrets-in-repo
```

### `skill profile list`

List reusable skill profiles.

```bash
agentbriefer skill profile list
```

### `skill profile create <NAME>`

Create a named profile from selected skills.

```bash
agentbriefer skill profile create secure-api
```

### `skill profile apply <NAME>`

Apply a profile to the current project's `agentbriefer.yaml`.

```bash
agentbriefer skill profile apply secure-api
```

Review the changed configuration, then run `agentbriefer doctor` and `agentbriefer sync`.

## Exit behavior

Agentbriefer exits with a non-zero status when a command cannot complete or configuration cannot be
loaded. `doctor` findings are currently diagnostic warnings and do not make the command fail, so CI
must review its output explicitly.
