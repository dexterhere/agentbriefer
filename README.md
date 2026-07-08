# SkillForge CLI

Configure how AI coding agents think, code, test, and stop inside your project.

SkillForge CLI is a command-line tool for configuring AI coding-agent behavior
inside software projects. Instead of repeatedly telling an AI assistant how to
work, you configure the rules once. SkillForge then generates the instruction
files different AI coding tools understand — `CLAUDE.md`, `AGENTS.md`, Cursor
rules, and Copilot instructions — from a single source of truth.

This is a personal tool first: the goal is to make day-to-day agentic
development easier and more consistent, not to chase every AI tool that
exists.

## Core idea

SkillForge is not only about telling AI what to write. It's also about
teaching it when *not* to write code, when to reuse existing code, when to
ask before a risky change, and when to stop once the requested work is done.

### The problem

AI coding agents behave inconsistently — clean and minimal one day,
over-engineered the next. Common failure modes: writing too much code,
touching unrelated files, adding unnecessary dependencies, ignoring existing
architecture, removing validation/error handling, and continuing past the
requested scope.

### The philosophy — adaptive decision loop

Before writing code, the agent should be guided to ask itself:

- Is code actually needed for this request?
- Can this be solved by explaining, configuring, or documenting instead?
- Can existing code be reused?
- Can the standard library / native platform feature solve this?
- Is a new function / abstraction / dependency really necessary?
- What is the smallest safe change?
- Does the change match the configured style?
- Does it preserve security, validation, tests, and existing behavior?
- Should the agent stop and ask before continuing?

## Planned commands (v1)

| Command | Purpose |
|---|---|
| `skillforge init` | Interactive setup; writes the project's `skillforge.yaml` config |
| `skillforge generate` | Generates AI instruction files from the current configuration |
| `skillforge sync` | Re-renders generated files when the configuration changes, preserving manual edits outside managed blocks |
| `skillforge doctor` | Checks for weak, missing, or conflicting rules in the configuration |
| `skillforge profile` | Creates/switches between reusable developer profiles (for working across multiple stacks) |

## Configurable areas

- **Developer style** — minimal, practical, learning-focused, enterprise, security-first
- **Project type** — backend API, frontend app, CLI tool, full-stack app, library, docs-heavy
- **Programming stack** — language, framework, database, testing tools, package manager
- **Security level** — basic, standard, strict
- **Testing level** — light, practical, strict
- **Dependency policy** — allow, explain-first, ask-first
- **Architecture style** — simple, simple-layered, feature-based, advanced
- **Explanation style** — short, beginner-friendly, detailed, tradeoff-based
- **Stop rules** — when the agent must stop, ask, or avoid continuing

## Generated output (v1 targets)

- `CLAUDE.md`
- `AGENTS.md`
- Cursor rules
- GitHub Copilot instructions

Shared philosophy content (the decision loop, workflow loops) is templated
once and included across all four formats, so they can't drift out of sync
with each other.

## Workflow loops

- **Feature Loop** — understand the request, check existing code, plan the smallest safe change, implement, verify, stop.
- **Bug Fix Loop** — find the root cause, avoid broad refactors, smallest fix, regression test when practical, verify.
- **Security Loop** — treat auth, validation, secrets, permissions, and DB changes as high-risk; preserve safety checks; require explanation.
- **Refactor Loop** — only refactor when requested; preserve behavior; don't mix with feature work.
- **Testing Loop** — add/update tests per configured testing level; explain how to run them.
- **Documentation Loop** — update docs only when setup, commands, env vars, or behavior actually change.

## Tech stack

Built in Rust:

- `clap` — CLI argument/subcommand parsing
- `dialoguer` — interactive `init` wizard prompts
- `serde` / `serde_yaml` — `skillforge.yaml` config (de)serialization
- `tera` — Jinja2-style templating for conditional instruction content
- `rust-embed` — bundles `templates/` into release binaries, reads from disk in debug builds so template content can be edited without recompiling
- `anyhow` — error handling
- `directories` — cross-platform resolution of `~/.config/skillforge`
- `insta` — snapshot tests for rendered template output

## Status

Early scaffold — no functionality implemented yet. See the full architecture
and build-order plan for the intended module layout
(`config`, `render`, `generators`, `cli`) and command behavior.

## Out of scope for v1

Web dashboard, cloud sync, marketplace, team billing, VS Code extension, full
autonomous agent runner, automatic PR creation, complex memory system,
enterprise policy management.
