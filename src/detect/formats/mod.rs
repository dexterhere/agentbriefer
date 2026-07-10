//! Shared low-level parsers reused across multiple language detectors, so
//! each language module stays a thin "locate manifest, extract names,
//! categorize" wrapper instead of duplicating format-walking code.

pub(super) mod gradle;
pub(super) mod xml;
