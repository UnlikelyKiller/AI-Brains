# AI-Brains Product Requirements Document

**Product name:** AI-Brains
**Document status:** Draft v1.0
**Primary platform:** Windows 11, PowerShell-first
**Secondary platform:** Ubuntu / WSL, best-effort after Windows reliability is proven
**Product type:** Local-first CLI + optional local daemon for AI harness conversation memory
**Core directive:** Capture clean user/AI conversation memory across AI harnesses without storing hidden thinking, tool/action noise, or terminal sludge.

---

## 1. Executive Summary

AI-Brains is a Windows 11-first memory system for AI coding and planning harnesses such as Gemini CLI, Claude Code, Claude Code launched through Ollama, Codex CLI, OpenCode, and degraded/wrapper capture for Antigravity.

The product solves a practical problem: modern AI coding tools are powerful but forgetful. Sessions get compacted, usage windows run out, different models are used for implementation versus review, IDEs lose conversations, and parallel AI agents working in the same repo do not know what the others already did.

AI-Brains provides a durable, queryable, project-aware memory layer that records the parts worth preserving:

```text
User: Do X
AI final response: I did X
```

It intentionally excludes hidden thinking, noisy intermediate tool actions, and chain-of-thought-like text. It stores final answers, user prompts, failure/abort markers, project/session metadata, Git state, summaries, embeddings, graph relationships, and derived durable lessons.

The system operates as a CLI and local daemon, not as an MCP server. Harnesses may call the CLI directly through hooks, wrappers, or scripts. The CLI returns JSON-first context optimized for AI injection.

---

## 2. Product Thesis

A useful AI memory system must be boringly reliable at capture time and smart later.

The capture path should be fast, append-only, encrypted, crash-resilient, and independent of model availability. The intelligence path can be slower and scheduled: summarization, embeddings, graph edges, conflict detection, RAPTOR clustering, CRAG-style evaluation, and preflight context synthesis.

This produces two operating modes:

| Mode         | Purpose                                             |              Timing | Must work if models/graph are offline? |
| ------------ | --------------------------------------------------- | ------------------: | -------------------------------------: |
| Capture Mode | Preserve clean conversation events and metadata     |           Immediate |                                    Yes |
| Brain Mode   | Summarize, relate, embed, cluster, validate, inject | Scheduled/on-demand |                                     No |

The product should make AI harnesses better without requiring the user to commit repo-local memory files, manually update project docs after every session, or remember which model did what yesterday.

---

## 3. Target User

### Primary User

A Windows-based developer, lawyer-builder, or power user who:

* Uses multiple AI harnesses interchangeably.
* Runs several concurrent AI sessions against the same project.
* Uses fresh sessions for review/audit after implementation.
* Frequently loses context because of compaction, usage windows, crashes, or IDE conversation loss.
* Works primarily under `C:\dev\ProjectName`.
* Needs local-first control but may optionally use cloud models for non-sensitive memory processing.

### Secondary Future User

A small team or enterprise buyer that wants:

* Shared project memory.
* Per-user/private memory.
* Auditability of AI-assisted work.
* Privacy controls for local-only versus cloud-safe processing.
* Commercially usable open-source or permissively licensed components.

---

## 4. Core Problems

### 4.1 Harness amnesia

Each harness has its own session memory, compaction behavior, context window, and log format. Valuable final answers disappear after window exhaustion or tool crashes.

### 4.2 Parallel AI isolation

Two AI agents may work on `C:\dev\project1` while another works on `C:\dev\project2`. Without a shared memory layer, they duplicate work, contradict each other, or miss fresh context.

### 4.3 Review-agent blindness

A fresh model used for review often lacks the implementation trail. The reviewer needs a compact, project-scoped history of what happened, what files changed, what decisions were made, and which risks remain.

### 4.4 IDE/harness log fragility

Some environments, especially Antigravity in the user’s current setup, cannot reliably recover closed conversations. AI-Brains cannot depend on IDE logs as the source of truth.

### 4.5 Windows path and shell weirdness

AI agents regularly mishandle Windows paths, PowerShell quoting, here-strings, `cmd /c`, WSL `/mnt/c`, symlinks, junctions, and workspace-relative dynamic imports. These should become durable execution knowledge instead of being rediscovered.

### 4.6 Repo pollution

