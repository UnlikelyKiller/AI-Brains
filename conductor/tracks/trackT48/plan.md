## Plan: Automated Project Mapping

### Phase 1: Event and Projection Definition
- [ ] Task 1.1: Define the `ProjectAliasAdded` event in `ai-brains-contracts` and `ai-brains-events`. Ensure serialization/deserialization tests pass.
- [ ] Task 1.2: Create a new SQL migration in `ai-brains-store` to create the `project_alias_projection` table (`alias` TEXT UNIQUE, `project_id` TEXT).
- [ ] Task 1.3: Update the projection builder logic to handle `ProjectAliasAdded` events and populate the `project_alias_projection` table.

### Phase 2: AppContext and Core Logic Updates
- [ ] Task 2.1: Add `resolve_project_id_from_alias` to the `QueryStore` trait and implement it for the SQLite backend.
- [ ] Task 2.2: Add auto-link logic to `AppContext` (or equivalent service): If an alias is not found but `AI_BRAINS_PROJECT_ID` is present in the environment context, return the ID and generate/dispatch a `ProjectAliasAdded` event.

### Phase 3: Integration and Testing
- [ ] Task 3.1: Modify `agy-hook` handling in `ai-brains-cli` to extract `projectHash` and call the resolution logic before building the `IngestRequest`.
- [ ] Task 3.2: Modify `antigravity-import` to apply the same resolution pipeline.
- [ ] Task 3.3: Write an integration test proving that importing an Antigravity log with a new `projectHash` correctly triggers a `ProjectAliasAdded` event when a project ID is in the environment, and subsequent imports use the cached mapping.