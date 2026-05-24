# Plan: Track T45 - Antigravity CLI (agy) Integration

### Phase 1: Hook Configuration
- [ ] Task 1.1: Design and document the `hooks.json` structure required for `agy` integration in `.agents/hooks.json`.
- [ ] Task 1.2: Write a setup/init script in PowerShell to generate or update the `hooks.json` with the correct `ai-brains` trigger configuration.

### Phase 2: Transcript Capture Adapter
- [ ] Task 2.1: Implement a new module in `ai-brains-adapters` to parse the `agy` JSONL transcript format.
- [ ] Task 2.2: Ensure the parser correctly normalizes the incoming `transcriptPath` using the `ai-brains-path` crate before attempting file reads.
- [ ] Task 2.3: Write unit tests with sample `agy` JSONL data to verify correct parsing and robust error handling.

### Phase 3: Ingestion Integration
- [ ] Task 3.1: Map the parsed `agy` transcript items to `ai-brains-contracts` event DTOs.
- [ ] Task 3.2: Integrate the mapped events into the `CaptureService` event flow in `ai-brains-capture`.
- [ ] Task 3.3: Add a CLI command or flag to `ai-brains-cli` that `agy` can invoke to initiate the hook payload processing.
