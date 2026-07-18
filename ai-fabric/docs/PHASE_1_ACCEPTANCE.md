# Phase 1 Acceptance Gates

The foundation is accepted only when:

- `cargo fmt --all -- --check` passes.
- `cargo check --workspace --all-targets` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo test --workspace --all-targets` passes.
- Invalid payload integrity is rejected.
- Stale owner epoch is rejected.
- Illegal state transition is rejected.
- Terminal transition without closure evidence is rejected.
- Same idempotency key and same payload returns the prior receipt.
- Same idempotency key and different payload is rejected.
- Event queue never exceeds configured capacity.
- Listener defaults to loopback.
- Request body is bounded to 16 MiB by default.
- No Redis or Celery dependency exists under `ai-fabric/`.
- Dependency-license evidence is generated in CI.

Failure of any gate keeps the branch in foundation status and blocks primary-runtime cutover.
