# Track T09 — Security Scanning

## Owner
architecture-planner

## Status
Completed

## Objective
Implement secret detection, privacy escalation, redaction, and embedding eligibility rules in the `ai-brains-security` crate.

## Scope
- Detect likely bearer tokens, private keys, and connection strings.
- Produce typed findings with confidence levels.
- Escalate privacy according to finding severity.
- Redact sensitive spans while preserving readability.
- Expose a simple policy for whether content may be embedded.

## Out of Scope
- Cloud scanning services.
- Persistent storage or event writing.
- Full DLP or compliance workflows.

## Files Owned
`crates/ai-brains-security/*`

## Files Allowed To Touch
`crates/ai-brains-security/src/lib.rs`
`crates/ai-brains-security/src/scanner.rs`
`crates/ai-brains-security/src/pattern.rs`
`crates/ai-brains-security/src/finding.rs`
`crates/ai-brains-security/src/escalation.rs`
`crates/ai-brains-security/src/redaction.rs`
`crates/ai-brains-security/src/policy.rs`
`crates/ai-brains-security/src/errors.rs`
`crates/ai-brains-security/tests/*.rs`
`crates/ai-brains-security/Cargo.toml`
`Docs/conductor/trackT09/spec.md`
`Docs/conductor/trackT09/plan.md`
`Docs/conductor/conductor.md`
`Docs/status.md`
`tracks/T09-security-scanning.md`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-security/` and conductor/status planning docs.

## Required Tests First
- `tests/detects_bearer_token.rs`
- `tests/detects_private_key.rs`
- `tests/detects_connection_string.rs`
- `tests/clean_text_not_flagged.rs`
- `tests/likely_secret_escalates_local_only.rs`
- `tests/high_confidence_secret_escalates_sealed.rs`
- `tests/redaction_preserves_readability.rs`
- `tests/sealed_content_not_embeddable.rs`
