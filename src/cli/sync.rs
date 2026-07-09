//! `skillforge sync` — re-renders configured outputs while preserving any
//! manual edits made outside the SkillForge-managed block.
//!
//! Unlike `generate` (a blunt overwrite), `sync` wraps the content it owns
//! in `<!-- skillforge:managed:start -->` / `<!-- skillforge:managed:end -->`
//! markers and only ever replaces what's between them. Anything you add
//! above or below the markers survives every future `sync`. Running `generate`
//! on a synced file does *not* preserve those manual additions — `generate`
//! has no concept of markers, by design (see the project plan).

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::generate::OutputReport;
use super::generate::{output_path, refuse_if_symlink};
use super::ui;
use crate::config::{self, SkillforgeConfig};
use crate::render::Renderer;

const MARKER_START: &str = "<!-- skillforge:managed:start -->";
const MARKER_END: &str = "<!-- skillforge:managed:end -->";

/// Runs `skillforge sync` in the current directory.
pub fn run() -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve the current directory")?;
    let config_path = root.join(config::CONFIG_FILE_NAME);

    let config = config::load(&config_path).with_context(|| {
        format!(
            "no {} found — run `skillforge init` first",
            config::CONFIG_FILE_NAME
        )
    })?;

    let renderer = Renderer::new().context("failed to initialize the template engine")?;

    let report = sync(&root, &config, &renderer)?;

    for (format, path) in &report.succeeded {
        ui::success(&format!("synced {format} -> {}", path.display()));
    }
    for (format, reason) in &report.failed {
        ui::warn(&format!("skipped {format}: {reason}"));
    }

    Ok(())
}

/// Renders and merges every format in `config.outputs` into its output
/// file, relative to `root`. Kept separate from [`run`] so it can be
/// exercised in tests against a temporary directory.
fn sync(root: &Path, config: &SkillforgeConfig, renderer: &Renderer) -> Result<OutputReport> {
    let mut report = OutputReport::default();

    for &format in &config.outputs {
        match renderer.render_output(format, config) {
            Ok(rendered) => {
                let path = root.join(output_path(format));
                write_merged(&path, &rendered)?;
                report.succeeded.push((format, path));
            }
            Err(err) => {
                report.failed.push((format, err.to_string()));
            }
        }
    }

    Ok(report)
}