The system should not write memory files into projects by default. It should identify projects from the working directory/Git metadata and store memory globally under the user profile.

### 4.7 Sensitive data risk

AI sessions may include legal, proprietary, credential-adjacent, or client-specific content. Storage must be encrypted. Derived memories must inherit privacy restrictions from source material.

---

## 5. Goals

| Goal                        | Description                                                                                                     | Success Signal                                                 |
| --------------------------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Clean conversation capture  | Store user prompts and final AI responses, not hidden thinking or tool noise.                                   | Search returns useful session history without sludge.          |
| Multi-harness support       | Support Claude Code, Gemini CLI, Codex CLI, Claude Code via Ollama, OpenCode, and degraded Antigravity capture. | User can switch harnesses without losing project memory.       |
| Multi-instance awareness    | Track concurrent AI sessions by project, cwd, harness, model, branch, commit, and user.                         | A new AI instance can learn what another instance just did.    |
| Project-aware preflight     | Inject relevant project/global context at session start.                                                        | AI begins with useful 1500-word-or-less context.               |
| Mid-session recall          | Let AIs query AI-Brains for compacted/lost/cross-agent context.                                                 | AI can recover prior decisions without asking the user.        |
| Nightly memory intelligence | Run 3am summarization, embeddings, graph relationships, RAPTOR, CRAG, stale checks, and conflict detection.     | Daily project summaries and durable lessons improve over time. |
| Windows reliability         | Normalize paths and preserve proven Windows execution recipes.                                                  | Fewer repeated Windows-specific AI failures.                   |
| Commercial path             | Use active, permissively licensed or commercially safe dependencies.                                            | Product can become commercial without a license rewrite.       |

---

## 6. Non-Goals

| Non-Goal                       | Reason                                                                    |
| ------------------------------ | ------------------------------------------------------------------------- |
| MCP server v1                  | User explicitly wants CLI, not MCP. MCP can be future optional.           |
| Raw terminal recorder          | Too much noise and too many secrets.                                      |
| Hidden thinking capture        | Must not store private reasoning or chain-of-thought-like text.           |
| Default repo modification      | Avoid polluting repos and avoid accidental commits of memory files.       |
| Full-diff retention by default | Diffs may contain secrets, legal content, or high-volume noise.           |
| Cloud-first architecture       | Local-first and offline capture are mandatory.                            |
| Perfect Antigravity capture    | Antigravity is treated as degraded due to unreliable recoverability/logs. |

---

## 7. Operating Principles

1. **Capture first, think later.** Never let summarization, embeddings, graph writes, or model failures block capture.
2. **Final-answer memory beats action logs.** Store the meaningful conversation result, not every tool step.
3. **Append-only canonical store.** Raw events should be durable and recoverable.
4. **Derived data is rebuildable.** Graph projections, embeddings, summaries, and clusters can be regenerated.
5. **Project identity is normalized.** `C:\dev\Project`, `c:/dev/project`, `/mnt/c/dev/project`, symlinks, and aliases should resolve to one project when appropriate.
6. **Privacy propagates.** Derived summaries inherit the strictest privacy flag from their source memories.
7. **Human and AI outputs differ.** CLI output defaults to JSON for AI consumption but can render human-friendly markdown.
8. **No repo writes unless explicitly requested.** Global user storage is the default.
9. **Commercial-safe dependencies only.** Avoid SSPL, AGPL, GPL contamination, BSL restrictions, and abandoned critical components where practical.
10. **Windows is not a port.** Windows 11 is the primary product environment.

---

## 8. System Overview

```text
AI Harnesses
  Claude Code hooks
  Gemini CLI hooks
  Codex CLI hooks
  Claude Code via Ollama wrapper
  OpenCode adapter
  Antigravity wrapper/manual degraded adapter
        |
        v
AI-Brains CLI / Hook Commands
  ingest-turn
  ingest-final
  preflight
  recall
  context
  sessions
        |
        v
ai-brainsd Local Daemon
  single writer queue
  encryption/unlock
  path normalization
  project registry
  session registry
  task scheduler coordination
        |
        v
Canonical Encrypted Store
  SQLCipher-backed SQLite
  append-only events
  turns/sessions/projects/users/orgs
        |
        v
Brain Processing
  Nomic embeddings
  Relational SQLite graph projection
  FTS lexical search
  hybrid retrieval
  daily summaries
  durable lessons
  conflicts
  RAPTOR clusters
  CRAG checks
        |
        v
Injection / Recall Output
  JSON result
  markdown_brief
  warnings
  relevant memories
  active sessions
```

