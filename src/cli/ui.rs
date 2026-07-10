//! Shared presentation helpers: color, hints, and the startup banner.
//!
//! Kept as thin `println!` wrappers around `owo-colors` so the five command
//! modules don't each hand-roll their own styling.

use dialoguer::theme::ColorfulTheme;
use owo_colors::OwoColorize;

/// The theme every interactive prompt (`Select`, `MultiSelect`, `Input`,
/// `Confirm`) should be built with.
pub(super) fn theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Prints a dimmed one-line explanation just before a prompt. Because it's
/// a plain `println!` issued before `.interact()`, dialoguer's redraw-in-
/// place behavior leaves it on screen rather than overwriting it.
pub(super) fn hint(text: &str) {
    println!("{}", text.dimmed());
}

/// Prints a green status line for a successful outcome.
pub(super) fn success(text: &str) {
    println!("{}", text.green());
}

/// Prints a yellow status line for something worth the user's attention.
pub(super) fn warn(text: &str) {
    println!("{}", text.yellow());
}

/// Prints the "SkillForge" banner and tagline. Called once, at the start
/// of `init` — the one genuine "first impression" moment; every other
/// command is run too often day-to-day to justify reprinting it.
pub(super) fn print_banner() {
    if let Ok(font) = figlet_rs::FIGfont::standard()
        && let Some(figure) = font.convert("SkillForge")
    {
        println!("{}", figure.to_string().cyan().bold());
    } else {
        println!("{}", "SkillForge".cyan().bold());
    }

    println!(
        "{}",
        "Configure how AI coding agents think, code, test, and stop.".dimmed()
    );
    println!();
}
