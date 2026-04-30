# AI-Brains Failure Drills

This document defines the standard failure drills for AI-Brains. These drills ensure that the system remains resilient, encrypted, and rebuildable under realistic failure conditions.

## 1. Capture Path Failure Drills

| Drill ID | Scenario | Expected Result | Verification |
|---|---|---|---|
| F-CAP-01 | Kill daemon mid-ingest | CLI returns a retryable error or spools the event locally. No data corruption in the vault. | Run `ai-brains ingest`, kill `ai-brainsd` process during write, verify `changeguard scan --impact` is LOW. |
| F-CAP-02 | Kill process during session | Session marked as `interrupted` or `unknown` on next health scan. | Start session, kill harness process, run `ai-brains sessions`, verify status. |
| F-CAP-03 | Graph service unavailable | Capture succeeds. Retrieval falls back to lexical search. Health reports degraded graph. | Disable graph backend, run `ai-brains ingest`, verify capture success. |
| F-CAP-04 | Model provider unavailable | Nightly sweep continues for other projects. Partial failure recorded. | Block Ollama/Llama port, run `ai-brains nightly`, verify logs. |
| F-CAP-05 | Wrong vault key | No plaintext fallback. Actionable error reporting the locked state. | Provide invalid key, run `ai-brains doctor`, verify error message. |

## 2. Recovery Drills

| Drill ID | Scenario | Expected Result | Verification |
|---|---|---|---|
| F-REC-01 | Delete active vault | Events recovered from most recent encrypted backup. | Delete `vault.db`, run `ai-brains backup restore`, verify turnover. |
| F-REC-02 | Restore on new machine | Recovery kit successfully unlocks the vault on a fresh OS install. | Install AI-Brains on new test user, use recovery kit, verify access. |
| F-REC-03 | Corrupt projection | Read side rebuilt entirely from the append-only event log. | Delete projection tables, run `ai-brains service rebuild`, verify data parity. |
| F-REC-04 | Corrupt graph | Graph projection rebuilt from event log. | Delete graph file, run `ai-brains service rebuild --graph`, verify traversal. |

## 3. Security Drills

| Drill ID | Scenario | Expected Result | Verification |
|---|---|---|---|
| F-SEC-01 | Repo hook install | Try to install hooks in project scope without `--force`. Rejected by default. | Run `ai-brains install-hooks --project .`, verify rejection. |
| F-SEC-02 | Cloud leak prevention | Try to send `local_only` memory to cloud provider. Blocked by privacy gate. | Configure cloud model, attempt recall on secret, verify blocking. |
| F-SEC-03 | Sealed memory isolation | Sealed memory excluded from preflight and automatic summaries. | Mark turn as sealed, run `ai-brains preflight`, verify omission. |
| F-SEC-04 | Tool-log exclusion | Attempt to store raw tool calls. Ignored by capture core. | Send JSON with `tool_calls` to ingest, verify omission in turn projection. |

## 4. Execution Protocol

Drills should be run:
1.  **Phase Gates:** Before closing a major implementation phase.
2.  **Release Preflight:** Before any versioned release.
3.  **Ad-hoc:** When significant changes are made to the daemon, store, or security layers.
