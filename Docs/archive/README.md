# Docs Archive

This directory holds documents that have been superseded, replaced, or are
otherwise kept only for historical reference. They are not part of the current
documentation set.

## Contents

| File | Moved By | Reason |
|---|---|---|
| `audit_report.txt` | T72 | 0-byte placeholder; never populated. |
| `audit_report_integration.md` | T72 | Dated 2026-05-19, references pre-Windows Linux path (`/mnt/c/dev/AI-Brains`). Superseded by `audit2.md` and ongoing CI gate verification. |
| `Phase-17-Review.md` | T72 | Pre-T34 closure review; context no longer reflects the current architecture. |
| `Phase-17-Remediation-Review.md` | T72 | Pre-T34 remediation follow-up; concerns addressed by subsequent tracks. |
| `Phase-17-Final-Closure-Review.md` | T72 | Pre-T34 final closure; concerns addressed by subsequent tracks. |

## When to Add Files Here

Add a file to this archive (rather than deleting it outright) when:

- The content is still searchable from CI logs, changelogs, or PRs, and a
  reader might want to look it up later.
- The file is too large to inline into a conductor track spec.
- The historical signal is still useful for understanding the system's
  evolution.

Otherwise, delete the file and capture the decision in a `changeguard ledger`
entry or conductor track.
