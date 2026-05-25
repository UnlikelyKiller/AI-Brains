# Track T54: Bridge Stderr Hardening

## Objective
Further reduce noise in the ChangeGuard bridge path by suppressing transient stderr output from child processes when operating in `--quiet` mode.

## Problem Statement
While T51 suppressed AI-Brains' own "daemon down" messages, the `sync query` command still prints stderr from the `changeguard` CLI (e.g., `CozoDB: unable to open database file`). These errors are often transient (due to file locks) and can confuse the user into thinking AI-Brains itself is failing.

## Requirements
- **Stderr Suppression**: When `--quiet` is enabled, the CLI should redirect or filter stderr from all child process invocations (`changeguard bridge export`, `changeguard ledger search`, etc.).
- **Smart Filtering**: Allow critical errors through while suppressing known transient noise (locks, unreachable daemons).

## Technical Design
- Update `run_query` and `run_pull` to check the `quiet` flag before spawning child processes.
- If `quiet`, set `.stderr(Stdio::null())` or pipe it and only print if the error code is not among the suppressed set.

## Verification Plan
- Trigger a scenario where ChangeGuard would error (e.g., lock its DB).
- Run `ai-brains sync query "test" --quiet`.
- Verify no error output is visible in the terminal.
