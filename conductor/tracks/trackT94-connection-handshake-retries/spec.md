# Track T94: Automatic Host Connection Handshake Jitter & Retries

**Status:** ⏳ **Pending**
**Started:** None
**Owner:** None
**Priority:** P2 — improves robustness of host detection.

---

## Problem Statement

In `ai-brains` daemon status checks (added in T85), the daemon probes local and configured model/embedding backend servers (such as Ollama and llama.cpp) via TCP/HTTP status checks. During slow startup cycles of these backend providers (e.g., when the system is under load or loading a large model), the TCP connection might initially fail or reject, leading to false-negative "Closed" reports. To resolve this, TCP status/handshake checks should implement backoff retries with randomized jitter to handle startup latency gracefully.

## Acceptance Criteria

**AC1:** The backend server connection check logic implements a retry pattern rather than failing on the first attempt.

**AC2:** The retry pattern incorporates exponential backoff with randomized jitter to prevent thundering herds and allow backends to complete their handshakes.

**AC3:** The retry behavior is configured with reasonable limits (e.g., 3-5 max retries, starting at 100ms and capping at a reasonable duration) to avoid delaying status updates excessively if a host is truly offline.

## Design Notes

- Update the TCP/HTTP client connection status helper.
- Implement a helper or loop that sleeps with exponential backoff + jitter using standard Rust duration math and/or rand crate.

## Verification

- Run status probes against a port that opens after a slight delay. Verify the status check retries and successfully resolves the host as online once open, instead of instantly returning "Closed".
