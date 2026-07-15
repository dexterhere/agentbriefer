# CI Integration

## Problem

Tools that generate AI-agent instruction files (`CLAUDE.md`, `AGENTS.md`, Cursor rules, Copilot
instructions) are being commoditized — the generation step itself is no longer a differentiator.
Nobody else in this space enforces drift detection in CI: once the files are generated, most tools
leave it to chance whether they stay in sync with the policy that produced them. Research on
AI-agent instruction file maintenance shows they go stale at a meaningful rate as projects evolve
and configs change without a corresponding resync. That gap is why `doctor` moved from a local,
opt-in linter to the product's CI-facing differentiator: a project can now fail its pipeline the
moment its instruction files drift from `agentbriefer.yaml`, instead of finding out weeks later
when an agent acts on stale guidance.

## Design

`agentbriefer doctor` (`src/cli/doctor.rs`) takes three flags, all `clap`-derived boolean switches:

- `--fix` — resyncs any missing or stale generated output (via the same `sync` machinery `agentbriefer
  sync` uses) before reporting, so a single run can both repair and report.
- `--json` — emits a `DoctorReportJson` document to stdout instead of the human-readable listing.
- `--no-fail` — always exits `0`, regardless of findings.

Findings carry a `Severity` of `Warning` or `Error`. Only `Error`-severity findings affect the exit
code: `output-missing` (a configured output was never generated) and `output-stale` (a generated
file no longer matches a fresh render) are both `Error` and `fixable: true`. `conflicting-settings`,
`missing-field`, and `unknown-skill` are all `Warning` and not fixable — they are surfaced but never
fail the command on their own.

The `--json` shape:

```json
{
  "schema_version": 1,
  "findings": [
    { "code": "output-stale", "severity": "error", "message": "...", "fixable": true }
  ],
  "summary": { "total": 1, "errors": 1, "warnings": 0, "fixed": 0 },
  "clean": false
}
```

**This is a breaking change.** Previously, `agentbriefer doctor` always exited `0` — it was a purely
advisory report. As of this change, `doctor` exits non-zero when any `Error`-severity finding
(drift) is present, unless `--no-fail` is passed. `--no-fail` is the escape hatch for anyone who
depended on the old always-succeed behavior and isn't ready to make CI enforcement blocking yet.

## The Action

`.github/actions/doctor/action.yml` is a composite action that:

1. Sets up Node via `actions/setup-node@v4`.
2. Installs the CLI with `npm install -g agentbriefer@${{ inputs.version }}`.
3. Runs `agentbriefer doctor --json` (or `--fix --json` when `inputs.fix == 'true'`), piped through
   `tee doctor-report.json` under `set -o pipefail` so the doctor process's real exit code — not
   `tee`'s — determines whether the step (and therefore the job) fails.
4. On an `if: always()` step, reads `doctor-report.json` with an inline `node -e` script and appends
   a Markdown summary (totals plus one bullet per finding) to `$GITHUB_STEP_SUMMARY`.

It installs via the published npm package, at a pinned version, rather than compiling from source or
using a raw `curl | sh` install, for three reasons: it's the same supply chain the project already
uses for its own release (`release.yml`'s `publish-npm` job pushes to the same registry), it's
faster than a from-source build on every CI run, and a versioned npm install is more auditable than
a shell-pipe install of an unpinned script.

The `doctor` step's own non-zero exit is what fails the job — the action does not add a second,
redundant pass/fail check on top of it.

## Security model

This is the canonical checklist for this Action. Audit the actual `action.yml` against it, not the
other way around:

- **No secrets required.** The action installs a public npm package and reads/writes files inside
  the checked-out workspace. It does not need `GITHUB_TOKEN`, `NPM_TOKEN`, or any other credential.
- **Recommended permissions for consumers:** `contents: read` only, set explicitly at the job level
  in the calling workflow. The action never needs `contents: write`, `pull-requests: write`, or any
  other elevated scope.
- **No auto-commit / auto-push.** The action never commits changes back to the repository, even when
  `fix: 'true'` is passed — `--fix` only rewrites files in the CI runner's ephemeral workspace so the
  subsequent `doctor` pass can report a clean (or improved) result. A "`--fix`, then open/push a PR
  with the result" flow is explicitly out of scope for this version. It's plausible future work, not
  something built here.
- **Shell interpolation is limited to trusted, caller-supplied inputs.** The only values interpolated
  into shell commands in `action.yml` are `inputs.version`, `inputs.fix`, and
  `inputs.working-directory` — all supplied by the consumer's own workflow file, which they control
  and review like any other CI config. Nothing in the action interpolates `github.event.*` payload
  data (PR titles, branch names, issue bodies, etc.), which on a `pull_request` trigger from a fork
  can carry attacker-controlled content. Mixing those two trust levels — trusted workflow-authored
  inputs vs. untrusted event-payload data — inside a shell string is the classic GitHub Actions
  script-injection mistake this action deliberately avoids.
- **`--fix`'s blast radius is bounded.** `--fix` calls the same `sync::sync` used by `agentbriefer
  sync`, which only ever writes to the four known output paths (`CLAUDE.md`, `AGENTS.md`, Cursor
  rules, Copilot instructions) selected by `config.outputs`. It cannot delete arbitrary files, cannot
  touch anything under `.agentbriefer/skills/`, and cannot remove or alter configured skills — those
  are separate write paths `doctor --fix` never calls into.

## What's explicitly not built

A "skill engine" that would auto-generate new bundled skills by ingesting other tools' skill/rules
libraries (e.g. scraping or importing third-party agent-skill collections) was proposed, researched,
and explicitly declined for this release. Reasons: unlicensed third-party content in bundled skills
would create real legal exposure; a documented security study of public agent-skill corpora found
widespread issues including prompt injection embedded in skill content, which is not a risk worth
inheriting into a CLI that ships skills as trusted, embedded data; and ingesting external content at
install or build time would break this CLI's zero-runtime-dependency release guarantee (skills and
templates are `rust-embed`-embedded at compile time, not fetched). This is recorded here so a future
contributor who rediscovers the idea has the context already and doesn't have to re-run the same
research.

## Rollout

The single most consumer-visible change from this work is the breaking exit-code change recorded in
`CHANGELOG.md`'s `[Unreleased]` section: `agentbriefer doctor` now exits non-zero on drift findings
(missing or stale generated output) by default, where it previously always exited `0`; conflicting
settings, missing fields, and unknown skill ids remain warnings that don't affect the exit code; and
`--no-fail` restores the previous always-succeed behavior for anyone not ready to make this
blocking yet.
