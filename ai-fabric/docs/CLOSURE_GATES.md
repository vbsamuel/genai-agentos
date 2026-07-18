# Cutover Closure Gates

The branch cannot become primary runtime until all of the following are true:

- Redis and Celery execution paths are removed.
- Idempotency and event flow are durable.
- Cancellation, replay, restore, and closure verification are implemented.
- HTTP, gRPC, and WebSocket use the same kernel.
- Local certificate wallet and workload identity are implemented.
- Model, RAG, agent, and tool execution remain behind shared governance.
- Self-SRE and closure watchdog are operational.
- Security, performance, recovery, and cross-tenant tests pass.
- Compliance evidence is generated for the deployed boundary.
