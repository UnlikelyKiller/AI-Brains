# Track T72: Status & Doc Reconciliation

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P1 — closes a 13-track documentation gap that obscured the actual scope of the system

---

## Problem Statement

By the time T71 closed, `Docs/status.md` was last updated on 2026-04-30 and claimed "Current Track: T34." The reality (per `git log`) was that work had shipped through T71 plus a daemon track, with 35+ new tracks, several major architectural additions (live graph, memory events, ChangeGuard bridge), and a substantially different operational surface (daemon, scheduling, structured errors, `--status`).

In addition, the two parallel copies of the changeguard skill had drifted:
- `.claude/skills/changeguard/SKILL.md` contained the full original content including Code Symbol Index, Route Extraction, Call Graph, and AI-Brains Bridge documentation.
- `.agents/skills/changeguard/SKILL.md` was a newer, slimmer copy that had *removed* those four entries and added new ones (Audit Smoke Tests, Publish Hygiene, Cross-Model Review). The live `safety sync` command proves the bridge is real, so the slim copy was misleading.

Finally, three sets of stale artifacts sat at the repo root: an empty `audit_report.txt`, an `audit_report_integration.md` dated 2026-05-19 with a pre-Windows Linux path (`/mnt/c/dev/AI-Brains`), and three `Phase-17-*.md` files from May 18 that pre-dated the T34 work.

## Acceptance Criteria

**AC1:** `Docs/status.md` is rewritten to reflect reality — current track = T72, last shipped = T71, with a Track Roster table covering T35–T76 and a Notable Shipped section summarizing the T44–T71 capabilities.

**AC2:** `.agents/skills/changeguard/SKILL.md` is updated to include the Code Symbol Index, Route Extraction, Call Graph, and AI-Brains Bridge entries in its Core Capabilities list, plus a "Code Symbol Queries — Use These First" section after the Default Workflow. The `.claude/skills/changeguard/SKILL.md` file is left unchanged.

**AC3:** The five stale artifacts are moved to `Docs/archive/` (preserving git history via `git mv`), and a `Docs/archive/README.md` is created explaining the move.

**AC4:** `Docs/ci-tooling.md` gets a "Behavior Notes" section that warns `cargo audit` 0.22.x exits 0 with no final summary line on a clean run, and shows how to interpret the output or use `--json` for explicit confirmation.

## Design Notes

- **No new code, no new tests.** This is a documentation-only track. The only verification is a manual diff review of the changed files.
- **The two SKILL.md copies are not unified** — the `.claude/` version stays as the original full reference, and `.agents/` is the canonical extended version. Both now mention the bridge, so a reader of either file gets the same high-level mental model.
- **Archive uses `git mv`** to preserve history. A reader can still `git log` the files in their new location.

## Files

- `Docs/status.md` (rewrite)
- `.agents/skills/changeguard/SKILL.md` (additions: 4 core-capability entries + 1 workflow section)
- `Docs/ci-tooling.md` (append "Behavior Notes" section)
- `Docs/archive/README.md` (new)
- `audit_report.txt` → `Docs/archive/audit_report.txt` (git mv)
- `audit_report_integration.md` → `Docs/archive/audit_report_integration.md` (git mv)
- `Phase-17-Final-Closure-Review.md` → `Docs/archive/Phase-17-Final-Closure-Review.md` (git mv)
- `Phase-17-Remediation-Review.md` → `Docs/archive/Phase-17-Remediation-Review.md` (git mv)
- `Phase-17-Review.md` → `Docs/archive/Phase-17-Review.md` (git mv)

## Verification

- `git status` shows only the intended moves and edits.
- `git diff --stat` against the prior commit matches the expected file list.
- Manual review of the rewritten `Docs/status.md` confirms no claims about tracks that have not actually shipped.
- `.agents/skills/changeguard/SKILL.md` mentions the bridge, the symbol index, the route extractor, and the call graph in both the Core Capabilities and the workflow sections.
