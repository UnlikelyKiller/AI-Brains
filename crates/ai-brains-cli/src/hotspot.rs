use strip_ansi_escapes::strip_str;

/// Strip ANSI escape sequences from raw changeguard output.
pub fn strip_ansi_from_output(raw: &str) -> String {
    strip_str(raw)
}

/// Condense a changeguard hotspot table into a compact summary.
/// Takes the raw table output (after ANSI stripping) and extracts
/// just the data rows, discarding formatting headers, dividers,
/// and decorative whitespace. Truncates to 5 entries max.
pub fn condense_hotspots(stripped_output: &str) -> String {
    let mut lines: Vec<String> = Vec::new();

    for line in stripped_output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip table dividers (all dashes, all equals, or separator lines)
        if trimmed.chars().all(|c| c == '-' || c == '=' || c == '+') {
            continue;
        }
        // Skip lines that are purely decorative borders (e.g. "=====" or "-----")
        if trimmed.starts_with("===") || trimmed.starts_with("---") {
            continue;
        }
        lines.push(trimmed.to_string());
    }

    if lines.is_empty() {
        return String::new();
    }

    // Truncate to 5 data lines plus a notice
    if lines.len() > 5 {
        lines.truncate(5);
        lines.push("... (truncated)".to_string());
    }

    lines.join("\n")
}

/// Full pipeline: strip ANSI and condense a changeguard output.
pub fn sanitize_and_condense(raw_output: &str) -> String {
    let stripped = strip_ansi_from_output(raw_output);
    condense_hotspots(&stripped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_and_condenses() {
        let raw = "\u{1b}[1m\u{1b}[4m  FILE          SCORE  \u{1b}[0m\n\u{1b}[32m src/main.rs   42     \u{1b}[0m\n\u{1b}[32m src/lib.rs     28     \u{1b}[0m\n===\n\u{1b}[32m src/mod.rs     15     \u{1b}[0m";
        let result = sanitize_and_condense(raw);
        assert!(
            !result.contains('\x1b'),
            "Result should not contain ANSI escapes"
        );
        assert!(
            result.contains("src/main.rs"),
            "Result should contain data rows"
        );
        assert!(
            result.contains("src/lib.rs"),
            "Result should contain data rows"
        );
        assert!(
            !result.contains("==="),
            "Result should not contain dividers"
        );
    }

    #[test]
    fn condenses_truncates_at_five() {
        let input =
            "file1.rs 10\nfile2.rs 9\nfile3.rs 8\nfile4.rs 7\nfile5.rs 6\nfile6.rs 5\nfile7.rs 4";
        let result = condense_hotspots(input);
        assert!(
            result.contains("... (truncated)"),
            "Result should contain truncation notice"
        );
        let line_count = result.lines().count();
        assert_eq!(
            line_count, 6,
            "Should have 5 data lines + 1 truncation notice"
        );
    }

    #[test]
    fn condenses_strips_empty_lines() {
        let input = "\n\nsrc/main.rs 42\n\nsrc/lib.rs 28\n\n";
        let result = condense_hotspots(input);
        assert!(
            !result.starts_with('\n'),
            "Result should not start with empty line"
        );
        assert!(result.contains("src/main.rs 42"));
    }

    #[test]
    fn handles_clean_input() {
        let input = "src/main.rs 42\nsrc/lib.rs 28";
        assert_eq!(condense_hotspots(input), input);
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(condense_hotspots(""), "");
        assert_eq!(condense_hotspots("\n\n\n"), "");
    }

    #[test]
    fn strips_divider_lines() {
        let input = "HEADER\n------\nsrc/main.rs 42\n======\nsrc/lib.rs 28";
        let result = condense_hotspots(input);
        assert!(!result.contains("------"), "Should strip dash dividers");
        assert!(!result.contains("======"), "Should strip equals dividers");
    }
}
