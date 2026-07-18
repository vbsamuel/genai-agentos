# Namespace and Enum Standard

## Canonical names

- Product: `AI_Fabric_Microservices`
- Rust crate prefix: `ai-fabric-*`
- Rust module prefix: `ai_fabric`
- Environment prefix: `AI_FABRIC_`
- HTTP resource prefix: `/v1/`
- Metric prefix: `ai_fabric_`
- Event type prefix: `AIF.`

## Enum rules

- Rust enum variants use `UpperCamelCase`.
- Serialized enum values use `SCREAMING_SNAKE_CASE`.
- Enum values are never reused after deprecation.
- State enums distinguish transitional, waiting, failure, recovery, and terminal states.
- Generic values such as `UNKNOWN`, `OTHER`, and `ERROR` require a typed reason code and may not conceal a known state.

## Behavioral ownership

Only these canonical plane values are permitted:

- `BEHAVIOR_CONTROL`
- `EVENT_ORCHESTRATION`
- `EVENT_FLOW`
- `DATA_FLOW`
- `TRANSITION_STATE`
- `GOVERNANCE`

A new service or package must declare one primary plane and must call shared kernels for all other plane responsibilities.