---

## 9. Capture Strategy

AI-Brains uses adapters in this priority order:

1. **Native hooks** where the harness exposes session start, prompt submit, stop/final-response, or equivalent events.
2. **Wrapper command** where hooks are unavailable or insufficient.
3. **Transcript/log parser** where reliable logs exist.
4. **Manual/degraded capture** where the environment is not recoverable.

### 9.1 Required capture events

| Event                 |           Required? | Description                                                   |
| --------------------- | ------------------: | ------------------------------------------------------------- |
| Session start         |                 Yes | Harness, cwd, project, user, model, pid, timestamp.           |
| User prompt           | Yes where available | Store submitted user prompt.                                  |
| Final AI response     |                 Yes | Store final assistant response, not intermediate tool output. |
| Session stop          |                 Yes | Completed/failed/aborted/interrupted/unknown.                 |
| Crash recovery marker |                 Yes | Persist incomplete session state if process dies.             |

### 9.2 Optional capture metadata

| Metadata            | Default       | Notes                                                         |
| ------------------- | ------------- | ------------------------------------------------------------- |
| Tool calls          | Off           | Excluded by default.                                          |
| Action digest       | On when cheap | Derived from metadata, not raw tool logs.                     |
| Git diffstat        | On            | File names/counts/insertions/deletions.                       |
| Full diff           | Off           | Only local summarizer may inspect; do not persist by default. |
| Test result summary | On if obvious | pass/fail/unknown.                                            |
| Files touched       | On            | Useful for recall/review.                                     |
| Model name          | On            | Important because models are swapped often.                   |

### 9.3 Hidden thinking policy

AI-Brains must not store hidden chain-of-thought, hidden model reasoning, tool traces, or chain-of-thought-like visible text. If a harness visibly emits thinking-like text, the adapter should ignore it unless the text is part of the final user-visible answer.

---

## 10. Harness Adapter Requirements

### 10.1 Claude Code adapter

Priority: P0

Requirements:

* Install hook commands into user-level Claude Code settings, not repo settings.
* Capture session start, user prompt submit if available, stop/final assistant message, and subagent stop where available.
* Support normal `claude` invocations and Ollama-launched Claude Code, including commands shaped like:

```powershell
ollama launch claude --model glm-5.1:cloud -- --dangerously-skip-permissions
```

* Detect model name dynamically when possible.
* Handle hook re-entry/loop prevention.
* Never output ANSI/stderr sludge that breaks hook JSON parsing.

### 10.2 Gemini CLI adapter

Priority: P0

Requirements:

* Install user-level hooks/extensions only.
* Capture startup/preflight injection and stop/final-response events where available.
* Provide JSON hook output designed for direct model consumption.
* Avoid blocking the Gemini loop with long summarization work.

### 10.3 Codex CLI adapter

Priority: P0/P1

Requirements:

* Support Codex hook configuration when enabled.
* Capture user prompt and stop/final-response events.
* Support future hook expansion without hardcoding a brittle event list.
* Degrade gracefully if hooks are disabled or feature-gated.

### 10.4 OpenCode adapter

Priority: P1

Requirements:

* Prefer plugin/command integration where available.
* Fall back to wrapper capture.
* Avoid assuming Claude Code-compatible hook semantics.

### 10.5 Antigravity adapter

Priority: P1 degraded

Requirements:

* Do not rely on Antigravity’s internal conversation recovery.
* Provide wrapper/manual capture first.
* Support a user command to paste/import final response if automated capture is unreliable.
* Detect and warn when capture is incomplete.

---

## 11. Project Identity

AI-Brains resolves project identity from:

1. Current working directory.
2. Git root.
3. Git remote URL hash.
4. Explicit project aliases.
5. Optional manual project registration.

### 11.1 Path normalization

Required normalization examples:

```text
C:\dev\Project
c:\dev\project
C:/dev/Project
/mnt/c/dev/Project
\\?\C:\dev\Project
junction target
symlink target
```

### 11.2 Project aliasing

Users must be able to define aliases:

