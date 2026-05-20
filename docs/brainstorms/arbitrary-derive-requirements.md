---
date: 2026-05-18
topic: arbitrary-proptest-impls
---

# Arbitrary and Proptest Impls for Domain-Key Types

## Summary

Add `arbitrary::Arbitrary` and proptest `Strategy` impls for all four domain-key types (`Key<D>`, `Id<D>`, `Uuid<D>`, `Ulid<D>`) behind two independent feature flags (`arbitrary`, `proptest`). Ships after S1 (DomainConstraints) stabilizes — using DomainConstraints as the single source of truth for generation constraints rather than reading from multiple `KeyDomain` trait calls directly.

---

## Problem Frame

Property-based testing and fuzzing are table-stakes integrations for foundational type libraries. `nutype` v0.7.0 and `newtype-uuid` both ship these today. Users who write property tests for code that accepts `Key<D>` or `Id<D>` must currently provide manual `Strategy` impls or wrap their domain types to generate valid instances — boilerplate that undercuts the zero-friction promise of domain-key.

The ROADMAP names `proptest` explicitly in v0.10 as a hardening deliverable. `arbitrary::Arbitrary` is the complementary fuzzer interface needed for `cargo-fuzz` and `honggfuzz` targets.

---

## Requirements

**Feature packaging**

- R1. A new `arbitrary` feature flag adds `arbitrary::Arbitrary` impls for `Key<D>`, `Id<D>`, `Uuid<D>`, and `Ulid<D>` (the `Ulid<D>` impl is additionally gated on the `ulid` feature).
- R2. A new `proptest` feature flag adds proptest `Strategy` impls for all four types (same `ulid` co-gating for `Ulid<D>`).
- R3. Both features maintain `no_std` discipline wherever the underlying crates permit.
- R4. Both feature flags are added to the docs.rs feature list in `Cargo.toml`.

**`Key<D>` generation**

- R5. Generated `Key<D>` instances are always valid — they satisfy all domain validation including `validate_domain_rules()`.
- R6. Generation is constructive (assembled character-by-character from domain predicates), not filter-based, so domains with `HAS_CUSTOM_VALIDATION = false` never spend effort producing invalid candidates.
- R7. When `HAS_CUSTOM_VALIDATION = true` and `examples()` is non-empty, the proptest Strategy incorporates the known-good examples as a weighted component alongside constructive generation.
- R8. `KeyDomain` gains a defaulted `proptest_strategy()` method returning `None`; domains with complex custom validation may override it to supply a complete Strategy, bypassing the constructive path entirely.

**`Id<D>`, `Uuid<D>`, `Ulid<D>` generation**

- R9. `Id<D>` (both interfaces): generates any valid `NonZeroU64` — every non-zero value is valid by definition.
- R10. `Uuid<D>` (both interfaces, behind `uuid` feature): delegates to inner `uuid::Uuid` generation.
- R11. `Ulid<D>` (both interfaces, behind `ulid` feature): delegates to inner ULID generation.

---

## Acceptance Examples

- AE1. **Covers R5, R6.** Given a domain with `HAS_CUSTOM_VALIDATION = false`, when `arbitrary::Arbitrary::arbitrary()` is called, every returned `Key<D>` passes `Key::new()` without error and no `IncorrectFormat` is ever returned.
- AE2. **Covers R5, R7.** Given a domain with `HAS_CUSTOM_VALIDATION = true` and a non-empty `examples()` array, when the proptest Strategy generates 100 instances, all are valid and a statistically non-trivial fraction are drawn from the examples pool.
- AE3. **Covers R5, R8.** Given a domain that overrides `proptest_strategy()` with a custom Strategy, when the proptest Strategy runs, the override's output is used exclusively and the constructive path is not exercised.

---

## Success Criteria

- `cargo test --features arbitrary,proptest` passes with no proptest filter-rejection warnings on all bundled test domains.
- A user can write `proptest! { |key: Key<MyDomain>| { ... } }` with zero extra setup after adding `features = ["proptest"]` to `Cargo.toml`.
- Domains with complex custom validation have a documented, tested escape hatch (`proptest_strategy()` override) that produces valid instances without filter-rejection risk.

---

## Scope Boundaries

- `CompositeKey<A, B>` arbitrary/proptest impls are not in scope (type ships in v0.8; this feature targets v0.10).
- `cargo-fuzz` targets and corpus tooling are not in scope — this feature provides the interface; users write their own fuzz targets.
- No derive macro for `proptest_strategy()` — the override is a manual impl on `KeyDomain`.

---

## Key Decisions

- **S1 prerequisite**: Impls ship after DomainConstraints (S1) stabilizes and use it as the single source of truth for generation constraints, rather than reading from multiple `KeyDomain` trait calls at call-site.
- **Constructive over filter-first**: Character-by-character construction from domain predicates eliminates proptest filter-rejection risk for the common case (`HAS_CUSTOM_VALIDATION = false`).
- **`proptest_strategy()` hook on `KeyDomain`**: The opt-in override method lives on `KeyDomain` (not solely on `DomainConstraints`), adding one defaulted method to the existing trait. Alternative — a standalone `ArbitraryKeyDomain` companion trait — is deferred to planning.
- **Two independent feature flags**: `arbitrary` and `proptest` are separate flags; fuzz-only users don't pull in proptest; property-test-only users don't pull in the arbitrary crate.

---

## Dependencies / Assumptions

- S1 (DomainConstraints) must be stable before this feature ships.
- `arbitrary` crate (v1.x) supports `no_std` — assumption verified against upstream docs.
- `proptest` no_std support is unverified; the `proptest` feature may require `std` — see Outstanding Questions.

---

## Outstanding Questions

### Deferred to Planning

- [Affects R3][Needs research] Does proptest's no_std story cover `Strategy` and the `proptest!` macro, or must the `proptest` feature gate on `std`?
- [Affects R8][Technical] Should `proptest_strategy()` live as a method on `KeyDomain` or on a separate companion trait (e.g., `ArbitraryKeyDomain`)? Both satisfy R8; the right choice depends on how S1's `DomainConstraints` type exposes its data.
- [Affects R1, R2][Technical] Minimum `arbitrary` and `proptest` crate versions to pin — resolve during planning codebase scan.
