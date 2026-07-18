# AI_Fabric_Microservices Implementation Ledger

## Governing constraints

- Consumer-grade laptop is the primary runtime.
- No Redis.
- No Celery.
- No paid runtime dependency.
- One product namespace: `ai_fabric`.
- One canonical behavior-control path.
- One canonical event-orchestration path.
- One canonical event-flow contract.
- One canonical data-flow contract.
- One canonical transition-state engine.
- One canonical governance and closure path.

## Implemented in foundation branch

- [x] Rust workspace isolated under `ai-fabric/`.
- [x] Strong canonical IDs.
- [x] Product-plane enum.
- [x] Transport-surface enum.
- [x] Operation-state enum and legal-transition table.
- [x] Terminal-disposition enum.
- [x] Resource-budget contract.
- [x] Payload integrity verification.
- [x] Embedded durable operation ledger using permissively licensed `redb`.
- [x] Owner-epoch guarded compare-and-transition.
- [x] Terminal closure evidence requirement.
- [x] Singular governance admission kernel.
- [x] Consumer-laptop resource ceiling.
- [x] Bounded local event queue.
- [x] Idempotency replay and collision rejection.
- [x] HTTP control-plane vertical slice.
- [x] Loopback-only default listener.
- [x] CI format, compile, clippy, test, dependency-license, and Redis/Celery prohibition gates.

## Required before primary-runtime cutover

- [ ] Persist idempotency records in the embedded ledger.
- [ ] Persist event flow through an append-only local event log.
- [ ] Add durable outbox publication and consumer checkpoints.
- [ ] Add operation lookup, cancellation, replay, and closure verification APIs.
- [ ] Add gRPC unary/client-stream/server-stream/bidirectional adapters.
- [ ] Add authenticated WebSocket session, heartbeat, resume, and replay.
- [ ] Add artifact range delivery and zero-copy path.
- [ ] Add workload/device authentication and BYO certificate wallet.
- [ ] Add capability, policy, model, prompt, workflow, and tool registries.
- [ ] Add local model and optional provider-neutral inference connectors.
- [ ] Add RAG ingestion/retrieval/reranking/grounding.
- [ ] Add bounded agent/workflow runtime and checkpoints.
- [ ] Add tool executor with independent authorization and side-effect receipts.
- [ ] Add self-SRE diagnosis, quarantine, healing, and closure watchdog.
- [ ] Add FedRAMP-oriented control evidence objects and continuous assurance.
- [ ] Replace legacy Redis/Celery paths capability-by-capability.
- [ ] Remove legacy Redis/Celery deployment dependencies after migration tests pass.

## Cutover gate

The new runtime becomes primary only when every accepted operation reaches a terminal disposition; every state mutation, event, side effect, resource lease, audit record, and replay cursor reconciles; and all legacy Redis/Celery execution paths are disabled and removed.