/// Writes `rendered` to `path`, preserving any content outside an existing
/// managed block (see the module docs for the merge rules).
fn write_merged(path: &Path, rendered: &str) -> Result<()> {
    refuse_if_symlink(path)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let (frontmatter, body) = split_frontmatter(rendered);
    let block = managed_block(body);

    let content = match fs::read_to_string(path) {
        Ok(existing) => match find_managed_block(&existing) {
            Some((start, end)) => {
                format!("{}{block}{}", &existing[..start], &existing[end..])
            }
            None => {
                ui::warn(&format!(
                    "no managed block found in {} — replacing its contents with a fresh one",
                    path.display()
                ));
                format!("{frontmatter}{block}")
            }
        },
        Err(_) => format!("{frontmatter}{block}"),
    };

    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

/// Wraps `body` in the managed-block markers, as written to disk.
pub(super) fn managed_block(body: &str) -> String {
    format!("{MARKER_START}\n{body}\n{MARKER_END}\n")
}

/// Splits a rendered template's leading YAML frontmatter (`---\n...\n---\n`)
/// from its body. Only `cursor_rules.tera` currently emits frontmatter —
/// everything else returns an empty frontmatter and the whole string as the
/// body. Frontmatter must stay the literal first bytes of a `.mdc` file for
/// Cursor to parse it, so it's kept outside (above) the managed block rather
/// than wrapped inside it.
pub(super) fn split_frontmatter(rendered: &str) -> (&str, &str) {
    let Some(rest) = rendered.strip_prefix("---\n") else {
        return ("", rendered);
    };

    match rest.find("\n---\n") {
        Some(offset) => {
            let end = "---\n".len() + offset + "\n---\n".len();
            (&rendered[..end], &rendered[end..])
        }
        None => ("", rendered),
    }
}

/// Finds the byte offset of `marker`'s first occurrence that sits alone on
/// its own line -- immediately preceded by the start of the string or a
/// newline, and immediately followed by the end of the string or a
/// newline. The genuine markers `sync` writes are always alone on their
/// own line; a config value that merely *contains* the literal marker text
/// (e.g. a stop rule quoting it, whether by accident or on purpose) never
/// is, since it has other text sharing its line. Without this line anchor,
/// such a value would be mistaken for a real boundary and silently corrupt
/// the file on the next sync (content after the fake marker gets treated
/// as "outside the block" and duplicated back in verbatim).
fn find_marker_line(haystack: &str, marker: &str) -> Option<usize> {
    haystack.match_indices(marker).find_map(|(pos, _)| {
        let preceded_by_newline = pos == 0 || haystack.as_bytes()[pos - 1] == b'\n';
        let after = pos + marker.len();
        let followed_by_newline = after == haystack.len() || haystack.as_bytes()[after] == b'\n';
        (preceded_by_newline && followed_by_newline).then_some(pos)
    })
}

/// Finds the byte range covering an existing managed block, including its
/// marker lines and the end marker's trailing newline. Returns `None` if
/// the markers are missing, out of order, or otherwise malformed.
pub(super) fn find_managed_block(existing: &str) -> Option<(usize, usize)> {
    let start = find_marker_line(existing, MARKER_START)?;
    let after_start = start + MARKER_START.len();
    let end = after_start + find_marker_line(&existing[after_start..], MARKER_END)?;
    let end = end + MARKER_END.len();
    let end = existing[end..].strip_prefix('\n').map_or(end, |_| end + 1);

    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ArchitectureStyle, DependencyPolicy, DeveloperProfile, DeveloperStyle, ExplanationStyle,
        OutputFormat, ProjectProfile, ProjectType, SecurityLevel, Stack, TestingLevel,
    };

    fn sample_config() -> SkillforgeConfig {
        SkillforgeConfig {
            extends: None,
            developer: DeveloperProfile {
                style: DeveloperStyle::Practical,
                explanation_style: ExplanationStyle::Short,
            },
            project: ProjectProfile {
                project_type: ProjectType::CliTool,
                stack: Stack {
                    language: "rust".to_string(),
                    framework: None,
                    database: None,
                    testing_tools: vec![],
                    package_manager: Some("cargo".to_string()),
                    key_dependencies: vec![],
                },
                security_level: SecurityLevel::Standard,
                testing_level: TestingLevel::Practical,
                dependency_policy: DependencyPolicy::ExplainFirst,
                architecture_style: ArchitectureStyle::Simple,
            },
            stop_rules: vec![],
            custom_instructions: None,
            outputs: OutputFormat::all(),
        }
    }

    #[test]
    fn split_frontmatter_extracts_leading_yaml_block() {
        let rendered = "---\ndescription: x\nalwaysApply: true\n---\n\n# Body\ncontent\n";

        let (frontmatter, body) = split_frontmatter(rendered);

        assert_eq!(frontmatter, "---\ndescription: x\nalwaysApply: true\n---\n");
        assert_eq!(body, "\n# Body\ncontent\n");
    }

    #[test]
    fn split_frontmatter_is_empty_when_there_is_none() {
        let rendered = "# CLAUDE.md\n\ncontent\n";

        let (frontmatter, body) = split_frontmatter(rendered);

        assert_eq!(frontmatter, "");
        assert_eq!(body, rendered);
    }

    #[test]
    fn first_sync_creates_a_managed_block() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let report = sync(dir.path(), &config, &renderer).unwrap();

        assert!(
            report.failed.is_empty(),
            "unexpected failures: {:?}",
            report.failed
        );
        let (_, path) = report
            .succeeded
            .iter()
            .find(|(format, _)| *format == OutputFormat::ClaudeMd)
            .unwrap();
        let content = fs::read_to_string(path).unwrap();

        assert!(content.starts_with(MARKER_START));
        assert!(content.contains(MARKER_END));
        assert!(content.contains("## Decision Loop"));
    }

    #[test]
    fn first_sync_keeps_cursor_frontmatter_outside_the_managed_block() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let config = sample_config();

        let report = sync(dir.path(), &config, &renderer).unwrap();

        let (_, path) = report
            .succeeded
            .iter()
            .find(|(format, _)| *format == OutputFormat::CursorRules)
            .unwrap();
        let content = fs::read_to_string(path).unwrap();

        assert!(
            content.starts_with("---\n"),
            "frontmatter must be the first bytes of a .mdc file"
        );
        let marker_pos = content.find(MARKER_START).unwrap();
        let frontmatter_end = content.find("\n---\n").unwrap() + "\n---\n".len();
        assert!(
            marker_pos >= frontmatter_end,
            "managed block must start after the frontmatter"
        );
    }

    #[test]
    fn managed_marker_text_embedded_inside_a_line_does_not_confuse_the_boundary() {
        // A rendered body that merely *contains* the literal end-marker text
        // (e.g. a stop rule that happens to quote it) must not be mistaken
        // for the real end marker, which is always alone on its own line.
        let existing = format!(
            "{MARKER_START}\nRogue line with {MARKER_END} embedded mid-sentence.\nreal tail content\n{MARKER_END}\n"
        );

        let (start, end) = find_managed_block(&existing).unwrap();

        assert_eq!(&existing[start..start + MARKER_START.len()], MARKER_START);
        assert!(
            existing[..end].ends_with(&format!("{MARKER_END}\n")),
            "boundary must land on the real, line-alone end marker, not the embedded text"
        );
        assert_eq!(end, existing.len());
    }

    #[test]
    fn resync_preserves_manual_content_outside_the_markers() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let config = sample_config();
        let path = dir.path().join("CLAUDE.md");

        fs::write(
            &path,
            format!(
                "My own notes above.\n\n{MARKER_START}\nstale body\n{MARKER_END}\n\nMy own notes below.\n"
            ),
        )
        .unwrap();

        write_merged(
            &path,
            &renderer
                .render_output(OutputFormat::ClaudeMd, &config)
                .unwrap(),
        )
        .unwrap();
        let content = fs::read_to_string(&path).unwrap();

        assert!(content.contains("My own notes above."));
        assert!(content.contains("My own notes below."));
        assert!(content.contains("## Decision Loop"));
        assert!(!content.contains("stale body"));
    }

    #[test]
    fn resync_without_existing_markers_replaces_the_whole_file() {
        let dir = tempfile::tempdir().unwrap();
        let renderer = Renderer::new().unwrap();
        let config = sample_config();
        let path = dir.path().join("CLAUDE.md");

        fs::write(
            &path,
            "Some old hand-written CLAUDE.md with no markers at all.\n",
        )
        .unwrap();

        write_merged(
            &path,
            &renderer
                .render_output(OutputFormat::ClaudeMd, &config)
                .unwrap(),
        )
        .unwrap();
        let content = fs::read_to_string(&path).unwrap();

        assert!(!content.contains("Some old hand-written CLAUDE.md"));
        assert!(content.contains(MARKER_START));
        assert!(content.contains("## Decision Loop"));
    }

    #[test]
    #[cfg(unix)]
    fn refuses_to_write_through_a_symlinked_output_path() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let sentinel = outside.path().join("sensitive.txt");
        fs::write(&sentinel, "do not touch").unwrap();
        let path = dir.path().join("CLAUDE.md");
        std::os::unix::fs::symlink(&sentinel, &path).unwrap();

        let result = write_merged(&path, "rendered content");

        assert!(result.is_err(), "sync should refuse the symlink");
        assert_eq!(fs::read_to_string(&sentinel).unwrap(), "do not touch");
    }
}
