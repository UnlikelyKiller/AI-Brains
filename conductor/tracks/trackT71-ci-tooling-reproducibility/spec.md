# Track T71: CI Tooling Reproducibility

**Status:** Complete
**Started:** 2026-05-31
**Owner:** Claude
**Parent:** T60 (MinGW-w64 Toolchain)
**Priority:** High

---

## Problem Statement

The project CI gate requires:

```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
```

Local verification cannot currently complete that exact gate because `cargo-nextest` is not installed and Windows Application Control blocks the existing `cargo-deny.exe` binary with OS error 4551.

This leaves otherwise-green repository changes unable to prove the full required gate from a clean Windows workstation.

---

## Acceptance Criteria

**AC1:** Document the supported installation path and version pins for `cargo-nextest`, `cargo-deny`, and `cargo-audit`.

**AC2:** Provide a PowerShell verification script that checks tool presence, versions, Windows execution policy/App Control compatibility, and exits non-zero with actionable remediation.

**AC3:** Ensure `cargo deny check` can execute on the target Windows workstation without App Control rejection.

**AC4:** Ensure `cargo nextest run --workspace` executes locally and produces the expected workspace test results.

**AC5:** Update project onboarding or tooling docs with the validated setup so new agents can reproduce the CI gate without ad hoc fixes.

---

## Design Notes

- Prefer user-global installation and configuration. Do not add project-local binaries or generated caches.
- Keep the script read-only by default. It should diagnose first and only install tools behind an explicit user-invoked command.
- If App Control requires binary allowlisting or reinstall from source, document the exact supported path.

---

## Verification

Run from the repository root:

```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
```

The track is complete only when every command exits successfully on the target Windows environment.
