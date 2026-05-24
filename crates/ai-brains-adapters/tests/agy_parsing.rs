#[cfg(test)]
mod tests {
    use ai_brains_adapters::agy::{generate_deterministic_turn_id, parse_agy_transcript};
    use ai_brains_core::ids::SessionId;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[allow(clippy::disallowed_methods)]
    fn test_parse_agy_transcript() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "{{\"role\": \"user\", \"content\": \"hello\", \"timestamp\": \"2026-05-24T12:00:00Z\"}}"
        )
        .unwrap();
        writeln!(
            file,
            "{{\"role\": \"assistant\", \"content\": \"hi\", \"timestamp\": \"2026-05-24T12:00:01Z\"}}"
        )
        .unwrap();
        writeln!(
            file,
            "{{\"role\": \"system\", \"content\": \"internal\", \"timestamp\": \"2026-05-24T12:00:02Z\"}}"
        )
        .unwrap();

        let turns = parse_agy_transcript(file.path()).unwrap();
        assert_eq!(turns.len(), 3);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "hi");
        assert_eq!(turns[2].role, "system");
        assert_eq!(turns[2].content, "internal");
    }

    #[test]
    fn test_deterministic_turn_id() {
        let session_id = SessionId::new();
        let id1 = generate_deterministic_turn_id(&session_id, 0);
        let id2 = generate_deterministic_turn_id(&session_id, 0);
        let id3 = generate_deterministic_turn_id(&session_id, 1);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
