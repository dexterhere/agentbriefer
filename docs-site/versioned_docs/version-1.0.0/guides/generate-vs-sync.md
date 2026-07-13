---
title: Generate vs. sync
description: Choose between full output replacement and managed-block synchronization.
---

# Generate vs. sync

Both commands render the same configuration and installed skills. They differ
only in how the result is written.

| Behavior | `generate` | `sync` |
|---|---:|---:|
| Render every configured format | Yes | Yes |
| Create parent directories | Yes | Yes |
| Replace the whole file | Yes | Only when no managed block exists |
| Preserve content outside markers | No | Yes |
| Refuse final-path symlinks | Yes | Yes |

## Generate

```bash
agentbriefer generate
```

Use it for the first render or when outputs are completely machine-owned. It
always writes a clean template result and discards existing file contents.

## Sync

```bash
agentbriefer sync
```

Sync wraps the generated body in markers:

```markdown
<!-- agentbriefer:managed:start -->
Generated Agentbriefer content lives here.
<!-- agentbriefer:managed:end -->
```

Future syncs replace that block and preserve content before or after it.

```markdown title="CLAUDE.md"
My repository-specific note stays here.

<!-- agentbriefer:managed:start -->
Agentbriefer replaces only this section.
<!-- agentbriefer:managed:end -->

Another manual note also stays here.
```

Cursor's YAML frontmatter stays before the managed block because `.mdc` files
must begin with frontmatter.

## Missing or malformed markers

If a file exists without a valid managed block, sync warns and replaces the
whole file with a fresh managed result. Markers are recognized only when they
appear alone on their own lines, preventing quoted marker text inside a stop
rule from corrupting synchronization.

## Recommended workflow

1. Run `generate` on initial setup.
2. If no manual output edits are needed, continue using `generate` or `sync`.
3. Before adding manual notes, run `sync` once to establish markers.
4. Add notes only outside the markers.
5. Use `sync` thereafter.

:::danger[Do not switch back casually]

Running `generate` after manual edits fully overwrites the file. Agentbriefer
does not try to recover discarded content.

:::
