# Plan: Track T40 - Unified Retrieval & Feedback Loop

### Phase 1: Blended Retrieval
- [ ] Task 1.1: Update projection schemas (if necessary) in `ai-brains-store` to efficiently query imported `BridgeRecord` data.
- [ ] Task 1.2: Modify `ai-brains-retrieval` to include relevant `BridgeRecord` projections when answering preflight or ask queries.
- [ ] Task 1.3: Add privacy-filtering checks in the retrieval layer for blended data.

### Phase 2: Nightly Accuracy Sweep
- [ ] Task 2.1: Define the accuracy check algorithm in `ai-brains-brain`, comparing predicted context paths against actual accessed paths from ChangeGuard events.
- [ ] Task 2.2: Integrate the accuracy check into `ai-brains-scheduler` to run as a nightly background job.

### Phase 3: Feedback Metrics
- [ ] Task 3.1: Define new `FeedbackMetricEvent` in `ai-brains-contracts`.
- [ ] Task 3.2: Update the nightly job to append `FeedbackMetricEvent`s to the log based on sweep results.
- [ ] Task 3.3: Write tests verifying the nightly job properly calculates and stores feedback events.