```powershell
ai-brains project alias add --project LegisAI --path C:\dev\LegisAI
ai-brains project alias add --project Newton --path C:\dev\newton-workspace
```

### 11.3 Duplicate repo handling

If the same Git remote exists at multiple paths, AI-Brains should ask the user once whether they are the same logical project. The decision is stored in the global config.

---

## 12. Privacy and Security Model

### 12.1 Storage location

Default storage root:

```powershell
$env:USERPROFILE\.ai-brains
```

Required directories:

```text
.ai-brains/
  config.toml
  vault/
  graph/
  backups/
  hooks/
  logs/
  cache/
  recovery/
```

### 12.2 Encryption

Use hybrid envelope encryption:

1. Generate one local Data Encryption Key.
2. Wrap it with Windows DPAPI for normal local unlock.
3. Wrap it with a user recovery passphrase/keyfile for disaster recovery.
4. Store encrypted recovery material in `.ai-brains/recovery/`.
5. Require the user to export/back up a recovery kit during initialization.

### 12.3 Crash and drive-failure recovery

Requirements:

* Append-only event writes must survive process crashes.
* Nightly encrypted snapshots must be generated.
* Recovery kit must allow restoring the vault on a different Windows install or replacement drive.
* `ai-brains doctor` must validate recoverability.

### 12.4 Privacy tiers

| Tier           | Meaning                                                           |
| -------------- | ----------------------------------------------------------------- |
| `cloud_ok`     | May be processed by configured cloud models.                      |
| `local_only`   | May only be processed by local models.                            |
| `never_inject` | Stored/searchable but never auto-injected.                        |
| `sealed`       | Encrypted, explicit recall only, no automatic summaries/clusters. |

Inheritance rule:

```text
memory_privacy = strictest(project_default, explicit_memory_flag, source_sensitivity)
```

Derived summaries, embeddings, graph edges, and clusters inherit the strictest privacy tier of their source memories.

### 12.5 Secret handling

The product should not silently discard user data by default, but it must detect likely secrets and flag them.

Requirements:

* Secret scanner before embedding.
* Mark memory as `local_only` or `sealed` if high-confidence secret-like content appears.
* Do not send flagged content to cloud processors.
* Provide `ai-brains redact`, `ai-brains seal`, and `ai-brains forget` commands.

---

## 13. Data Retention

Default retention:

| Data type                       |                    Default retention |
| ------------------------------- | -----------------------------------: |
| Raw prompt/final-response pairs |   Expire if not accessed for 90 days |
| Failed/aborted raw sessions     |   Expire if not accessed for 90 days |
| Session summaries               |                               1 year |
| Daily project summaries         |                              2 years |
| Durable lessons                 | Indefinite unless superseded/expired |
| Pinned memories                 |            Indefinite until unpinned |
| Graph/embedding projections     | Rebuildable; follow source retention |

Expiration means soft-delete first, hard-delete after a configurable grace period.

---

## 14. Search and Retrieval

AI-Brains retrieval combines:

* Exact lexical search.
* Semantic embedding search.
* Graph traversal.
* Project/session recency.
* Git branch/commit/file overlap.
* Memory confidence.
* Staleness downgrade.
* Pin boost.
* Privacy eligibility.

### 14.1 Ranking signals

| Signal                          |      Weight |
| ------------------------------- | ----------: |
| Same project                    |   Very high |
| Same Git branch                 |        High |
| Same touched files              |        High |
| Same dependency/tooling context |        High |
| Recent access/use               | Medium-high |
| Pinned memory                   |        High |
| High confidence                 | Medium-high |
| Stale/superseded                |   Downgrade |
| Global environment relevance    | Conditional |
| Same harness/model              |  Low-medium |

### 14.2 Staleness

Technical memories should be downgraded when:

* Dependency versions changed.
* Tool versions changed.
* Git repo changed materially.
* Memory has not been validated recently.
* Contradictory newer memory exists.

Stale memories should usually be warned and downgraded, not silently hidden.

---

## 15. Preflight Injection

### 15.1 Purpose

At session start, AI-Brains should provide the harness with a compact project briefing.

### 15.2 Trigger

* Harness startup hook.
* Wrapper launch.
* Manual command:

```powershell
ai-brains preflight --cwd . --format json
```

### 15.3 Output target

Default output is JSON with a `markdown_brief` field.

