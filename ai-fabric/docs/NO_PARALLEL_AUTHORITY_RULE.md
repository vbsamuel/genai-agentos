# No Parallel Authority Rule

No service, package, adapter, connector, agent, model, workflow, or UI may introduce a second authoritative implementation of:

- tenancy;
- authorization;
- idempotency;
- operation state;
- event ordering;
- quota;
- cancellation;
- audit;
- closure.

All such behavior must call the shared `ai_fabric` core. A duplicate implementation is a release-blocking architecture violation.
