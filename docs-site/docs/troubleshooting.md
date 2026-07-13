---
title: Troubleshooting
description: Diagnose installation, configuration, generation, synchronization, and skill issues.
---

# Troubleshooting

Start with the installed help and the read-only project diagnostic:

```bash
agentbriefer --version
agentbriefer --help
agentbriefer doctor
```

## Command not found after installation

Open a new terminal, then check the binary directory for your installation method.

```bash
npm prefix -g
echo "$HOME/.cargo/bin"
```

Add the applicable binary directory to `PATH`. On Windows, reopen PowerShell after the installer
updates the environment.

## No `agentbriefer.yaml` found

Project commands resolve the current working directory. Change to the configured repository root or
initialize the current directory:

```bash
cd path/to/project
agentbriefer init
```

## YAML cannot be parsed

Compare the file with the [configuration schema](./reference/configuration.md). Common causes are
incorrect indentation, a policy field placed outside `project`, or a list written as a scalar.

If the file is disposable, move it aside and rerun `init`; otherwise fix the YAML and run `doctor`.

## Generated output is stale

Refresh configured files and check again:

```bash
agentbriefer sync
agentbriefer doctor
```

If a file has no valid managed block, sync warns and replaces its contents. Back up valuable manual
content first, establish the managed block, then restore the manual content outside the markers.

## Manual output edits disappeared

`generate` replaces a complete output. Only `sync` preserves text outside managed markers. Recover
the previous file from source control, run `sync` to add a managed block, and keep restored notes
outside it.

## A symlink output is refused

Agentbriefer will not write through an output path whose final file is a symbolic link. Replace it
with a regular file in the project, then rerun the command. This boundary prevents an instruction
path from redirecting writes somewhere unexpected.

## Skill is unknown

The project may reference an ID that is not bundled with the installed CLI version.

```bash
agentbriefer skill list
agentbriefer --version
```

Upgrade Agentbriefer if a teammate used a newer catalog. Otherwise correct or remove the invalid ID
in `agentbriefer.yaml`, then run `agentbriefer skill update`.

## Recommended skills look unrelated

Recommendations match when any compatible stack tag intersects detected language, framework,
database, package manager, test tool, or dependency tags. Inspect before installing:

```bash
agentbriefer skill list --recommended
agentbriefer skill info <id>
```

## Doctor reports warnings but exits successfully

That is the v1 behavior: findings are diagnostic and do not set a failing exit code. Review the
output explicitly in CI rather than treating process success as a clean report.

## Still blocked

Collect the output of `agentbriefer --version`, the relevant command, and a redacted configuration,
then [open a GitHub issue](https://github.com/dexterhere/agentbriefer/issues). Never include secrets
or private skill content in a public report.

