# Product Boundary

All canonical runtime behavior belongs to `AI_Fabric_Microservices` and the `ai_fabric` namespace.

The following are internal planes of the same product, not separately governable products:

- Behavior control
- Event orchestration
- Event flow
- Data flow
- Transition state
- Governance

A component that cannot operate through the canonical envelope, governance kernel, transition engine, event contract, and closure contract is outside the product boundary and cannot be treated as production-authoritative.
