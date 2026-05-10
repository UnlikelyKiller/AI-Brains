# Track T07 — Path Normalization

## Owner
architecture-planner

## Status
Completed

## Objective
Implement Windows-first path normalization in the `ai-brains-path` crate so project identity is stable across drive-case differences, slash styles, WSL `/mnt/<drive>` mappings, UNC paths, and extended-length prefixes.

## Scope
- Normalize Windows drive letters and slash styles into a canonical project path.
- Map WSL `/mnt/<drive>/...` paths to canonical Windows form.
- Normalize extended-length `\\?\` paths without losing path meaning.
- Preserve UNC paths while applying safe normalization rules.
- Resolve symlinks and aliases on a best-effort basis without making canonicalization dependent on filesystem mutability.
- Provide stable display formatting distinct from canonical storage when needed.

## Out of Scope
- Git metadata enrichment.
- Security scanning or privacy escalation.
- Capture, daemon, or adapter integration beyond consuming the normalization API.
- Network path discovery beyond safe string and path normalization rules.

## Files Owned
`crates/ai-brains-path/*`

## Files Allowed To Touch
`crates/ai-brains-path/src/lib.rs`
`crates/ai-brains-path/src/canonical.rs`
`crates/ai-brains-path/src/display.rs`
`crates/ai-brains-path/src/windows.rs`
`crates/ai-brains-path/src/wsl.rs`
`crates/ai-brains-path/src/unc.rs`
`crates/ai-brains-path/src/symlink.rs`
`crates/ai-brains-path/src/alias.rs`
`crates/ai-brains-path/src/project_path.rs`
`crates/ai-brains-path/src/errors.rs`
`crates/ai-brains-path/tests/*.rs`
`crates/ai-brains-path/Cargo.toml`
`Docs/conductor/trackT07/spec.md`
`Docs/conductor/trackT07/plan.md`
`Docs/conductor/conductor.md`
`tracks/T07-path-normalization.md`
`Docs/status.md`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-path/` and the conductor/status planning docs.
Must NOT touch `ai-brains-git`, `ai-brains-security`, `ai-brains-store`, `ai-brains-cli`, or `ai-brainsd` while executing this track.

## Public Contracts Consumed
- Rust standard library path types.
- `thiserror` for library errors.

## Public Contracts Produced
- A canonical project-path normalization API.
- Stable display-path helpers.
- Error types for malformed or unsupported path inputs.

## Required Tests First
- `tests/windows_drive_case_normalized.rs`
- `tests/forward_slashes_normalized.rs`
- `tests/wsl_mnt_c_maps_to_windows.rs`
- `tests/extended_length_prefix_normalized.rs`
- `tests/unc_paths_preserved.rs`
- `tests/symlink_resolution_best_effort.rs`
- `tests/display_path_preserved.rs`
- `tests/malformed_paths_return_error_not_panic.rs`

## Implementation Steps
1. Configure `ai-brains-path` dependencies and replace the crate stub with module structure.
2. Implement canonical normalization rules for Windows drive letters, separators, extended-length prefixes, and WSL mappings.
3. Add UNC and display-path handling with explicit separation between canonical storage and presentation.
4. Add best-effort symlink and alias handling that never panics on malformed inputs.
5. Write the required tests and verify clippy and test passes for the crate.

## Failure Modes To Handle
- Malformed WSL mount paths.
- Mixed slash and drive-case variants resolving to inconsistent canonical forms.
- Extended-length and UNC prefixes being stripped incorrectly.
- Symlink resolution failures causing hard errors or panics when best-effort handling should degrade safely.

## Security Requirements
- No path normalization routine may panic on malformed input.
- Canonicalization must not perform unsafe repo writes or shell execution.
- Path identity must remain deterministic across equivalent Windows and WSL representations.

## Acceptance Criteria
- Equivalent Windows and WSL paths normalize to a single canonical project identity.
- UNC paths remain valid and stable after normalization.
- Display formatting preserves user-meaningful structure where required.
- All required tests pass.

## Commands To Run
`cargo test -p ai-brains-path`
`cargo clippy -p ai-brains-path --all-targets -- -D warnings`

## Handoff Notes
Path normalization must remain capture-safe and free of git/security logic so T08 and T09 can consume it cleanly.
