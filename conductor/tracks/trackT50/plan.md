## Plan: CozoDB Bridge Hardening & Unification (Track T50)

### Phase 1: Batch Mutation & Datalog Escaping
- [ ] Task 1.1: Implement `escape_datalog_string` to securely handle Unicode, control characters, and Datalog delimiters. Add exhaustive unit tests.
- [ ] Task 1.2: Update the `GraphBackend` trait in `ai-brains-graph` to include `add_nodes(nodes: &[Node])` and `add_edges(edges: &[Edge])`.
- [ ] Task 1.3: Implement the batch APIs in `CozoProxyBackend` by buffering inputs into a single Datalog `:put` query per batch, utilizing a single IPC invocation.

### Phase 2: Handshake & Error Handling
- [ ] Task 2.1: Integrate the Track T44 ultra-fast handshake into `CozoProxyBackend` initialization to verify bridge health in <10ms before executing batches.
- [ ] Task 2.2: Update `send_datalog_mutation` and `run_datalog_query` in `CozoProxyBackend` to parse `ApiResult` JSON payloads.
- [ ] Task 2.3: Map structured `ApiResult` errors to native `ai_brains_graph::Error` variants, ensuring strict adherence to the zero-panic mandate.

### Phase 3: Projector Unification
- [ ] Task 3.1: Create a `MultiplexGraphBackend` struct that accepts multiple `Box<dyn GraphBackend>` instances and broadcasts operations to all.
- [ ] Task 3.2: Update `GraphProjector` to instantiate and use a multiplexer containing both `SqliteGraphBackend` and `CozoProxyBackend`.
- [ ] Task 3.3: Refactor the `GraphProjector` replay and build logic to accumulate nodes and edges in memory and flush them using the new batch mutation API.

### Phase 4: Integration & Testing
- [ ] Task 4.1: Write integration tests to simulate event log replays, asserting correct projection behavior across both SQLite and CozoDB backends.
- [ ] Task 4.2: Add instrumentation and tests to verify that batch operations invoke the `changeguard` IPC process exactly once per batch.
- [ ] Task 4.3: Ensure passing workspace check: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`.
