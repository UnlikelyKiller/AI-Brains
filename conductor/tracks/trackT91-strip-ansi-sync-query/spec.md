# Track T91: Strip ANSI Escape Codes Before Ledger Search in `sync query`

**Status:** ⏳ **Pending**
**Owner:** —
**Priority:** P2 — ledger search always returns 0 results; query is ANSI-contaminated.

---

## Problem Statement

`ai-brains sync query <term>` passes the query to the ChangeGuard ledger search with ANSI escape codes embedded in the string. Observed output:

```
No ledger entries found matching '[33mcontext brittle hotspot[39m'
```

The `[33m` and `[39m` sequences are color escape codes that have leaked into the query string before it is forwarded to `changeguard`. ChangeGuard ledger search never finds any results because it is literally searching for colored text.

## Root Cause

Somewhere in the `sync query` call chain — likely when formatting the query for display and then reusing the formatted string, or when reading from a colored output — ANSI escape codes are being included in the string that gets passed to the changeguard ledger search subcommand.

## Acceptance Criteria

**AC1:** `ai-brains sync query "context brittle hotspot"` forwards a plain, ANSI-free query string to ChangeGuard ledger search.

**AC2:** The ledger search returns actual results when matching entries exist in the ChangeGuard ledger.

**AC3:** No other output is affected — the display portion of `sync query` may still use color.

**AC4:** The fix uses `strip_ansi_escapes` (already a workspace dependency) or equivalent; no new deps required.

## Design Notes

- Locate the point in `crates/ai-brains-cli/src/commands/sync.rs` where the query string is assembled for the changeguard ledger subcommand call.
- Apply `strip_ansi_escapes::strip_str(&query)` (or `String::from_utf8(strip_ansi_escapes::strip(query.as_bytes()).unwrap_or_default()).unwrap_or(query)`) to the string before it is passed as an argument.
- The T90 FTS5 sanitization should be applied after ANSI stripping so the two transforms compose correctly.

## Verification

```
changeguard ledger                                   # note a recent entry keyword
ai-brains sync query "<that keyword>"                # must find the ledger entry
# output must NOT contain '[33m' in the "matching '...'" portion of the message
```
