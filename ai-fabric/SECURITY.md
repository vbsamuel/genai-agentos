# Security Baseline

## Required defaults

- Bind to loopback unless explicitly configured otherwise.
- Reject payloads before large allocation when declared size exceeds budget.
- Verify payload integrity before materialization.
- Keep tenant identity in canonical context, never trust payload-selected tenancy.
- Require owner epoch for every state mutation.
- Require closure evidence for terminal state.
- Keep secrets out of logs and persisted request bodies unless explicitly classified and encrypted.
- Do not allow Redis, Celery, or paid service dependencies in the canonical runtime.

## Planned security work before production cutover

- mTLS and workload identity.
- BYO certificate/key wallet.
- policy-backed authorization.
- encrypted embedded state.
- signed event and closure records.
- dependency and SBOM verification.
- fuzzing of all parsers.
- cross-tenant isolation tests.
- FedRAMP control evidence generation.

This branch is an implementation foundation and is not yet a production authorization boundary.
