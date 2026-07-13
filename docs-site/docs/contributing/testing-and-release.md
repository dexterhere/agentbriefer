---
title: Testing and release
description: Validation layers and the Agentbriefer v1 release checklist.
---

# Testing and release

## Validation layers

Run the complete Rust gate:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Run the documentation gate:

```bash
cd docs-site
npm ci
npm run typecheck
npm run build
```

Also exercise `agentbriefer init`, `generate`, `sync`, `doctor`, and the skill lifecycle in a temporary
project when user-visible workflows change.

## Release checklist

1. Set the same semantic version in `Cargo.toml`, the changelog, and the release notes.
2. Verify generated snapshots and documentation examples against the current clap help.
3. Build docs and confirm stable plus Next navigation, local search, sitemap, `llms.txt`, and
   `llms-full.txt`.
4. Run the Rust validation gate on a clean checkout.
5. Review `dist-workspace.toml` targets and installers.
6. Create and push the matching Git tag only after the release commit is approved.
7. Verify the GitHub release archives, checksums, shell and PowerShell installers, and npm package.
8. Smoke-test `agentbriefer --version` and `agentbriefer skill list` through at least one published
   installation path.

The generated cargo-dist workflow builds macOS ARM64/x64, Linux ARM64/x64, and Windows x64 artifacts,
then publishes GitHub-hosted installers and the npm package. Tagging and publishing change external
state and remain a maintainer-controlled action.

## Versioned docs

Stable documentation is a snapshot under `versioned_docs/version-<version>/`. Ongoing source docs
under `docs/` appear as **Next**. Create a new snapshot only for a real release boundary; do not edit
an old snapshot to describe later behavior.

