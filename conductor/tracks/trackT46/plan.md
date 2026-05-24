# Plan: Track T46 - Multi-Path Antigravity Discovery

### Phase 1: Discovery Expansion
- [ ] Task 1.1: Refactor `discover_sessions` in `crates/ai-brains-adapters/src/antigravity.rs` to support multiple base paths.
- [ ] Task 1.2: Add scanning for `antigravity-cli/brain` and `antigravity-ide/brain`.
- [ ] Task 1.3: Add scanning for `%USERPROFILE%\.gemini\tmp\*\chats\session-*.jsonl`.
- [ ] Task 1.4: Define `AntigravitySource` enum to distinguish between `BrainLog` and `ProjectChat` formats.

### Phase 2: Multi-Format Parsing
- [ ] Task 2.1: Update `parse_overview_file` to also handle `transcript.jsonl` (same format).
- [ ] Task 2.2: Implement `parse_project_chat_file` for the `session-*.jsonl` format.
- [ ] Task 2.3: Map `type: "user" | "gemini" | "claude" | "gpt-..."` to standard roles.
- [ ] Task 2.4: Ensure `thoughts` and `toolCalls` are handled according to Capture Privacy mandates (log only content).

### Phase 3: Integration & Mapping
- [ ] Task 3.1: Update `import_antigravity_sessions` to loop through all discovered sources and use the appropriate parser.
- [ ] Task 3.2: Map `projectHash` from project-chats to the `project_id` in the vault if a match is found.

### Phase 4: Verification
- [ ] Task 4.1: Add unit tests for `parse_project_chat_file`.
- [ ] Task 4.2: Add integration test with mock directory structure covering all paths.
- [ ] Task 4.3: Full CI gate pass.
