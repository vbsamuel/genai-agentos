# Legacy Replacement Sequence

1. Inventory every Redis key, Celery queue, task, retry, lock, cache, and state dependency.
2. Classify each use as authoritative state, derived cache, work queue, pub/sub, lock, idempotency, session, or rate limit.
3. Reject direct migration of mixed semantics into one replacement abstraction.
4. Move authoritative operation and transition state to `OperationLedger`.
5. Move idempotency to a durable tenant-scoped ledger table.
6. Move work activation to the canonical event-orchestration plane.
7. Move events to an append-only local event log with consumer checkpoints.
8. Move derived cache to bounded process-local caches with rebuild sources.
9. Move distributed locks to owner epochs and compare-and-transition where possible.
10. Adapt each legacy caller to the local HTTP/gRPC/embedded `ai_fabric` interface.
11. Run dual-read verification without dual authoritative writes.
12. Disable legacy execution path.
13. Verify replay, cancellation, recovery, and closure.
14. Remove Redis and Celery containers, packages, configuration, and secrets.
15. Produce a removal evidence manifest.

No legacy component may remain a hidden state or workflow authority after cutover.