Maximum default brief length: **1500 words**.

### 15.4 Content

Preflight should include:

* Current project identity.
* Recent active sessions on the same project.
* Important unresolved issues.
* Recent completed work.
* Pinned project memories.
* Relevant global environment memories.
* Windows/path/shell quirks only when relevant.
* Staleness warnings.
* Privacy warnings if content was excluded.

### 15.5 Example output shape

```json
{
  "kind": "ai_brains_preflight",
  "project": "LegisAI",
  "cwd": "C:\\dev\\LegisAI",
  "generated_at": "2026-04-26T12:00:00Z",
  "confidence": "medium",
  "active_sessions": [],
  "warnings": [],
  "memories": [],
  "markdown_brief": "..."
}
```

---

## 16. Mid-Session Recall

AI harnesses should be able to call AI-Brains mid-session for lost or compacted context.

Recommended commands:

```powershell
ai-brains recall "what happened with the auth refactor?" --project . --format json
ai-brains context --project . --topic "recent implementation" --format json
ai-brains sessions --project . --since 7d --format json
ai-brains active --project . --format json
ai-brains conflicts --project . --format json
```

Output must be concise, structured, and injection-ready.

---

## 17. Nightly Heartbeat

### 17.1 Schedule

Default: 3:00 AM local time via Windows Task Scheduler.

Manual command:

```powershell
ai-brains nightly run
```

### 17.2 Workflow

For each project with new or recently accessed memory:

1. Load new raw turns and session metadata.
2. Summarize each session.
3. Summarize each project’s day.
4. Extract durable lessons.
5. Detect conflicts and contradictions.
6. Inspect Git diff summaries and, when locally allowed, full diffs.
7. Generate embeddings.
8. Build/update graph relationships.
9. Run RAPTOR clustering.
10. Run CRAG-style evaluation against retrieved evidence.
11. Downgrade stale memories.
12. Promote highly useful execution recipes.
13. Generate next-day project briefings.
14. Create encrypted backup snapshot.

### 17.3 Failure behavior

* If local model unavailable: skip summarization, keep raw captures.
* If graph unavailable: skip graph projection, keep canonical store.
* If embeddings unavailable: skip vector index, keep lexical search.
* If backup fails: raise high-priority warning.
* If one project fails: continue other projects.

---

## 18. Conflict Detection

AI-Brains should detect conflicts such as:

* Agent A says feature implemented; Agent B says feature removed.
* One session says tests pass; later session says tests fail.
* Two active agents edit overlapping files.
* A reviewer rejects implementation assumptions.
* A memory is superseded by a newer decision.

Conflict records should include:

```text
conflict_id
project_id
source_memory_ids
relationship_type
summary
severity
status: open | acknowledged | resolved | ignored
created_at
resolved_at
```

---

## 19. Execution Recipes

AI-Brains should promote problematic command patterns into reusable execution recipes.

Examples:

* PowerShell here-string into script utilities.
* `cmd /c` usage to force EOF for certain tools.
* WSL path conversion.
* Dynamic imports with absolute workspace paths.
* Playwright/Node/Python commands that require special environment variables.

Recipes should be retrieved only when relevant to the current project/tooling.

---

## 20. CLI Command Surface

### 20.1 Setup and service

```powershell
ai-brains init
ai-brains doctor
ai-brains service install
ai-brains service start
ai-brains service stop
ai-brains service status
ai-brains schedule install --time 03:00
ai-brains schedule run-now
```

### 20.2 Harness integration

```powershell
ai-brains install-hooks --harness claude
ai-brains install-hooks --harness gemini
ai-brains install-hooks --harness codex
ai-brains install-hooks --harness opencode
ai-brains uninstall-hooks --harness claude
ai-brains hook status
```

### 20.3 Wrapper launch

```powershell
ai-brains run -- claude
ai-brains run -- gemini
ai-brains run -- codex
ai-brains run -- ollama launch claude --model glm-5.1:cloud -- --dangerously-skip-permissions
```

### 20.4 Capture API

```powershell
ai-brains ingest-turn
ai-brains ingest-final
ai-brains ingest-session-event
```

These commands are primarily for harness hooks and should accept JSON over stdin.

### 20.5 Recall and injection

