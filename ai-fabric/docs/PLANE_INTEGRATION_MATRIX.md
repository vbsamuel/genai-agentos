# Plane Integration Matrix

| Operation Stage | Behavior Control | Event Orchestration | Event Flow | Data Flow | Transition State | Governance |
|---|---|---|---|---|---|---|
| Admit | quota/idempotency | activation eligibility | accepted event | envelope validation | NEW→VALIDATING | identity/tenant/policy |
| Execute | cancellation/budget | postcursor scheduling | progress events | payload movement | RUNNABLE→EXECUTING | audit/trace |
| Commit | commit policy | downstream activation | committed event | persistence | EXECUTING→COMMITTED | integrity evidence |
| Close | terminal disposition | no remaining jobs | terminal event | lineage/reconciliation | FINALIZING→TERMINAL | audit/closure evidence |
