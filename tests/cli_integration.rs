//! Black-box tests that spawn the actual compiled `agentbriefer` binary,
//! unlike every other test in this crate (which calls internal functions
//! directly). Covers the non-interactive commands only — `init` and
//! `profile create`/`switch` need a real terminal for `dialoguer`'s raw
//! mode and are verified manually instead (see the project plan).

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

const SAMPLE_CONFIG: &str = "\
developer:
  style: practical
  explanation_style: short
project:
  project_type: cli-tool
  stack:
    language: rust
    package_manager: cargo
  security_level: standard
  testing_level: practical
  dependency_policy: explain-first
  architecture_style: simple
stop_rules: []
";

fn agentbriefer() -> Command {
    Command::cargo_bin("agentbriefer").unwrap()
}

/// Isolates `profile`-family commands from the real developer's
/// `~/.config/agentbriefer/profiles/` — every test must call this rather than
/// ever touching that directory.
fn isolate_profiles_dir(cmd: &mut Command, config_home: &Path) {
    cmd.env("XDG_CONFIG_HOME", config_home);
}

fn write_config(root: &Path) {
    fs::write(root.join("agentbriefer.yaml"), SAMPLE_CONFIG).unwrap();
}

#[test]
fn generate_without_a_config_fails_with_a_helpful_hint() {
    let dir = tempdir().unwrap();

    agentbriefer()
        .current_dir(dir.path())
        .arg("generate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("agentbriefer init"));
}

#[test]
fn generate_writes_all_four_output_files() {
    let dir = tempdir().unwrap();
    write_config(dir.path());

    agentbriefer()
        .current_dir(dir.path())
        .arg("generate")
        .assert()
        .success();

    for path in [
        "CLAUDE.md",
        "AGENTS.md",
        ".cursor/rules/agentbriefer.mdc",
        ".github/copilot-instructions.md",
    ] {
        let content = fs::read_to_string(dir.path().join(path))
            .unwrap_or_else(|_| panic!("{path} should have been written"));
        assert!(content.contains("## Decision Loop"));
    }
}

#[test]
fn sync_preserves_manual_edits_outside_the_managed_block_on_resync() {
    let dir = tempdir().unwrap();
    write_config(dir.path());

    agentbriefer()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    let claude_md = dir.path().join("CLAUDE.md");
    let synced = fs::read_to_string(&claude_md).unwrap();
    let with_manual_notes = format!("My own manual notes.\n\n{synced}");
    fs::write(&claude_md, with_manual_notes).unwrap();

    agentbriefer()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    let resynced = fs::read_to_string(&claude_md).unwrap();
    assert!(resynced.contains("My own manual notes."));
    assert!(resynced.contains("## Decision Loop"));
}

#[test]
fn doctor_flags_conflicting_security_and_dependency_settings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("agentbriefer.yaml"),
        SAMPLE_CONFIG
            .replace("security_level: standard", "security_level: strict")
            .replace(
                "dependency_policy: explain-first",
                "dependency_policy: allow",
            ),
    )
    .unwrap();

    agentbriefer()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("dependency policy is 'allow'"));
}

#[test]
fn doctor_reports_no_issues_once_generated_and_in_sync() {
    let dir = tempdir().unwrap();
    write_config(dir.path());

    agentbriefer()
        .current_dir(dir.path())
        .arg("generate")
        .assert()
        .success();

    agentbriefer()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found."));
}

#[test]
fn profile_list_hints_when_no_profiles_are_saved_yet() {
    let dir = tempdir().unwrap();
    let config_home = tempdir().unwrap();

    let mut cmd = agentbriefer();
    cmd.current_dir(dir.path()).arg("profile").arg("list");
    isolate_profiles_dir(&mut cmd, config_home.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("profile create"));
}
