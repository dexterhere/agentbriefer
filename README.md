<p align="center">
  <img src="assets/agentbriefer-icon.svg" width="88" height="88" alt="Agentbriefer logo" />
</p>

<h1 align="center">Agentbriefer</h1>

<p align="center">
  Configure how AI coding agents think, code, test, and stop inside your project.
</p>

<p align="center">
  <a href="https://github.com/dexterhere/agentbriefer/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/dexterhere/agentbriefer/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="https://www.npmjs.com/package/agentbriefer"><img alt="npm" src="https://img.shields.io/npm/v/agentbriefer" /></a>
  <a href="LICENSE-MIT"><img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" /></a>
</p>

Agentbriefer turns one reviewable `agentbriefer.yaml` into native repository instructions for Claude
Code, AGENTS.md-compatible agents, Cursor, and GitHub Copilot. It keeps behavioral policy consistent
across tools while supporting reusable developer profiles, focused skills, and safe synchronization
with handwritten guidance.

## Install

```bash
npm install -g agentbriefer
agentbriefer --version
```

Also available through GitHub release archives, shell and PowerShell installers, Cargo from the Git
tag, or a source build. See the [installation guide](https://docsagentbriefer.vercel.app/docs/installation).

## Quick start

```bash
cd your-project
agentbriefer init
agentbriefer generate
agentbriefer skill list --recommended
agentbriefer skill add no-secrets-in-repo
agentbriefer doctor
```

The default configuration generates:

```text
CLAUDE.md
AGENTS.md
.cursor/rules/agentbriefer.mdc
.github/copilot-instructions.md
```

Use `agentbriefer sync` when an output also contains manual notes. It updates only the
Agentbriefer-managed block; `generate` replaces the whole file.

## Why Agentbriefer

- **One source of truth:** align several coding agents without duplicating policy.
- **Concrete guardrails:** configure scope, architecture, dependencies, testing, security, and stop
  rules.
- **Stack-aware setup:** prefill project context from manifests across 28 ecosystems.
- **Focused skills:** browse, inspect, install, update, remove, and group seven bundled practices.
- **Reusable preferences:** carry developer and explanation styles between projects.
- **Reviewable output:** commit YAML and generated instructions with the application code they guide.

## Commands

| Command | Purpose |
| --- | --- |
| `agentbriefer init` | Detect the stack and create `agentbriefer.yaml` through an interactive wizard. |
| `agentbriefer generate` | Fully render every configured instruction file. |
| `agentbriefer sync` | Refresh managed sections while preserving surrounding content. |
| `agentbriefer doctor` | Report policy conflicts, unknown skills, missing outputs, and drift. |
| `agentbriefer profile …` | List, create, and switch reusable developer profiles. |
| `agentbriefer skill …` | Discover and manage project skills and reusable skill profiles. |

Read the [full documentation](https://docsagentbriefer.vercel.app/), browse the
[examples](https://docsagentbriefer.vercel.app/docs/examples/overview), or use the
[command reference](https://docsagentbriefer.vercel.app/docs/reference/commands).

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The versioned Docusaurus site lives in [`docs-site`](docs-site/README.md). Architecture details are in
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), and contribution setup is documented on the
[contributor site](https://docsagentbriefer.vercel.app/docs/contributing/development).

## Security and license

Report vulnerabilities according to [`SECURITY.md`](SECURITY.md). Agentbriefer is available under
MIT or Apache-2.0, at your option.