```powershell
ai-brains preflight --cwd . --format json
ai-brains recall "query" --project . --format json
ai-brains context --project . --format json
ai-brains sessions --project . --since 7d --format json
ai-brains active --project . --format json
ai-brains conflicts --project . --format json
```

### 20.6 Project management

```powershell
ai-brains project list
ai-brains project show .
ai-brains project alias add --project LegisAI --path C:\dev\LegisAI
ai-brains project policy set --project LegisAI --privacy local_only
ai-brains project exclude --path C:\dev\SecretProject
```

### 20.7 Memory management

```powershell
ai-brains pin "memory text" --project .
ai-brains forget --id <memory-id>
ai-brains seal --id <memory-id>
ai-brains redact --id <memory-id>
ai-brains compact --project . --since today
ai-brains export --project .
ai-brains backup create
ai-brains backup restore
```

---

## 21. Data Model

### 21.1 Canonical relational store

Core tables:

```text
organizations
users
devices
projects
project_aliases
harnesses
models
sessions
turns
session_events
memories
memory_sources
memory_access_log
privacy_policies
conflicts
execution_recipes
backups
jobs
```

### 21.2 Graph model

Node labels:

```text
Project
User
Device
Harness
Model
Session
Turn
Memory
File
GitCommit
Branch
Decision
Lesson
Recipe
Conflict
```

Relationship types:

```text
WORKED_ON
USED_HARNESS
USED_MODEL
OCCURRED_IN
MENTIONS
TOUCHED_FILE
DERIVED_FROM
SUPPORTS
CONTRADICTS
SUPERSEDES
RELATED_TO
CLUSTER_MEMBER
PINNED_TO_PROJECT
RELEVANT_TO
```

### 21.3 Memory record fields

```text
id
organization_id
user_id
project_id
source_session_id
kind: raw_turn | session_summary | daily_summary | durable_lesson | recipe | conflict | cluster
scope: global | project | user_private | org_shared
privacy: cloud_ok | local_only | never_inject | sealed
confidence: high | medium | low
basis: hook_final | wrapper_capture | transcript_parse | inferred | nightly_summary | user_pinned
content_markdown
content_json
source_hash
created_at
last_accessed_at
expires_at
is_pinned
is_deleted
```

---

## 22. Technical Stack

All critical components must be open for commercial use or isolated behind optional user-installed services. Avoid SSPL, AGPL, GPL, and BSL components in the default product path.

### 22.1 Recommended default stack as of 2026-04-26

| Layer                | Choice                    |                            Version pin | License posture            | Notes                                                       |
| -------------------- | ------------------------- | -------------------------------------: | -------------------------- | ----------------------------------------------------------- |
| Language             | Rust                      |                                 1.95.0 | MIT/Apache ecosystem       | Primary implementation language.                            |
| CLI parser           | clap                      |                                 4.5.57 | MIT OR Apache-2.0          | Stable Rust CLI standard.                                   |
| Async runtime        | Tokio                     |                                 1.51.0 | MIT                        | Local daemon, scheduler coordination, HTTP/named-pipe API.  |
| Local API            | Axum                      |                                  0.8.8 | MIT                        | Localhost daemon API.                                       |
| Serialization        | serde                     |                                1.0.228 | MIT OR Apache-2.0          | JSON/TOML serialization.                                    |
| HTTP client          | reqwest                   |                                 0.13.2 | MIT OR Apache-2.0          | Optional model/provider calls.                              |
| Canonical DB binding | rusqlite                  |                                 0.39.0 | MIT                        | Use bundled SQLCipher feature if practical.                 |
| Encrypted DB         | SQLCipher CE              | 4.14.0 / Android 4.14.1 where relevant | BSD-style                  | Encrypted SQLite vault.                                     |
| Graph DB             | SQLite (Relational Graph) |                                  3.45+ | MIT                        | Relational graph model using Recursive CTEs.                |
| Embeddings           | Nomic Embed Text v1.5     |                         768 dimensions | Provider/model terms apply | Default embedding model.                                    |
| Local summarizer     | Gemma 4 E4B-it            |           latest local installed model | Model license/terms apply  | Local nightly summarization.                                |
| Logging              | tracing                   |                                 0.1.41 | MIT                        | Rust instrumentation.                                       |
| Config               | TOML + serde              |                     current compatible | MIT/Apache ecosystem       | Store under user profile.                                   |
| Windows scheduler    | schtasks / Task Scheduler |                              OS-native | Microsoft OS component     | 3am heartbeat.                                              |

