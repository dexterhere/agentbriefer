---
title: Development setup
description: Build, run, and test Agentbriefer locally.
---

# Development setup

## Prerequisites

- a stable Rust toolchain with `rustfmt` and `clippy`;
- Git;
- Node.js 20 or newer for the documentation site.

## Build the CLI

```bash
git clone https://github.com/dexterhere/agentbriefer.git
cd agentbriefer
cargo build
cargo run -- --help
```

Run project commands against a disposable fixture directory because `init`, `generate`, `sync`, and
skill lifecycle commands write files in the current directory.

## Quality checks

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Unit tests live beside their modules. End-to-end CLI behavior lives in `tests/cli_integration.rs`, and
rendered output is protected by `insta` snapshots under `src/render/snapshots/`.

When a deliberate template change modifies snapshots, review the semantic diff before accepting it.

## Run the documentation

```bash
cd docs-site
npm install
npm start
```

Before submitting documentation changes:

```bash
npm run typecheck
npm run build
```

The production build checks internal links and emits the local search index, sitemap, and AI-readable
documentation files.

