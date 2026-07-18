# Dependency Policy

Canonical runtime dependencies SHALL be:

- Open source under an approved permissive license.
- Runnable without a paid service, paid runtime, or commercial database license.
- Replaceable behind `ai_fabric` interfaces.
- Included in the generated dependency-license inventory.

Disallowed in the canonical runtime:

- Redis.
- Celery.
- Mandatory proprietary SaaS.
- Copyleft dependencies that impose distribution obligations inconsistent with the product license, unless explicitly approved.
- Libraries that introduce a second policy, state, workflow, or event authority.

Optional model or third-party connectors SHALL remain disabled by default and SHALL not own authoritative operation, event, idempotency, audit, or closure state.
