# Track T08 — Git Metadata

## Owner
architecture-planner

## Status
Completed

## Objective
Implement bounded Git metadata capture in the `ai-brains-git` crate so higher layers can identify repository context without storing full diffs or other oversized/sensitive Git output.

## Scope
- Discover the repository root from an arbitrary working directory.
- Capture current branch and commit SHA when available.
- Detect dirty state from porcelain status output.
- Capture bounded untracked filenames.
- Derive a stable hash of the remote URL without storing the raw URL.
- Capture bounded diff statistics without persisting full diff content.

## Out of Scope
- Full diff capture or patch persistence.
- Git write operations.
- Advanced history analysis, blame, or commit graph traversal.
- Security classification beyond bounded metadata shape.

## Files Owned
`crates/ai-brains-git/*`

## Files Allowed To Touch
`crates/ai-brains-git/src/lib.rs`
`crates/ai-brains-git/src/discover.rs`
`crates/ai-brains-git/src/branch.rs`
`crates/ai-brains-git/src/commit.rs`
`crates/ai-brains-git/src/remote.rs`
`crates/ai-brains-git/src/status.rs`
`crates/ai-brains-git/src/diffstat.rs`
`crates/ai-brains-git/src/command.rs`
`crates/ai-brains-git/src/errors.rs`
`crates/ai-brains-git/tests/*.rs`
`crates/ai-brains-git/Cargo.toml`
`Docs/conductor/trackT08/spec.md`
`Docs/conductor/trackT08/plan.md`
`Docs/conductor/conductor.md`
`tracks/T08-git-metadata.md`
`Docs/status.md`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-git/` and the conductor/status planning docs.
Must NOT touch `ai-brains-path`, `ai-brains-security`, `ai-brains-store`, `ai-brains-cli`, or `ai-brainsd` while executing this track.

## Public Contracts Consumed
- Rust standard library process and path APIs.
- `thiserror` for typed errors.
- `sha2` for stable remote URL hashing.

## Public Contracts Produced
- Repository discovery helpers.
- Bounded Git metadata summary API.
- Stable diffstat and status parsing helpers.

## Required Tests First
- `tests/git_root_discovered.rs`
- `tests/non_git_directory_degrades.rs`
- `tests/remote_url_hash_stable.rs`
- `tests/branch_detected.rs`
- `tests/commit_detected.rs`
- `tests/dirty_status_detected.rs`
- `tests/diffstat_does_not_capture_full_diff.rs`
- `tests/untracked_filenames_bounded.rs`

## Implementation Steps
1. Replace the crate stub with module structure, typed errors, and a bounded metadata surface.
2. Implement repository discovery and Git command execution helpers.
3. Implement branch, commit, remote hash, dirty status, untracked filename, and diffstat readers.
4. Add the required tests using temporary local Git repositories.
5. Verify with crate tests, nextest, and clippy.

## Failure Modes To Handle
- Directory is not inside a Git repository.
- Git command unavailable or exits non-zero for detached/empty repo states.
- Untracked file lists grow unbounded.
- Diff collection accidentally captures full patch content instead of summary counts.

## Security Requirements
- Never persist raw full diff output.
- Never require network access.
- Remote URLs must be hashed before exposure from the public API.
- Non-Git directories must degrade gracefully instead of panicking.

## Acceptance Criteria
- Repository root, branch, commit, dirty state, remote hash, diffstat, and bounded untracked names are available when present.
- Non-Git directories return an empty/degraded metadata result rather than a hard failure.
- No full diff text is exposed from the crate API.
- All required tests pass.

## Commands To Run
`cargo test -p ai-brains-git`
`cargo clippy -p ai-brains-git --all-targets -- -D warnings`

## Handoff Notes
This crate should remain a bounded metadata provider so T09 security scanning and later capture logic can consume it safely without inheriting raw Git content.
