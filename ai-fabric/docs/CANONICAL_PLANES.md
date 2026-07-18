# Canonical Plane Responsibilities

## Behavior control

Owns admission, capability selection, quotas, idempotency, cancellation, and request-level policy.

## Event orchestration

Owns activation order, legal postcursors, scheduling, retries, compensation, and terminal resolution.

## Event flow

Owns canonical event identity, ordering, integrity, publication, acknowledgement, replay, and consumer checkpoints.

## Data flow

Owns payload validation, movement, persistence, retrieval, lineage, retention, and deletion.

## Transition state

Owns guarded compare-and-transition, owner epochs, transition sequence, and closure state.

## Governance

Owns identity, tenancy, authorization, policy epoch, audit, assurance, and evidence.

No plane may independently reimplement another plane's authority.
