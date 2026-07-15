# Changelog

All notable changes to Agentbriefer are documented here. The project follows semantic versioning.

## [Unreleased]

### Added

- `agentbriefer doctor --json` for machine-readable findings, and `agentbriefer doctor --fix` to
  automatically resync missing/stale generated outputs.

### Changed

- **Breaking:** `agentbriefer doctor` now exits with a non-zero status when it reports any finding
  classified as drift (a missing or stale generated output). Warnings about conflicting settings,
  missing fields, or unknown skill ids do not affect the exit code. Pass `--no-fail` to restore the
  previous always-exit-0 behavior.

## [1.0.0] - 2026-07-13

### Added

- Interactive project setup with editable stack detection across 28 ecosystems.
- Generation for Claude Code, AGENTS.md consumers, Cursor, and GitHub Copilot.
- Managed-block synchronization that preserves handwritten content outside generated sections.
- Developer profiles for reusable style and explanation preferences.
- Seven embedded, inspectable skills with role and stack recommendation filters.
- Skill installation, removal, refresh, and named reusable skill profiles.
- Project diagnostics for conflicting policies, missing context, unknown skills, missing outputs, and
  render drift.
- Cross-platform cargo-dist release archives, checksums, shell and PowerShell installers, and npm
  distribution under the `agentbriefer` package name.
- Versioned Docusaurus documentation with local search, examples, references, contributor guides,
  Vercel configuration, and AI-readable `llms.txt` output.

[Unreleased]: https://github.com/dexterhere/agentbriefer/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/dexterhere/agentbriefer/releases/tag/v1.0.0

