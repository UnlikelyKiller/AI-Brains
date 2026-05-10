# Specification: T08 Git Metadata

## 1. Overview
This specification covers bounded repository metadata capture in the `ai-brains-git` crate. The goal is to provide enough repository identity for capture and recall without storing raw patch text or other oversized Git output.

## 2. Dependencies
- Rust standard library path and process handling
- `thiserror` for typed errors
- `sha2` for stable remote URL hashing

## 3. Required Modules
- `src/discover.rs`
- `src/branch.rs`
- `src/commit.rs`
- `src/remote.rs`
- `src/status.rs`
- `src/diffstat.rs`
- `src/command.rs`
- `src/errors.rs`

## 4. Metadata Rules
- Repository root should be discovered via bounded Git commands.
- Branch and commit should be captured when available.
- Dirty state should come from porcelain status parsing.
- Untracked filenames must be bounded to a fixed list size.
- Remote URL must be hashed before leaving the crate.
- Diff information must be summary-only and must not contain raw diff hunks.

## 5. API Shape
The crate should expose a compact metadata summary object and focused helper functions for individual fields. Non-Git directories should degrade to an empty metadata result instead of failing the whole call.

## 6. Test Expectations
The crate must satisfy:
- `git_root_discovered`
- `non_git_directory_degrades`
- `remote_url_hash_stable`
- `branch_detected`
- `commit_detected`
- `dirty_status_detected`
- `diffstat_does_not_capture_full_diff`
- `untracked_filenames_bounded`

## 7. Constraints
- No full diff capture.
- No Git write commands in production code.
- No panic-based control flow.
- Keep the API deterministic and bounded for downstream capture consumers.
