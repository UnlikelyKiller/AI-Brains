# Track T90: FTS5 Query Sanitization

**Status:** ⏳ **Pending**
**Owner:** —
**Priority:** P1 — crashes `sync query` on common inputs; may also affect internal recall FTS.

---

## Problem Statement

Queries containing FTS5 special characters cause `changeguard search` and the internal SQLite FTS5 engine to return a syntax error and abort. Observed example:

```
changeguard search --json "context.rs brittle hotspot"
→ fts5: syntax error near "."
```

Characters that trigger FTS5 parse errors: `.` `*` `"` `(` `)` `:` `^` `-` (when leading).

This affects:
- `sync query <term>` — passes raw user input to `changeguard search`
- `recall <query>` — passes raw user input to the internal FTS5 vault search

## Acceptance Criteria

**AC1:** `sync query "context.rs brittle hotspot"` does not crash with an FTS5 syntax error; it returns whatever results FTS5 can match after sanitization.

**AC2:** `recall "some.method(arg)"` does not crash with an FTS5 syntax error.

**AC3:** Sanitization wraps each token in double-quotes (FTS5 phrase quoting), or strips non-alphanumeric characters that have no safe FTS5 representation. The chosen approach must be documented in a comment.

**AC4:** The sanitization function is a single shared utility used by both the bridge query path and the internal FTS path — not duplicated.

**AC5:** A unit test covers: bare dot, parentheses, asterisk, and a mixed query string.

## Design Notes

- Simplest safe approach: split the query on whitespace, wrap each token in double-quotes, rejoin with spaces: `"context.rs" "brittle" "hotspot"` — FTS5 treats quoted tokens as phrase literals, not operator syntax.
- Alternatively, strip non-word characters: `context rs brittle hotspot`. Less precise but avoids all FTS5 operator chars.
- The bridge path in `crates/ai-brains-retrieval/src/recall.rs` (`query_changeguard_bridge`) and the internal FTS path should both call this shared helper.
- `sync query` lives in `crates/ai-brains-cli/src/commands/sync.rs` — it constructs the query string before calling the bridge; sanitize there too.

## Verification

```
ai-brains sync query "context.rs brittle hotspot"   # must not error
ai-brains recall "some.method(arg)"                 # must not error
cargo nextest run --workspace                        # all tests pass
```
