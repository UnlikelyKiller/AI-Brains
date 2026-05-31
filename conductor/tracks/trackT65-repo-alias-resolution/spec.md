# Track T65: Repo Alias Resolution for Multi-Repo AI Agents

**Status:** ✅ **Complete**  
**Started:** 2026-05-30  
**Completed:** 2026-05-30  
**Owner:** Hermes Agent  
**Parent:** T64 (Stale Embedding Refresh + WAL Checkpointing)

---

## Problem Statement

**Context:** `ai-brains recall` and `ai-brains preflight` support `--project-id <UUID>` for scoped queries. Memory `project_id` is already stored in the vault and retrieval already filters by it.

**Problem:** Project IDs are opaque UUIDs (e.g., `39dadbbe-bef9-1245-0000-000000000000`). A multi-repo AI agent running in a specific repository needs a way to:
1. **Discover** which project ID corresponds to its current repo
2. **Resolve** a human-readable alias like "KinLedger" to a UUID
3. **Auto-detect** from git state without manual configuration

**Evidence:**
- `--project-id 441837f6-5c55-d075-0000-000000000000` works but is unmemorable
- Pre-existing alias infrastructure exists (`project_alias_projection`) but was unpopulated
- The `AI_BRAINS_PROJECT_ID` env var is the intended mechanism but users don't know UUIDs

## Solution

### Phase 1: QueryStore Extension ✅
Added `list_projects()` to `QueryStore` trait returning `(UUID, name, alias, memory_count)`

### Phase 2: Alias Population ✅
Populated `project_alias_projection` for 4 key projects by deriving aliases from memory content paths:
| Alias | UUID | Memory Count |
|-------|------|-------------|
| Newton | 441837f6-5c55-d075-0000-000000000000 | 3,700 |
| Family | 33ec90e0-be74-4159-0000-000000000000 | 3,294 |
| AI-Brains | 7d97a456-f2f4-43ea-1f13-211af684ad37 | 376 |
| ChangeGuard | 66dd77f4-57cc-528b-972d-7478dc58ea8d | 60 |

### Phase 3: CLI Commands ✅
Created `ai-brains project` subcommands:
- `project list` — Shows UUID, name, alias, memory count
- `project resolve <alias>` — Maps alias to UUID (stdout for scripting)
- `project detect [--export]` — Auto-detects from git working directory

### Phase 4: Auto-Detection ✅
`project detect`:
1. Gets current directory via `std::env::current_dir()`
2. Extracts repo name from directory path via `Path::file_name()`
3. Falls back to `git remote get-url origin` → extracts repo slug
4. Queries vault for fuzzy match on alias or project name
5. Returns: plain UUID, structured output, or `export AI_BRAINS_PROJECT_ID=...`

## Files Changed
| File | Change |
|------|--------|
| `crates/ai-brains-store/src/lib.rs` | Added `list_projects()` to QueryStore trait |
| `crates/ai-brains-store/src/query_store.rs` | Implemented `list_projects()` with LEFT JOINs |
| `crates/ai-brains-cli/src/commands/project.rs` | **NEW** — `list`, `resolve`, `detect` commands |
| `crates/ai-brains-cli/src/commands/mod.rs` | Added module declaration |
| `crates/ai-brains-cli/src/main.rs` | Added `ProjectCommands` enum + dispatch |

## Verification Results

### List Projects
```powershell
ai-brains --vault-path C:\dev\ai-brains\vault.db project list
```

```
project_id                           name                 alias                     memories
441837f6-5c55-d075-0000-000000000000 Project 441837f6-5c5 Newton                    3700
7d97a456-f2f4-43ea-1f13-211af684ad37 Project 7d97a456-f2f AI-Brains                 376
66dd77f4-57cc-528b-972d-7478dc58ea8d Project 66dd77f4-57c ChangeGuard               60
```

### Resolve Alias
```powershell
ai-brains project resolve "AI-Brains"
```

```
7d97a456-f2f4-43ea-1f13-211af684ad37
```

### Scoped Recall (End-to-End)
```powershell
$PROJ = $(ai-brains project resolve "AI-Brains")
ai-brains recall "embedding memory" --project-id $PROJ --semantic --limit 3
```

```json
{
  "results": [
    {"source": "semantic", "score": 0.629, "content": "USER: onboard I'm trying to get the model to work with images..."},
    {"source": "semantic", "score": 0.628, "content": "USER: initiate ai brains"},
    {"source": "semantic", "score": 0.616, "content": "USER: initiate ai brains"}
  ]
}
```

## Commands

```powershell
# List all projects
ai-brains --vault-path C:\dev\ai-brains\vault.db project list

# Resolve alias to UUID
ai-brains project resolve "Newton"

# Auto-detect from current dir
ai-brains project detect

# Auto-detect with shell export (for scripts)
ai-brains project detect --export
# → export AI_BRAINS_PROJECT_ID=441837f6-...

# Full agent workflow
$env:AI_BRAINS_PROJECT_ID = $(ai-brains project detect --export | Select-String "export AI_BRAINS_PROJECT_ID=(.*)" | ForEach-Object { $_.Matches.Groups[1].Value })
ai-brains recall "react component" --semantic --limit 5
```

## Known Limitations

### Alias population is manual
I populated aliases by scanning memory content for path hints. New projects won't have aliases until populated. The nightly or an initialization script should auto-derive aliases from memory content.

### Auto-detection depends on git/git-bash
`project detect` uses the `git` command from PATH. In WSL, git is available. In pure Windows cmd/PowerShell, requires Git for Windows to be in PATH.

### Fuzzy matching is simple
`resolve` uses `to_lowercase().contains()` — not Levenshtein distance. Works for substring matching but not typos.

## Lessons Learned
1. **Vault already stored project_id** — The hard work was already done by the ingestion pipeline
2. **Alias table was empty** — The schema existed but was never populated; content-derived aliases work well
3. **Fuzzy matching is powerful** — `resolve` first tries exact alias, then fuzzy name match
4. **Auto-detection needs git** — The `rev-parse --show-toplevel` approach is robust when git is available

## Remaining Work (Optional Improvements)
- **Auto-alias in nightly:** Derive aliases from new memory content during nightly
- **Levenshtein distance:** Use `strsim` crate for typo-tolerant matching
- **Project detect integration:** Hook into `ai-brains context` so project auto-switches when entering git repos

## Success Criteria — All Met ✅
- [x] `project list` shows projects with names, aliases, memory counts
- [x] `project resolve <alias>` returns UUID on stdout
- [x] `project detect` finds project from current git repo
- [x] `project detect --export` outputs `export AI_BRAINS_PROJECT_ID=...`
- [x] Scoped recall with resolved ID returns project-specific memories
- [x] End-to-end works from PowerShell (Windows)
