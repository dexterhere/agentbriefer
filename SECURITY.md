# Security Policy

## Supported Versions

SkillForge CLI has not yet made a versioned/tagged release — it's early-stage,
pre-1.0 software. Security fixes are made against the latest commit on the
`main` branch only; there is no support for older commits or forks.

| Version              | Supported          |
| --------------------- | ------------------ |
| `main` (latest commit) | :white_check_mark: |
| Anything older         | :x:                 |

Once this project starts publishing tagged releases (e.g. to crates.io),
this table will be updated to list which released versions still receive
security fixes.

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Report privately using GitHub's built-in vulnerability reporting:

1. Go to the [Security tab](https://github.com/dexterhere/skillforge/security) of this repository.
2. Click **Report a vulnerability**.
3. Include as much of the following as you can:
   - A description of the issue and its potential impact.
   - Steps to reproduce it, or a minimal example (e.g. a `skillforge.yaml`
     or profile file that triggers the problem).
   - The commit hash or version you tested against.
   - Any suggested fix, if you have one.

This is a solo-maintained, early-stage project, so responses are best-effort
rather than covered by a fixed SLA. You can expect an acknowledgment within
a few days. If a report turns out to be a real vulnerability, a fix will be
prioritized ahead of other work, and I'll credit you in the fix (commit
message and/or a GitHub Security Advisory) unless you'd prefer to stay
anonymous.

## Scope

**In scope:**
- The `skillforge` CLI binary itself: config loading (`skillforge.yaml`),
  developer profile files, template rendering, and file generation/sync.
- Anything that would let a malicious `skillforge.yaml` or profile file
  (including ones fetched from a future skill marketplace) cause more than
  a bad configuration — e.g. writing outside the intended project directory,
  path traversal via a crafted profile name, or executing anything other
  than rendering a template into a file.

**Out of scope / handled differently:**
- Vulnerabilities in third-party dependencies (`clap`, `serde`, `tera`,
  etc.) — please report those to the upstream crate first. If you find one
  that affects SkillForge specifically (e.g. it's reachable with
  attacker-controlled input), a report here is still welcome so the
  dependency can be updated.
- Social engineering, physical access, or issues that require the user to
  already run untrusted code with their own permissions.

**Why the surface is small today:** SkillForge does not execute arbitrary
code from configuration. `skillforge.yaml` and profile files deserialize
into a fixed set of typed fields (enums and plain strings) via `serde` —
there's no code path that turns config content into a shell command or
arbitrary file write outside what `generate`/`sync` already do by design.
Templates are shipped with the binary itself (`rust-embed`), not supplied
by users at runtime, so there is currently no template-injection surface.
The planned skill marketplace feature will only ever download typed YAML
profile data over HTTPS, never executable code — if that changes, this
policy will be updated before the feature ships.

## Disclosure Policy

Once a reported vulnerability is confirmed and fixed, it will be disclosed
via a GitHub Security Advisory on this repository, with credit to the
reporter (unless anonymity is requested). Please give a reasonable amount
of time for a fix to land before disclosing publicly elsewhere.