### 22.2 Explicitly rejected or non-default components

| Component                             | Status            | Reason                                                                                                                |
| ------------------------------------- | ----------------- | --------------------------------------------------------------------------------------------------------------------- |
| Neo4j Community as default            | Reject as default | GPL/enterprise posture is not clean enough for commercial redistribution as a bundled default.                        |
| FalkorDB core as default              | Reject as default | Core repository is SSPLv1. Official clients may be permissive, but core is not ideal for this product’s default path. |
| KuzuDB original                       | Reject as default | Archived/read-only in 2025.                                                                                           |
| AGPL memory frameworks                | Reject default    | Too much commercial licensing friction.                                                                               |
| Repo-local markdown memory as default | Reject default    | Pollutes repos and creates accidental commit risk.                                                                    |

### 22.3 Optional enterprise/service backends later

| Backend                | When to consider                                             |
| ---------------------- | ------------------------------------------------------------ |
| Postgres + pgvector    | Team/enterprise central server.                              |
| Neo4j Enterprise       | Enterprise customers already licensed or willing to license. |
| Managed object storage | Shared team backups/artifact storage.                        |
| Cloud LLM providers    | Only for `cloud_ok` content.                                 |

---

## 23. Commercial License Gate

Every dependency must pass a license gate before release.

Allowed by default:

```text
MIT
Apache-2.0
BSD-2-Clause
BSD-3-Clause
ISC
Unicode-3.0
Zlib
Public Domain / Unlicense / CC0 when legally acceptable
```

Requires review:

```text
MPL-2.0
LGPL
Elastic License
Commercial dual-license
Model-specific licenses
```

Rejected by default:

```text
AGPL
GPL
SSPL
BSL / BUSL with commercial restrictions
Unknown license
No license
```

Required CI checks:

