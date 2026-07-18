# AI_Fabric_Microservices

`AI_Fabric_Microservices` is the singular product namespace. All control, event, data, state-transition, governance, transport, AI, and recovery behavior enters through the `ai_fabric` contracts and `FabricKernel`.

## Product invariants

- One product namespace: `ai_fabric` in code and `AI_Fabric_Microservices` in product-facing text.
- One behavior control plane: the `GovernanceKernel` admits or rejects every operation.
- One event-orchestration plane: the `FabricKernel` owns activation order and emits canonical events.
- One event-flow contract: every event has tenant, operation, plane, sequence, and integrity hash.
- One data-flow contract: payloads enter through validated `FabricEnvelope` objects and remain tenant-bound.
- One transition-state manager: `OperationLedger::compare_and_transition` is the only legal operation-state mutation.
- One end-to-end governance path: contract validation, identity context, tenancy, policy epoch, resource budget, idempotency, state, event, and closure are inseparable.

## Consumer-laptop deployment

The foundation is deliberately local-first:

- Rust binary and embedded `redb` operation ledger.
- Loopback listener by default (`127.0.0.1:8787`).
- No Redis.
- No Celery.
- No required managed service.
- No paid runtime or commercial database license.
- Bounded request body, event queue, memory budget, token budget, tool calls, iterations, and deadline.

Third-party AI providers are optional connectors. The authoritative control, state, event, governance, idempotency, and closure behavior remains local and provider-neutral.

## Current implemented vertical slice

The first closed vertical slice implements:

1. Canonical strong IDs, enums, resource budgets, envelope integrity, and product planes.
2. Embedded durable operation ledger.
3. Guarded compare-and-transition state engine with owner epochs.
4. Local governance admission and consumer-laptop ceilings.
5. Bounded in-memory canonical event queue.
6. Idempotency conflict detection.
7. Deterministic execution and closure manifest hash.
8. HTTP control-plane endpoint and local health endpoints.

The current execution adapter deterministically echoes a validated JSON payload. Model, RAG, agent, tool, gRPC, WebSocket, and durable event adapters must integrate behind the same kernel rather than adding parallel policy or state paths.

## Build

```bash
cd ai-fabric
cargo test --workspace
cargo run -p ai-fabricd -- --bind 127.0.0.1:8787 --state ./var/ai-fabric.redb
```

## Execute a governed operation

Generate valid ULIDs for tenant, principal, and capability, then call:

```bash
curl -sS -X POST \
  http://127.0.0.1:8787/v1/capabilities/<capability-ulid>:execute \
  -H 'content-type: application/json' \
  -d '{
    "tenant_id":"<tenant-ulid>",
    "principal_id":"<principal-ulid>",
    "capability_id":"<capability-ulid>",
    "idempotency_key":"demo-001",
    "payload":{"message":"hello"}
  }'
```

A successful response includes the operation ID, terminal state, terminal disposition, output, and closure-manifest hash.

## Legacy repository integration rule

Existing Python, PostgreSQL, Redis, Celery, router, and agent code is not treated as canonical AI Fabric behavior. Migration must occur capability-by-capability behind adapters. Redis and Celery paths must be retired before the AI Fabric becomes the primary runtime. Existing services may call the local control plane during migration, but they may not directly mutate AI Fabric state.
