---
name: codex-review
description: "Use this skill when you want a cross-model code review, a second opinion on changes, or an independent audit before committing. Trigger when the user asks for a review, a second pair of eyes, cross-model review, Codex review, or wants GPT/Codex to examine code. Also trigger before final verification on high-risk changes."
---

# Codex Cross-Model Review (AI-Brains)

Different AI models catch different issues. Use Codex (GPT-based) as an independent read-only reviewer to supplement Claude-based development. This is especially valuable before committing high-risk changes, after substantial refactors, or when the ChangeGuard impact report shows elevated risk.

**Preferred Models (May 2026):**
- **`gpt-5.5-thinking`**: Best for reasoning and identifying architectural drift. (Default)
- **`gpt-5.3-codex`**: Best for dense code analysis and large diffs (1M token context).

## When To Use

- Before committing high-risk changes (ARCHITECTURE, FEATURE, SECURITY categories)
- After a substantial refactor spanning multiple crates in the workspace
- When ChangeGuard reports `riskLevel: High` or broad temporal couplings
- After implementing a full track from the `Docs/Implementation-Plan.md`
- When you want a second opinion on design decisions (e.g., Event Sourcing, SQLCipher integration, Privacy Inheritance)
- Before creating a PR

## Quick Review (One-Shot)

Run a non-interactive read-only review:

```powershell
codex exec -C "." -s read-only -m gpt-5.5-thinking -o review.md "Review the current phase of work. Compare the current git diff against the base branch, identify bugs, regressions, missing tests, risky patterns (panics, unwraps), and unclear assumptions regarding Event Sourcing or CQRS. Do not modify files. Give findings ordered by severity (critical/high/medium/low)."
```

## ChangeGuard-Aware Review

Include ChangeGuard signals in the review prompt:

```powershell
codex exec -C "." -s read-only -m gpt-5.5-thinking -o review.md "Run 'changeguard impact --summary' to see the current risk level. Then review the git diff with that risk context. Focus on: (1) files with high hotspot scores, (2) unintended couplings between ai-brains-capture and ai-brains-models, (3) SQLCipher migration logic. Do not modify files."
```

## Review Checklist for AI-Brains

When reviewing AI-Brains code, the reviewer should specifically look for:
- **Rust Safety**: Are there any `unwrap()`, `expect()`, or `panic()` calls?
- **Capture Independence**: Does the capture path accidentally depend on models/graph?
- **Event Immutability**: Are events being updated instead of appended?
- **Privacy Inheritance**: Does derived data inherit the source's privacy flag?
- **Windows Pathing**: Does the path normalization handle UNC and WSL correctly?

## Integration with ChangeGuard Workflow

1. Run `changeguard scan --impact` — get risk signals
2. Make your changes
3. Run `changeguard impact` — see blast radius
4. Run `codex exec -s read-only ...` — get cross-model review
5. Address critical/high findings
6. Run `changeguard verify` — run Rust CI gate
7. Commit with `changeguard ledger commit`

## Safety Notes

- Always use `-s read-only` for reviews. The reviewer should never modify files.
- Codex suggestions may not align with this project's strict Rust conventions. Evaluate suggestions against the `onboarding` skill and `core-mandates.md`.