```powershell
cargo deny check licenses advisories bans sources
cargo audit
cargo machete
cargo nextest run
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

---

## 24. Packaging and Installation

### 24.1 Initial installation

Recommended first release shape:

* GitHub release with signed Windows binary zip.
* `cargo install ai-brains` after crates.io publication.
* PowerShell install script later.
* Winget later after the binary stabilizes.

### 24.2 Runtime modes

| Mode           | Description                                              |
| -------------- | -------------------------------------------------------- |
| CLI-only       | Commands work without daemon; slower, lower concurrency. |
| Daemon mode    | Recommended; serializes writes and handles hooks safely. |
| Scheduled mode | Task Scheduler runs nightly heartbeat.                   |

### 24.3 Daemon communication

Use a local-only transport:

* Windows named pipe preferred for local service communication.
* Localhost HTTP fallback for easier debugging.
* Never expose externally by default.

---

## 25. Acceptance Criteria

### 25.1 Capture acceptance

* Captures user prompt and final assistant response from Claude Code.
* Captures user prompt and final assistant response from Gemini CLI.
* Captures user prompt and final assistant response from Codex CLI or degrades with clear warning.
* Stores failed/aborted sessions.
* Does not store hidden thinking or raw tool logs.
* Capture succeeds when local model, graph DB, and embeddings are offline.

### 25.2 Multi-session acceptance

* Tracks at least three simultaneous AI sessions across two projects.
* Correctly identifies two sessions under `C:\dev\project1` and one under `C:\dev\project2`.
* Shows active sessions by project.
* Allows a fresh reviewer AI to retrieve another agent’s recent final response.

### 25.3 Path normalization acceptance

The following resolve correctly when appropriate:

```text
C:\dev\Project
c:\dev\project
C:/dev/Project
/mnt/c/dev/Project
\\?\C:\dev\Project
```

### 25.4 Recovery acceptance

* Database remains consistent after process kill.
* Incomplete session is marked interrupted/unknown, not lost.
* Encrypted backup can be restored with recovery kit.
* `ai-brains doctor` detects missing recovery kit.

### 25.5 Preflight acceptance

* Produces JSON and markdown brief.
* Includes project-specific memory.
* Includes relevant global environment memory only when relevant.
* Stays under 1500 words by default.
* Excludes `never_inject` and `sealed` memories.

### 25.6 Nightly acceptance

* Runs via Task Scheduler.
* Creates session summaries.
* Creates daily project summaries.
* Generates embeddings.
* Updates graph relationships.
* Detects at least basic contradictions.
* Produces encrypted backup.
* Continues other projects if one project fails.

---

## 26. Implementation Phases

The product should be ambitious, but implementation still needs safe layering. Build all ultimate features, but keep capture independent from advanced intelligence.

### Phase 0: Repository and supply-chain foundation

Deliverables:

* Rust workspace.
* Cargo deny/audit/machete/nextest/clippy/fmt gates.
* License policy.
* Windows CI.
* Basic `ai-brains doctor`.

### Phase 1: Canonical encrypted capture

Deliverables:

* SQLCipher-backed vault.
* Envelope encryption and recovery kit.
* Append-only events.
* Session/project/user schema.
* Path normalization.
* CLI ingest commands.

### Phase 2: Daemon and concurrency

Deliverables:

* `ai-brainsd` daemon.
* Single writer queue.
* Named pipe or localhost API.
* Crash recovery.
* Active session registry.

### Phase 3: Harness adapters

Deliverables:

* Claude Code hooks.
* Gemini CLI hooks.
* Codex CLI hooks.
* Wrapper mode for Ollama-launched Claude.
* OpenCode adapter.
* Antigravity degraded adapter.

### Phase 4: Recall and preflight

Deliverables:

* Hybrid lexical search.
* Project context command.
* Preflight JSON/markdown output.
* Mid-session recall.
* Pinned memories.

### Phase 5: Nightly brain pipeline

Deliverables:

* Task Scheduler install.
* Session summaries.
* Daily project summaries.
* Nomic embeddings.
* Ladybug graph projection.
* Conflict detection.
* Execution recipe promotion.
* Encrypted backups.

### Phase 6: RAPTOR/CRAG and memory governance

Deliverables:

* RAPTOR clustering.
* CRAG-style evidence validation.
* Staleness downgrades.
* Supersession relationships.
* Retention engine.
* Privacy inheritance validation.

### Phase 7: Enterprise-ready architecture hardening

Deliverables:

* Organization/team IDs fully exercised.
* Export/import.
* Multi-user policy model.
* Optional central backend design.
* Admin audit report.
* License report generation.

---

## 27. Risks and Mitigations

| Risk                                    | Mitigation                                                                        |
| --------------------------------------- | --------------------------------------------------------------------------------- |
| Hooks change across harness versions    | Adapter versioning and `doctor` checks.                                           |
| Hook output breaks harness JSON parsing | Strict JSON-only hook output, tests with real harnesses.                          |
| SQLite write contention                 | Daemon owns writes; hooks call daemon/queue.                                      |
| Graph dependency churn                  | Canonical store independent; graph is rebuildable projection.                     |
| Secrets embedded accidentally           | Secret scanner before embedding; privacy escalation.                              |
| Summaries become misleading             | Preserve raw source until retention expiry; CRAG checks; confidence/basis labels. |
| Antigravity remains unrecoverable       | Treat as degraded, wrapper/manual only.                                           |
| Windows path edge cases                 | Dedicated path normalization test suite.                                          |
| Commercial licensing surprise           | License gate in CI; avoid SSPL/AGPL/GPL/BSL default dependencies.                 |

---

## 28. Open Decisions

1. Exact command schema for each harness hook input/output.
2. Whether to support a local web UI later.
3. Whether LadybugDB should be required in v1 or optional until graph features activate.
4. Whether full diff inspection should require explicit project-level opt-in or only local-only privacy.
5. Whether enterprise sync should use Postgres first or encrypted file replication first.

---

## 29. Initial Definition of Done

AI-Brains v1 is done when:

1. A user can run three concurrent AI sessions across two projects.
2. AI-Brains captures user prompts and final AI responses without tool noise.
3. A fresh reviewer AI can ask what another AI did and receive accurate context.
4. Session start preflight injects relevant project memory under 1500 words.
5. Nightly processing creates summaries, embeddings, graph relationships, conflict warnings, and backups.
6. Windows path normalization works across PowerShell, WSL, and Git roots.
7. Storage is encrypted and recoverable with a recovery kit.
8. The default dependency stack is commercially usable and passes license checks.
9. Capture remains reliable when advanced brain features fail.
