# Specification: T09 Security Scanning

## 1. Overview
This specification covers local secret detection and privacy escalation in `ai-brains-security`. The crate must detect likely secret-like content before embedding or cloud use, escalate privacy conservatively, and preserve readable redaction output.

## 2. Dependencies
- `ai-brains-core` for the canonical `Privacy` enum
- `regex` for bounded pattern matching
- `thiserror` for typed errors

## 3. Required Modules
- `src/scanner.rs`
- `src/pattern.rs`
- `src/finding.rs`
- `src/escalation.rs`
- `src/redaction.rs`
- `src/policy.rs`
- `src/errors.rs`

## 4. Rules
- High-confidence secrets such as private keys escalate to `Sealed`.
- Likely secrets such as bearer tokens and connection strings escalate to at least `LocalOnly`.
- Redaction must preserve surrounding readability.
- `NeverInject` and `Sealed` content must not be embeddable.
