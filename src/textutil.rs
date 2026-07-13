//! Small text-parsing helpers shared across layers that otherwise have no
//! business depending on each other (`cli::sync` and `skills::registry`
//! both need to split a leading YAML frontmatter block from body text).

/// Splits a leading `---\n...\n---\n` YAML frontmatter block from the rest
/// of `text`. Returns `("", text)` if there is no such leading block.
pub(crate) fn split_frontmatter(text: &str) -> (&str, &str) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return ("", text);
    };

    match rest.find("\n---\n") {
        Some(offset) => {
            let end = "---\n".len() + offset + "\n---\n".len();
            (&text[..end], &text[end..])
        }
        None => ("", text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
