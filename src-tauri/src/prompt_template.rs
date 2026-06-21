//! Pure prompt-template substitution for post-processing prompts.
//!
//! This module is intentionally I/O-free and has no Tauri dependency so it
//! can be unit-tested in isolation and reused from any context that needs to
//! render a prompt template against a transcription output and a custom-words
//! string.

/// Sentinel substituted into `PromptContext::custom_words` (and templates)
/// when the user has not configured any custom words.
///
/// Kept as a public constant so callers can feed the same sentinel into
/// [`PromptContext`] and have it render verbatim in the resulting prompt,
/// keeping the LLM from seeing an empty "Custom Words:" section.
pub const EMPTY_CUSTOM_WORDS: &str = "(none provided)";

/// Borrowed inputs to [`render`].
///
/// `output` is the raw transcription text and `custom_words` is the
/// pre-formatted custom-words string (or [`EMPTY_CUSTOM_WORDS`] when the user
/// has none configured). Both fields are borrowed for the lifetime of the
/// context so rendering never needs to clone until the final [`String`] is
/// produced.
pub struct PromptContext<'a> {
    pub output: &'a str,
    pub custom_words: &'a str,
}

/// Renders `template` by substituting the `${output}` and `${custom_words}`
/// placeholders with the corresponding fields from `ctx`.
///
/// Substitution is a dumb [`str::replace`] for each placeholder, so it is
/// deterministic and performs no special parsing of the surrounding text.
/// Multiple occurrences of the same placeholder are all replaced. Characters
/// that look like placeholder syntax (`$`, `{`, `}`) but are not exactly one
/// of the two supported placeholders are left untouched.
pub fn render(template: &str, ctx: &PromptContext) -> String {
    template
        .replace("${output}", ctx.output)
        .replace("${custom_words}", ctx.custom_words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_both_placeholders_substituted() {
        let ctx = PromptContext {
            output: "hello world",
            custom_words: "foo, bar",
        };
        let template = "Output: ${output}\nCustom words: ${custom_words}";
        assert_eq!(
            render(template, &ctx),
            "Output: hello world\nCustom words: foo, bar"
        );
    }

    #[test]
    fn test_only_output_placeholder_present() {
        let ctx = PromptContext {
            output: "hello world",
            custom_words: "foo, bar",
        };
        // The literal "Custom words: (none configured)" contains no
        // ${custom_words} placeholder, so it must be returned untouched even
        // though ctx.custom_words is non-empty.
        let template = "Output: ${output}\nCustom words: (none configured)";
        assert_eq!(
            render(template, &ctx),
            "Output: hello world\nCustom words: (none configured)"
        );
    }

    #[test]
    fn test_only_custom_words_placeholder_present() {
        let ctx = PromptContext {
            output: "hello world",
            custom_words: "foo, bar",
        };
        // The literal "(transcription pending)" contains no ${output}
        // placeholder, so it must be returned untouched even though
        // ctx.output is non-empty.
        let template = "Output: (transcription pending)\nCustom words: ${custom_words}";
        assert_eq!(
            render(template, &ctx),
            "Output: (transcription pending)\nCustom words: foo, bar"
        );
    }

    #[test]
    fn test_empty_custom_words_renders_literal() {
        let ctx = PromptContext {
            output: "hello world",
            custom_words: EMPTY_CUSTOM_WORDS,
        };
        let template = "Output: ${output}\nCustom words: ${custom_words}";
        assert_eq!(
            render(template, &ctx),
            "Output: hello world\nCustom words: (none provided)"
        );
    }

    #[test]
    fn test_no_placeholders_returned_unchanged() {
        let ctx = PromptContext {
            output: "hello world",
            custom_words: "foo, bar",
        };
        let template = "This template has no placeholders at all.";
        assert_eq!(
            render(template, &ctx),
            "This template has no placeholders at all."
        );
    }

    #[test]
    fn test_multiple_occurrences_all_substituted() {
        let ctx = PromptContext {
            output: "hi",
            custom_words: "foo",
        };
        let template = "${output} and ${output} and ${custom_words} and ${custom_words}";
        assert_eq!(render(template, &ctx), "hi and hi and foo and foo");
    }

    #[test]
    fn test_special_chars_outside_placeholder_not_substituted() {
        let ctx = PromptContext {
            output: "hello",
            custom_words: "foo",
        };
        // $, {, and } appearing individually or in non-matching combinations
        // must pass through untouched. Only the exact substrings
        // "${output}" and "${custom_words}" are replaced.
        let template = "Price: $5, set: {a, b}, end: }\n${output} ${custom_words}";
        assert_eq!(
            render(template, &ctx),
            "Price: $5, set: {a, b}, end: }\nhello foo"
        );
    }
}
