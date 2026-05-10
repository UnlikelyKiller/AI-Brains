## Plan: CLI Refactor (Decomposing main.rs God File)
### Phase 1: Setup and AppContext
- [ ] Task 1.1: Create `crates/ai-brains-cli/src/context.rs`.
- [ ] Task 1.2: Define `AppContext` struct with methods to initialize `VaultConnection` and common services.
- [ ] Task 1.3: Update `crates/ai-brains-cli/src/main.rs` to instantiate `AppContext` early in the execution flow.

### Phase 2: Command Extraction (Easy Commands)
- [ ] Task 2.1: Create `crates/ai-brains-cli/src/commands/mod.rs` and configure the module tree.
- [ ] Task 2.2: Extract `Init` and `Context` commands into `commands/init.rs` and `commands/context.rs`.
- [ ] Task 2.3: Extract `Backup`, `Forget`, `StopSession`, and `Pin` commands into their respective modules.
- [ ] Task 2.4: Extract `Ingest` and `Safety` into `commands/ingest.rs` and `commands/safety.rs`.

### Phase 3: Command Extraction (Complex Commands)
- [ ] Task 3.1: Extract `Recall` into `commands/recall.rs`.
- [ ] Task 3.2: Extract `Preflight` into `commands/preflight.rs`.
- [ ] Task 3.3: Extract `Nightly` into `commands/nightly.rs`.

### Phase 4: Antigravity Logic Migration
- [ ] Task 4.1: Move Antigravity session discovery and import core logic from `main.rs` to `crates/ai-brains-adapters/src/antigravity.rs` (or similar in `ai-brains-brain`).
- [ ] Task 4.2: Create `crates/ai-brains-cli/src/commands/antigravity_import.rs`.
- [ ] Task 4.3: Refactor the CLI command handler to delegate to the new adapter/brain function using `AppContext`.

### Phase 5: Cleanup and Verification
- [ ] Task 5.1: Remove residual dead code and unused imports in `crates/ai-brains-cli/src/main.rs`.
- [ ] Task 5.2: Ensure all sub-modules use `AppContext`.
- [ ] Task 5.3: Run `cargo fmt`, `cargo clippy`, and `cargo nextest run` to verify no regressions.
- [ ] Task 5.4: Run `changeguard scan` and record provenance if needed.