---
id: introduction
title: Introduction
sidebar_position: 1
description: What Agentbriefer is, what it solves, and when to use it.
---

# Meet Agentbriefer

Agentbriefer is a command-line tool that gives AI coding agents a consistent,
project-specific operating brief. You describe how work should be approached
once in `agentbriefer.yaml`; Agentbriefer turns that configuration into the
instruction files Claude Code, AGENTS.md-compatible tools, Cursor, and GitHub
Copilot already understand.

```text
agentbriefer.yaml
        ↓
Agentbriefer renderer + installed skills
        ↓
CLAUDE.md · AGENTS.md · Cursor rules · Copilot instructions
```

## The problem it solves

AI agents can be productive and still behave inconsistently. One session may
make a focused change; another may add dependencies, refactor unrelated code,
or continue beyond the requested scope. Repeating the same expectations in
every prompt is tedious, easy to forget, and difficult to share with a team.

Agentbriefer makes those expectations explicit and reviewable:

- how much architecture the agent should introduce;
- when dependencies require an explanation or approval;
- how security-sensitive changes should be treated;
- how much testing is expected;
- how detailed explanations should be;
- when the agent must stop and ask;
- which focused skills apply to the project.

## When Agentbriefer is useful

- **One developer, several tools:** keep Claude Code, Cursor, and Copilot aligned.
- **A shared repository:** review agent policy like any other project configuration.
- **Security-sensitive work:** make conservative auth, secret, permission, and validation behavior explicit.
- **A mixed-experience team:** choose learning-focused or concise explanations without rewriting prompts.
- **Repeated project setup:** reuse developer profiles and named skill sets.
- **Existing handwritten guidance:** use `sync` to preserve content outside Agentbriefer's managed block.

## What it is not

Agentbriefer does not run an autonomous agent, execute prompts, create pull
requests, store cloud memory, or replace your coding assistant. It prepares the
repository-level context those assistants consume.

:::tip[The shortest mental model 💡]

Agentbriefer is a compiler for agent instructions: YAML and bundled skills go
in; tool-specific Markdown and rules files come out.

:::

## Continue

Start with [Installation](./installation), follow the [Quick start](./quick-start),
or read [How it works](./how-it-works.md) for the full data flow.
