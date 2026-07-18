# AI_Fabric_Microservices Architecture

## Singular product model

`AI_Fabric_Microservices` is one product. `ai_fabric` is the only code namespace used for its canonical contracts and behaviors.

The runtime exposes six governed planes through one kernel:

1. `BEHAVIOR_CONTROL` — admission, policy, quota, idempotency, cancellation.
2. `EVENT_ORCHESTRATION` — activation order, scheduling, postcursor selection.
3. `EVENT_FLOW` — canonical ordered events and acknowledgements.
4. `DATA_FLOW` — validated payload movement, persistence, retrieval, lineage.
5. `TRANSITION_STATE` — guarded durable state changes and owner epochs.
6. `GOVERNANCE` — identity, tenant, policy epoch, audit, assurance, closure.

The planes are separate responsibilities, not independent products or bypass paths.

## Runtime path

```text
Surface Adapter
  -> FabricEnvelope validation
  -> Principal/Tenant context
  -> GovernanceKernel admission
  -> OperationLedger materialization
  -> FabricKernel orchestration
  -> Data and event commit
  -> deterministic closure
  -> surface-specific response
```

## Local-first state

The foundation uses an embedded local ledger to avoid Redis, Celery, paid services, and always-on external infrastructure. The ledger is authoritative for operation state. Future event, idempotency, checkpoint, replay, and lineage tables must share the same ownership and transition rules.

## Migration boundary

Legacy services remain outside the authoritative boundary until individually adapted. They may submit canonical requests and consume canonical events. They may not write operation state, bypass policy, or introduce a parallel event/state model.
