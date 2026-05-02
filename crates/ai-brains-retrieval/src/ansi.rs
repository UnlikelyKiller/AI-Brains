use strip_ansi_escapes::strip_str;

/// Remove ANSI escape sequences (color codes, bold, dim, etc.) from text.
/// Used to sanitize external command output before pinning to the vault,
/// and as a defense-in-depth measure when assembling preflight context.
pub fn strip_ansi(input: &str) -> String {
    strip_str(input)
}

#[cfg(test)]
mod tests {
    use super::strip_ansi;

    #[test]
    fn strips_color_codes() {
        let input = "\u{1b}[32m INFO\u{1b}[0m file loaded";
        assert_eq!(strip_ansi(input), " INFO file loaded");
    }

    #[test]
    fn strips_bold_and_dim() {
        let input = "\u{1b}[1mBOLD\u{1b}[0m \u{1b}[2mDIM\u{1b}[0m";
        assert_eq!(strip_ansi(input), "BOLD DIM");
    }

    #[test]
    fn passes_clean_text_through() {
        let input = "clean text no escapes";
        assert_eq!(strip_ansi(input), input);
    }

    #[test]
    fn strips_complex_ansi_sequences() {
        let input = "\u{1b}[32;1m\u{1b}[40mHEADER\u{1b}[0m";
        assert_eq!(strip_ansi(input), "HEADER");
    }

    #[test]
    fn handles_empty_string() {
        assert_eq!(strip_ansi(""), "");
    }

    #[test]
    fn handles_multiline_with_ansi() {
        let input = "\u{1b}[32mOK\u{1b}[0m\n\u{1b}[31mFAIL\u{1b}[0m";
        assert_eq!(strip_ansi(input), "OK\nFAIL");
    }
}
