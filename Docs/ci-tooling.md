# CI Tooling — Installation & Version Pins (T71)

This document records the supported installation paths and version pins for the tools required by the AI-Brains CI gate. Updated whenever a tool is intentionally upgraded.

## Required Tools

| Tool | Minimum Version | Install Command |
|------|----------------|-----------------|
| `cargo-nextest` | 0.9.137 | `cargo install cargo-nextest --locked` |
| `cargo-deny` | 0.19.4 | `cargo install cargo-deny --locked` |
| `cargo-audit` | 0.22.1 | `cargo install cargo-audit --locked` |

All three tools install to `~/.cargo/bin/` via standard `cargo install`. No project-local binaries or generated caches are used.

## Full CI Gate

Run from the repository root:

```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
```

Or use the verification script, which checks tool presence and versions before running the gate:

```powershell
.\scripts\dev-check.ps1
```

Pass `--check-only` to verify tool presence without running the full gate:

```powershell
.\scripts\dev-check.ps1 --check-only
```

## Windows App Control Notes

- `cargo-deny` and `cargo-nextest` must be installed via `cargo install` (MSVC or GNU toolchain). Pre-built binaries from third-party sources may be blocked by Windows Application Control.
- If `cargo-deny` is blocked (OS error 4551), uninstall it and reinstall via `cargo install cargo-deny --locked`.
- No special execution policy changes are required for `scripts/dev-check.ps1` if run within the project shell.

## Upgrading a Tool

1. Run `cargo install <tool> --locked` with the new version.
2. Verify the full gate still passes: `.\scripts\dev-check.ps1`
3. Update the version pin table above and in `scripts/dev-check.ps1` (`$Required` hash).

## Behavior Notes

### `cargo audit` exits 0 with no final summary on a clean run

`cargo-audit` 0.22.x changed its CLI output — a clean run now exits 0 but
emits **no final summary line**. The visible output ends with
`Scanning Cargo.lock for vulnerabilities (N crate dependencies)`. To a casual
reader, that looks like a hang that exited 0.

How to interpret:

- Exit 0 + tail `Scanning …` → success, no vulnerabilities found.
- Exit 0 + any text after `Scanning …` (a `warning` or `error:` block) →
  success with informational warnings.
- Exit non-zero → real failure; the message before exit code is the cause.

To get an explicit confirmation in scripts or CI logs, use the JSON output:

```powershell
cargo audit --json
# => {"database":{...},"lockfile":{"dependency-count":N},"vulnerabilities":{"found":false,"count":0,"list":[]},"warnings":{}}
```

The JSON envelope's `vulnerabilities.count` is the authoritative answer.

This quirk is what made the early T71 verification confusing. The
`scripts\dev-check.ps1` script treats exit 0 as success, which is correct —
just be aware that the human-readable form gives no positive confirmation.
