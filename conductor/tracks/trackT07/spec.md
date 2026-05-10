# Specification: T07 Path Normalization

## 1. Overview
This specification covers the implementation of canonical path identity in the `ai-brains-path` crate. The project is Windows-first, but it must also recognize equivalent WSL forms so `C:\dev\Project`, `c:/dev/project`, and `/mnt/c/dev/project` can map to a stable identity when appropriate.

## 2. Dependencies
- Rust standard library path handling
- `thiserror` for typed error handling

## 3. Required Modules
- `src/canonical.rs`
- `src/display.rs`
- `src/windows.rs`
- `src/wsl.rs`
- `src/unc.rs`
- `src/symlink.rs`
- `src/alias.rs`
- `src/project_path.rs`
- `src/errors.rs`

## 4. Canonicalization Rules
- Windows drive letters must normalize consistently regardless of input case.
- Forward slashes and backslashes must converge to the same canonical Windows form.
- WSL `/mnt/<drive>/...` paths must map to Windows drive paths.
- Extended-length prefixes such as `\\?\` must normalize without corrupting the underlying path.
- UNC paths must remain UNC paths after normalization.
- Symlink and alias handling should be best-effort and must not panic when resolution fails.

## 5. API Shape
The crate should expose a small, deterministic API that:
1. Accepts a raw path input.
2. Returns a canonical project path representation or a typed error.
3. Supports display formatting separately from canonical storage formatting.

## 6. Test Expectations
The crate must satisfy:
- `windows_drive_case_normalized`
- `forward_slashes_normalized`
- `wsl_mnt_c_maps_to_windows`
- `extended_length_prefix_normalized`
- `unc_paths_preserved`
- `symlink_resolution_best_effort`
- `display_path_preserved`
- `malformed_paths_return_error_not_panic`

## 7. Constraints
- No `unwrap`, `expect`, or `panic` in production code.
- No filesystem writes.
- No Git, daemon, or capture logic in this crate.
- Keep canonicalization deterministic and side-effect-light so higher layers can rely on it.
