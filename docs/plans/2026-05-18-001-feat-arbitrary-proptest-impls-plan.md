---
title: "feat: Add arbitrary::Arbitrary and proptest Strategy impls for all domain-key types"
type: feat
status: active
date: 2026-05-18
origin: docs/brainstorms/arbitrary-derive-requirements.md
---

# feat: Add arbitrary::Arbitrary and proptest Strategy Impls

## Summary

Add `arbitrary::Arbitrary` impls and proptest `Strategy` impls for `Key<D>`, `Id<D>`, `Uuid<D>`, and `Ulid<D>` behind two independent feature flags (`arbitrary`, `proptest`). `Key<D>` generation is constructive — assembled character-by-character from `KeyDomain` predicates — ensuring every generated value is valid without filter-rejection risk. A `ProptestKeyDomain` companion trait provides an explicit opt-in override hook for domains with complex custom validation that the constructive path cannot fully capture; domains opt in with a trivial empty impl.

---

## Problem Frame

Property-based testing and fuzzing are table-stakes integrations for foundational type libraries. `nutype` v0.7.0 and `newtype-uuid` both ship these today. Users who write property tests for code that accepts `Key<D>` or `Id<D>` must currently provide manual `Strategy` impls or wrap their domain types to generate valid instances — boilerplate that undercuts the zero-friction promise of domain-key.

The ROADMAP names `proptest` explicitly in v0.10 as a hardening deliverable. `arbitrary::Arbitrary` is the complementary fuzzer interface needed for `cargo-fuzz` and `honggfuzz` targets. (see origin: `docs/brainstorms/arbitrary-derive-requirements.md`)

---

## Assumptions

*This plan was authored without synchronous user confirmation. The items below are agent inferences that fill gaps in the input — un-validated bets that should be reviewed before implementation proceeds.*

- **`proptest_strategy()` lives on companion trait `ProptestKeyDomain`** rather than on `KeyDomain` directly. The brainstorm deferred this choice to planning. This plan resolves it as a companion trait because: `KeyDomain` is a core domain abstraction that should not carry test-framework imports; the companion trait can evolve independently; and changing a `KeyDomain` method signature is semver-unsafe per the v0.4.0 history. If the team prefers the `KeyDomain` method approach, U4 and the re-export pattern will need adjustment. *(Deviates from origin R8, which places this method on `KeyDomain` directly — this is the plan's resolution of the brainstorm-deferred question; see R8 below.)*

- **`ProptestKeyDomain` uses explicit opt-in, not a blanket impl.** `Strategy for Key<D>` is gated on `D: ProptestKeyDomain`. Domains that want the default constructive generation add a trivial empty impl (`impl ProptestKeyDomain for MyDomain {}`); domains that need the override implement `proptest_strategy()`. A blanket `impl<D: KeyDomain> ProptestKeyDomain for D {}` was considered and rejected: it is incompatible with R8's override requirement in stable Rust — any downstream `impl ProptestKeyDomain for MyDomain` where `MyDomain: KeyDomain` triggers E0119 (coherence conflict). Specialization (RFC 1210) is unstable and not available. The opt-in approach has a minor ergonomics cost (one empty impl per domain) but is the only sound choice on stable Rust.

- **`Uuid<D>` is double-gated on the `uuid` feature** (same as `Ulid<D>` on `ulid`). Origin R1 called out only `ulid` co-gating; `uuid` co-gating was added as a symmetric extension of the same pattern.

- **`proptest = ["std", "dep:proptest"]`** (proptest gates on `std`). proptest 1.x has partial `no_std` support via `alloc = []` + `no_std` features, but the full `proptest!` macro, Strategy combinators, and `TestRunner` require std. The `proptest` feature conservatively implies `std`.

- **`arbitrary = ["dep:arbitrary"]`** (no std implied). `arbitrary` v1.x is no_std-compatible with `default-features = false`. The `arbitrary` feature does not gate on `std`.

- **Dedicated `src/arbitrary_impls.rs` and `src/proptest_impls.rs`** rather than extending `src/integrations.rs`. These are feature-volume-heavy (4 types × 2 features) and keeping them separate mirrors the uuid/ulid pattern of dedicated per-feature files.

- **Version pins**: `arbitrary = "1"`, `proptest = "1"` (both have stable 1.x releases with the needed API surface).

- **`ProptestKeyDomain` is defined in `src/proptest_impls.rs`** and re-exported from `src/lib.rs` at crate-root visibility.

---

## Requirements

- R1. A new `arbitrary` feature flag adds `arbitrary::Arbitrary` impls for `Key<D>`, `Id<D>`, `Uuid<D>` (also behind `uuid` feature; `Uuid<D>` double-gating is a symmetric extension of the `ulid` pattern — origin R1 cited only `ulid`), and `Ulid<D>` (also behind `ulid` feature).
- R2. A new `proptest` feature flag adds proptest `Strategy` impls for all four types (same double-gating for `Uuid<D>` and `Ulid<D>`). `Strategy` for `Key<D>` is gated on `D: ProptestKeyDomain`.
- R3. Both features maintain `no_std` discipline wherever the underlying crates permit (`arbitrary` is fully no_std; `proptest` requires `std`).
- R4. Both feature flags are added to the docs.rs feature list in `Cargo.toml`.
- R5. Generated `Key<D>` instances are always valid — they satisfy all domain validation including `validate_domain_rules()`.
- R6. Generation is constructive (assembled character-by-character from domain predicates), not filter-based, so domains with `HAS_CUSTOM_VALIDATION = false` never produce invalid candidates.
- R7. When `HAS_CUSTOM_VALIDATION = true` and `examples()` is non-empty, the proptest Strategy incorporates the known-good examples as a weighted component alongside constructive generation.
- R8. `ProptestKeyDomain` companion trait provides a defaulted `proptest_strategy()` method returning `None`; domains with complex custom validation may override it to supply a complete Strategy, bypassing the constructive path entirely. *(Deviates from origin R8, which placed `proptest_strategy()` on `KeyDomain` directly — this plan resolves it on the companion trait; see Assumptions for rationale.)*
- R8a. When `HAS_CUSTOM_VALIDATION = true` and `examples()` is non-empty, the `arbitrary::Arbitrary` impl for `Key<D>` draws uniformly from `examples()` rather than the constructive path. For domains where `HAS_CUSTOM_VALIDATION = true` and `examples()` is empty, the impl uses constructive generation but documents that R5 (all generated values are valid) is not guaranteed — such domains should provide `examples()` or use proptest with an R8 override.
- R9. `Id<D>`: generates any valid `NonZeroU64`.
- R10. `Uuid<D>`: delegates to inner `uuid::Uuid` generation (behind `uuid` feature).
- R11. `Ulid<D>`: delegates to inner ULID generation (behind `ulid` feature).

**Origin acceptance examples:** AE1 (covers R5, R6 — constructive generation always valid for `HAS_CUSTOM_VALIDATION = false`), AE2 (covers R5, R7 — examples weighted into proptest strategy), AE3 (covers R5, R8 — override hook bypasses constructive path entirely)

---

## Scope Boundaries

- `CompositeKey<A, B>` arbitrary/proptest impls are not in scope (type ships in v0.8; this feature targets v0.10).
- `cargo-fuzz` targets and corpus tooling are not in scope — this feature provides the interface; users write their own fuzz targets.
- No derive macro for `proptest_strategy()` override — the override is a manual impl of `ProptestKeyDomain`.

### Deferred to Follow-Up Work

- `arbitrary` impl for `Key<D>` alphabet caching (avoid re-enumerating allowed chars on every call) — deferred as a performance optimization after the initial impl ships. Note: the initial impl scans ASCII printable (`U+0020–U+007E`); domains requiring non-ASCII characters must provide `examples()` or override `ProptestKeyDomain::proptest_strategy()`.
- `CompositeKey<A, B>` impls: separate PR after v0.8.

---

## Context & Research

### Relevant Code and Patterns

- `Cargo.toml` optional dep pattern to mirror: `ulid = { version = "1", optional = true, default-features = false }` with feature entry `ulid = ["std", "dep:ulid"]`. The `arbitrary` dep differs — no `std` gate.
- `src/lib.rs` feature-gated module: `#[cfg(feature = "uuid")] pub mod uuid;` — mirror for `arbitrary_impls` and `proptest_impls`.
- `src/key.rs` serde impls (lines 196–231): manually written `impl<D: KeyDomain> Serialize for Key<D>` with no extra bounds on D — the exact signature pattern to mirror for Arbitrary and Strategy impls.
- `src/integrations.rs`: architectural model for cross-cutting feature impls in their own module.
- `src/id.rs` (lines 288–301), `src/uuid.rs` (lines 514–531), `src/ulid.rs` (lines 415–445): serde impls for each inner type — inner construction patterns to follow.
- `src/domain.rs` (lines 228–510): `KeyDomain` methods relevant to generation: `MAX_LENGTH`, `min_length()`, `allowed_characters()`, `allowed_start_character()`, `allowed_end_character()`, `allowed_consecutive_characters()`, `HAS_CUSTOM_VALIDATION`, `validate_domain_rules()`, `examples()`. **All predicates needed for constructive generation already exist on `KeyDomain` directly — no additional abstraction type is needed.**
- `Key::new()`: validation entry point for final call after constructive assembly.
- `Key::from(SmartString)`: bypass constructor — usable after constructive assembly that has already verified all predicates.
- `no_std` import pattern: every `std` import paired with `#[cfg(not(feature = "std"))] use alloc::...` (see v0.4.1 bug history).
- `#[expect(lint, reason = "...")]` over `#[allow(...)]` — enforced by `clippy::all = "deny"` + zero `#[allow]` in `src/`.
- `KeyParseError` is `#[non_exhaustive]` — any match inside impls uses `_ =>` catch-all.

### Institutional Learnings

- (CHANGELOG v0.4.0) Removing or changing `KeyDomain` method signatures is breaking. Using a companion trait (`ProptestKeyDomain: KeyDomain`) avoids touching `KeyDomain`'s API surface entirely.
- (CHANGELOG v0.4.1) `no_std` bugs: missing `alloc` import for `ToOwned` after `Cow` use; unguarded `std` in test code. All `String`/`Vec`/`format!` in `arbitrary_impls.rs` must use `alloc::` aliases.
- (CHANGELOG v0.4.0) `suggestions()` return type change was breaking — further justification for keeping proptest concerns off `KeyDomain`.
- `KeyParseError` is `#[non_exhaustive]` — use `_ =>` catch-all in all match arms.

### External References

- `proptest` v1.11.0 `Cargo.toml`: `default = ["std", "fork", "timeout", "bit-set"]`; has partial `alloc`/`no_std` features but full Strategy + `proptest!` macro require `std`. Confirms `proptest = ["std", "dep:proptest"]`.
- `arbitrary` v1 docs: `Arbitrary` trait for structured data from raw bytes; no_std-compatible with `default-features = false, features = ["derive"]`.

---

## Key Technical Decisions

- **Companion trait `ProptestKeyDomain`** (see Assumptions): `KeyDomain` stays free of test-framework imports; the companion trait evolves independently; explicit opt-in (empty `impl ProptestKeyDomain for MyDomain {}`) gives domains the default constructive strategy. A blanket impl was rejected as incompatible with R8's override requirement in stable Rust (E0119 coherence conflict — see Assumptions for full rationale).
- **Alphabet enumeration range (resolved):** The constructive generator scans ASCII printable (`U+0020–U+007E`). Domains that require non-ASCII characters must provide `examples()` (proptest path R7/R8a, arbitrary path R8a) or override `proptest_strategy()`. This is a plan-level correctness boundary, not a performance decision: a narrower range is predictable; scanning full Unicode (1.1M codepoints) per call would make property-test suites impractically slow without caching, which is deferred.
- **Constructive generation for `Key<D>`**: Enumerate allowed chars from `KeyDomain` predicates into a `Vec<char>` at generation time (ASCII printable range); pick position-appropriate chars respecting start/end/consecutive constraints. Call `Key::new()` as the final validation step. Always produces valid keys for `HAS_CUSTOM_VALIDATION = false` domains.
- **`proptest` gates on `std`**: Full proptest feature set requires std; `proptest = ["std", "dep:proptest"]`.
- **`arbitrary` is genuinely no_std**: `arbitrary = ["dep:arbitrary"]`; use `alloc::` throughout `arbitrary_impls.rs`, not `std::`.
- **Double-gating for `Uuid<D>` and `Ulid<D>`**: impls inside `arbitrary_impls.rs` / `proptest_impls.rs` are additionally gated `#[cfg(feature = "uuid")]` / `#[cfg(feature = "ulid")]` via `cfg` attributes. When enabling the `arbitrary` feature alongside `uuid`, the manifest should also activate `uuid`'s own `arbitrary` impl feature: `uuid = { ..., features = ["arbitrary"] }` under `[dev-dependencies]` or by updating the `uuid` entry to include the feature when `arbitrary` is active. Confirm the exact feature names for `ulid` in the same pass. If neither uuid nor ulid expose their own `Arbitrary` impls, the impls are hand-written (construct via bytes) and this note becomes N/A.
- **Phantom-bound-free impls**: No `D: Arbitrary`, `D: Strategy`, or similar extra bounds. Every impl is `impl<D: KeyDomain> Trait for Type<D>` or `impl<D: IdDomain> Trait for Id<D>` — mirroring the serde impl pattern.

---

## Open Questions

### Resolved During Planning

- **Does proptest support no_std for Strategy and proptest! macro?** — Not practically. proptest's `alloc`/`no_std` features are limited; the full `proptest!` macro and Strategy combinators require `std`. Resolution: `proptest = ["std", "dep:proptest"]`.
- **Version pins** — `arbitrary = "1"` (v1.x stable), `proptest = "1"` (v1.11.0 current). Both pin to SemVer-major.
- **`proptest_strategy()` placement** — Companion trait `ProptestKeyDomain`: explicit opt-in, no blanket impl. See Assumptions — blanket impl was evaluated and rejected (E0119 in stable Rust).
- **Alphabet enumeration range** — ASCII printable (`U+0020–U+007E`). Domains needing non-ASCII must supply `examples()` or override `ProptestKeyDomain::proptest_strategy()`. This is a correctness boundary: the initial impl makes this explicit rather than silently producing a biased alphabet for non-ASCII domains.
- **`DomainConstraints` prerequisite removed** — All predicates needed for constructive generation already exist on `KeyDomain` directly (`src/domain.rs`). U3 and U5 use `D::*` calls on `KeyDomain` without requiring any additional abstraction type.

---

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The implementing agent should treat it as context, not code to reproduce.*

**Key<D> constructive generation (applies to both Arbitrary and proptest Strategy):**

```
function generate_key<D: KeyDomain>(length, char_source):
  // CHAR_RANGE = ASCII printable U+0020–U+007E (see Key Technical Decisions — alphabet enumeration range)
  alphabet      = chars in CHAR_RANGE where D::allowed_characters(c)
  start_chars   = chars in alphabet where D::allowed_start_character(c)
  end_chars     = chars in alphabet where D::allowed_end_character(c)

  chars = []
  prev  = None
  for position in 0..length:
    candidates = match position:
      0    => start_chars
      last => [c in end_chars   if D::allowed_consecutive_characters(prev, c)]
      _    => [c in alphabet    if D::allowed_consecutive_characters(prev, c)]
    if candidates.is_empty():
      return Err(EmptyChoose)   // propagate to fuzzer/test-runner
    c = char_source.pick_from(candidates)
    chars.push(c); prev = Some(c)

  Key::new(chars.join())   // final validation (always passes for HAS_CUSTOM_VALIDATION=false)
```

**ProptestKeyDomain three-path strategy selection (evaluated at strategy construction):**

```
Strategy for Key<D: ProptestKeyDomain>:

  // Path 1 — explicit override (AE3)
  if let Some(s) = D::proptest_strategy():
    return s

  // Path 2 — weighted examples mix (AE2)
  if D::HAS_CUSTOM_VALIDATION && D::examples().is_empty() == false:
    return weighted_union(
      examples_strategy(D::examples()),   // known-good
      constructive_strategy::<D>()        // structural variety
    )

  // Path 3 — pure constructive (AE1 / happy path)
  return constructive_strategy::<D>()
```

---

## Output Structure

```
src/
  arbitrary_impls.rs    (new — Arbitrary impls for all four types)
  proptest_impls.rs     (new — ProptestKeyDomain trait + Strategy impls)
  lib.rs                (modified — add two cfg-gated mod declarations + ProptestKeyDomain re-export)
  domain.rs             (modified — add cfg(feature="proptest") cross-reference note to KeyDomain docs)
  key.rs                (unmodified — Key<D> impls live in the new files)
Cargo.toml              (modified — deps, features, docs.rs)
CHANGELOG.md            (modified — new [Unreleased] entry)
README.md               (modified — add arbitrary/proptest rows to features table if present)
```

---

## Implementation Units

### U1. Cargo.toml and module scaffolding

**Goal:** Wire up the two new feature flags, optional dependencies, docs.rs entry, and empty module stubs so the crate compiles with either or both features before any impls are written.

**Requirements:** R1, R2, R3, R4

**Dependencies:** None

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/arbitrary_impls.rs`
- Create: `src/proptest_impls.rs`

**Approach:**
- `Cargo.toml` `[dependencies]`: add `arbitrary = { version = "1", optional = true, default-features = false, features = ["derive"] }` and `proptest = { version = "1", optional = true, default-features = false }`.
- `Cargo.toml` `[features]`: add `arbitrary = ["dep:arbitrary"]` and `proptest = ["std", "dep:proptest"]`.
- `Cargo.toml` `[package.metadata.docs.rs]` features list: add `"arbitrary"` and `"proptest"` alongside the existing entries.
- **uuid/ulid upstream feature additions (verify at implementation time):** When the `arbitrary` feature is enabled alongside `uuid`, check whether `uuid` exposes an `arbitrary` impl feature (e.g., `uuid/arbitrary`); if so, add it to the `uuid` entry in `[dependencies]`. Repeat for `ulid`. If neither crate exposes their own Arbitrary impls, the impls in `arbitrary_impls.rs` are hand-written (construct via bytes) and no feature addition is needed. Document the finding in a code comment.
- `src/lib.rs`: add `#[cfg(feature = "arbitrary")] mod arbitrary_impls;` and `#[cfg(feature = "proptest")] pub mod proptest_impls;`. The proptest module is `pub` to allow `ProptestKeyDomain` re-export.
- `src/lib.rs`: add `#[cfg(feature = "proptest")] pub use proptest_impls::ProptestKeyDomain;` at the crate-root use block.
- `src/arbitrary_impls.rs`: module stub with module-level doc comment.
- `src/proptest_impls.rs`: module stub with module-level doc comment.

**Patterns to follow:**
- `Cargo.toml` ulid block: `ulid = { version = "1", optional = true, default-features = false }` + feature entry `ulid = ["std", "dep:ulid"]`.
- `src/lib.rs` feature-gated module: `#[cfg(feature = "uuid")] pub mod uuid;`.

**Test scenarios:**
- Happy path: `cargo check --features arbitrary` compiles with zero errors and zero warnings.
- Happy path: `cargo check --features proptest` compiles with zero errors and zero warnings.
- Happy path: `cargo check --features arbitrary,proptest` compiles with zero errors and zero warnings.
- Happy path: `cargo check` (no features) compiles — no new code in default build path.

**Verification:**
- `cargo check --features arbitrary,proptest` passes with zero errors and zero new lint warnings.

---

### U2. `arbitrary::Arbitrary` impls for `Id<D>`, `Uuid<D>`, `Ulid<D>`

**Goal:** Implement `arbitrary::Arbitrary` for the three simpler types — each delegates entirely to its inner type's generation logic. Every non-zero u64 is a valid `Id<D>`, any UUID is valid for any `Uuid<D>`, and any ULID is valid for any `Ulid<D>`, so no domain-predicate logic is needed.

**Requirements:** R1, R3, R9, R10, R11

**Dependencies:** U1

**Files:**
- Modify: `src/arbitrary_impls.rs`
- Test: `src/arbitrary_impls.rs` (inline `#[cfg(test)] mod tests`)

**Approach:**
- `Id<D>`: Inner type is `NonZeroU64`. Generate a `u64` via the `Unstructured` API; if zero, bit-OR with 1 to guarantee non-zero, then wrap as `NonZeroU64`. Impl signature: `impl<D: IdDomain> Arbitrary for Id<D>` — no extra bound on D.
- `Uuid<D>` (gated `#[cfg(feature = "uuid")]` inside module): Generate 16 bytes via `u.bytes(16)?` and construct `::uuid::Uuid::from_bytes(bytes)`. Impl: `impl<D: UuidDomain> Arbitrary for Uuid<D>`.
- `Ulid<D>` (gated `#[cfg(feature = "ulid")]` inside module): Inner type is `u128`; generate via `u128::arbitrary(u)?` and construct `::ulid::Ulid` from that value. Impl: `impl<D: UlidDomain> Arbitrary for Ulid<D>`.
- All three: no `std::` usage — use `alloc::` if any allocation is needed (these impls likely need none).

**Patterns to follow:**
- `src/key.rs` serde impl signature shape: no extra bounds on D.
- `src/id.rs` lines 288–301 for `IdDomain` bound pattern.

**Test scenarios:**
- Happy path: generate `Id<StandardDomain>` — result is a valid, non-zero Id; `.value()` or equivalent returns a non-zero value.
- Edge case: the zero-u64 case produces a non-zero `Id` (not an error, not a panic).
- Happy path: generate `Uuid<StandardDomain>` (with `uuid` feature) — result is a valid UUID.
- Happy path: generate `Ulid<StandardDomain>` (with `ulid` feature) — result is a valid Ulid.
- Covers AE1 (partial — non-Key types have trivial validity by construction).

**Verification:**
- `cargo test --features arbitrary` passes all test scenarios.
- `cargo test --features arbitrary,uuid,ulid` passes `Uuid<D>` and `Ulid<D>` tests.

---

### U3. `arbitrary::Arbitrary` impl for `Key<D>`

**Goal:** Implement constructive arbitrary generation for `Key<D>`. Pick a valid length, then assemble chars position-by-position using `KeyDomain` predicates directly. For domains where `HAS_CUSTOM_VALIDATION = false`, every generated key is valid by construction; for domains with `HAS_CUSTOM_VALIDATION = true`, draw from `examples()` when available — otherwise fall back to constructive with documented caveats.

**Requirements:** R1, R3, R5, R6, R8a; Covers AE1

**Dependencies:** U1, U2

**Files:**
- Modify: `src/arbitrary_impls.rs`
- Test: `src/arbitrary_impls.rs` (inline `#[cfg(test)] mod tests`)

**Approach:**
- Length: draw from the range `D::min_length()..=D::MAX_LENGTH` using `Unstructured`.
- **`HAS_CUSTOM_VALIDATION = true` path (R8a):** If `D::HAS_CUSTOM_VALIDATION` is true and `D::examples()` is non-empty, draw uniformly from `examples()` and return a key from that pool. If `HAS_CUSTOM_VALIDATION` is true and `examples()` is empty, proceed with the constructive path but emit no special handling — the doc comment must note that R5 is best-effort in this case.
- **Constructive path:** Build `allowed_chars: Vec<char>` by iterating over ASCII printable (`' '..='~'`, i.e., `U+0020–U+007E`) and filtering with `D::allowed_characters(c)`.
- Position 0: filter `allowed_chars` by `D::allowed_start_character(c)`.
- Positions 1..length-1: filter `allowed_chars` by `D::allowed_consecutive_characters(prev, c)`.
- Final position (when length > 1): additionally filter by `D::allowed_end_character(c)`.
- If any candidate set is empty, return `Err(arbitrary::Error::EmptyChoose)`.
- Assemble chars into a `SmartString` (or `alloc::string::String`) and call `Key::new()`. For `HAS_CUSTOM_VALIDATION = false` this is expected to always succeed.
- All allocations use `alloc::vec::Vec`, `alloc::string::String` — no `std::` imports.
- Use `#[cfg(not(feature = "std"))] use alloc::vec::Vec;` pattern.

- **Patterns to follow:**
- `src/key.rs` serde impl for impl signature.
- `src/key.rs` `Key::new()` and `From<SmartString>` for construction.
- `no_std` import pattern from `src/lib.rs` and v0.4.1 CHANGELOG fixes.

**Test scenarios:**
- Happy path: 1000 generated `Key<D>` values for a `HAS_CUSTOM_VALIDATION = false` domain — `Key::new()` succeeds on all; no `IncorrectFormat` errors. Covers AE1.
- Edge case: `min_length() == MAX_LENGTH` — every generated key has exactly that length.
- Edge case: `min_length() == 1` — generated single-char keys satisfy start, end, and allowed predicates simultaneously.
- Edge case: domain where `allowed_start_character()` is a strict subset of `allowed_characters()` — generated keys always start with a valid start character.
- R8a path: a `HAS_CUSTOM_VALIDATION = true` domain with non-empty `examples()` — 100% of generated values come from `examples()`.
- Error path: a degenerate domain predicate that produces an empty candidate set — impl returns `arbitrary::Error::EmptyChoose` without panicking.
- Integration: generated `Key<D>` values round-trip through `Key::as_str()` and reconstruct via `Key::new()` without error.

**Verification:**
- `cargo test --features arbitrary` passes all test scenarios.
- `cargo check --no-default-features --features arbitrary` compiles without any `std::`-only API usage (no_std correctness gate).

---

### U4. `ProptestKeyDomain` companion trait and proptest Strategy for `Id<D>`, `Uuid<D>`, `Ulid<D>`

**Goal:** Define the `ProptestKeyDomain` companion trait with a defaulted `proptest_strategy() → None`. No blanket impl — domains opt in by implementing the trait (empty impl for the default constructive behavior, override for custom). Implement proptest Strategy for the three simpler inner types by delegating to inner-type strategies. Export `ProptestKeyDomain` from the crate root.

**Requirements:** R2, R8, R9, R10, R11

**Dependencies:** U1

**Files:**
- Modify: `src/proptest_impls.rs`
- Modify: `src/lib.rs` (re-export already scaffolded in U1 but verified here)
- Test: `src/proptest_impls.rs` (inline `#[cfg(test)] mod tests`)

**Approach:**
- Define `pub trait ProptestKeyDomain: KeyDomain` with `fn proptest_strategy() -> Option<BoxedStrategy<Key<Self>>>` defaulting to `None`.
- **No blanket impl.** `Strategy for Key<D>` is bounded `where D: ProptestKeyDomain`. Domains that want the constructive default add `impl ProptestKeyDomain for MyDomain {}` (empty impl). This is the only sound approach in stable Rust — see Assumptions for E0119 rationale.
- Verify the `pub use proptest_impls::ProptestKeyDomain;` re-export in `src/lib.rs` compiles.
- `Id<D>` Strategy: generate `u64`, OR with 1 for non-zero guarantee, wrap as `Id`. Alternatively use `prop::num::u64::ANY.prop_map(|v| v | 1)`.
- `Uuid<D>` Strategy (gated `#[cfg(feature = "uuid")]`): generate 16 bytes via `proptest::collection::vec(any::<u8>(), 16)` (or `prop::array::uniform16(any::<u8>())`), construct UUID.
- `Ulid<D>` Strategy (gated `#[cfg(feature = "ulid")]`): `any::<u128>().prop_map(|v| ::ulid::Ulid(v))` (or via public constructor).
- All impls: `impl<D: ...Domain> Strategy for ...Strategy<D>` — no extra bounds on D beyond the domain trait.

**Patterns to follow:**
- `src/integrations.rs` for sub-feature gating pattern inside a module.
- `src/key.rs` serde impl signature for the bound-free impl pattern.

**Test scenarios:**
- Happy path: `Id<StandardDomain>` strategy generates valid non-zero Ids via `TestRunner`.
- Happy path: default `ProptestKeyDomain::proptest_strategy()` returns `None` for a standard unoverridden domain.
- Happy path: a domain struct that overrides `proptest_strategy()` returning `Some(Just(specific_key))` returns that value when called.
- Happy path: `Uuid<D>` and `Ulid<D>` strategies generate values of the correct type (with `uuid`/`ulid` features).
- Integration: `ProptestKeyDomain` is importable from the crate root via `domain_key::ProptestKeyDomain`.

**Verification:**
- `cargo test --features proptest` passes.
- `cargo test --features proptest,uuid,ulid` passes.
- `ProptestKeyDomain` is accessible from the crate root.

---

### U5. proptest Strategy for `Key<D>`

**Goal:** Implement proptest `Strategy` for `Key<D>` with the three-path selection: (1) `ProptestKeyDomain::proptest_strategy()` override takes priority; (2) when custom validation is active with known-good examples, use a weighted union; (3) otherwise pure constructive. Directly covers AE2 and AE3.

**Requirements:** R2, R5, R6, R7, R8; Covers AE2, AE3

**Dependencies:** U1, U3, U4; *(S1 prerequisite removed — see Open Questions)*

**Files:**
- Modify: `src/proptest_impls.rs`
- Test: `src/proptest_impls.rs` (inline `#[cfg(test)] mod tests`)

**Approach:**
- At strategy construction time (not value generation time), evaluate the three paths in the order shown in the High-Level Technical Design section.
- Path 1 (override): `D::proptest_strategy()` returns `Some(s)` — use `s` directly. This path exercises the override and skips constructive entirely.
- Path 2 (examples-weighted): `D::HAS_CUSTOM_VALIDATION && !D::examples().is_empty()` — build a `Union` (or `TupleUnion`) of `Just(key)` elements from `examples()` and the constructive strategy. The weighting ratio is an implementation-time decision; the requirement is that examples appear in a statistically non-trivial fraction.
- Path 3 (constructive): Mirror the constructive logic from U3 but expressed as a proptest `Strategy` using `prop::num::usize::ANY` for length and `prop::sample::select()` or char-range strategies for char selection.
- Final assembly: `Key::new()` on the assembled string — expected to succeed for well-defined domains.
- `std::` usage is acceptable here since `proptest` gates on `std`.

**Execution note:** Write tests for AE2 and AE3 before implementing the weighted union path and override path, respectively.

**Technical design:** *(see High-Level Technical Design section for the three-path pseudocode)*

**Patterns to follow:**
- U3 constructive generation logic — mirror for proptest Strategy.
- `src/key.rs` serde impl signature.

**Test scenarios:**
- Happy path: `Key<StandardDomain>` strategy generates valid keys — all pass `Key::new()`.
- Happy path: `Key<FixedLengthDomain>` (min==max) generates keys of exactly that length.
- Covers AE2: given a `HAS_CUSTOM_VALIDATION = true` domain with a non-empty `examples()` array, generate 100 values — all are valid and at least one is drawn from the examples pool.
- Covers AE3: given a domain overriding `proptest_strategy()` with `Some(Just(known_key))`, strategy always produces `known_key` and never exercises the constructive path.
- Integration: `proptest! { |key: Key<StandardDomain>| { assert!(Key::new(key.as_str()).is_ok()); } }` passes with zero filter rejections.
- Edge case: `HAS_CUSTOM_VALIDATION = true` with empty `examples()` — falls back to pure constructive path, not the examples-weighted path.
- Edge case: `proptest_strategy()` returns `None` explicitly — falls through to standard path selection.
- Edge case: domain with consecutive-char restriction — generated keys never violate the consecutive constraint.

**Verification:**
- `cargo test --features proptest` passes all scenarios.
- Zero proptest filter-rejection (`TestCaseError::Reject`) warnings in test output for `HAS_CUSTOM_VALIDATION = false` domains.
- The origin success criterion: `proptest! { |key: Key<MyDomain>| { ... } }` works with only `features = ["proptest"]` and no extra setup.

---

### U6. Documentation, CHANGELOG, and user-facing polish

**Goal:** Ensure the new feature flags are discoverable, `ProptestKeyDomain` has clear doc comments, and the CHANGELOG records the addition. Update README if a feature-flags section exists.

**Requirements:** R4 (docs.rs exposure); SC3 (documented, discoverable escape hatch for custom-validation domains)

**Dependencies:** U1 through U5

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `src/proptest_impls.rs` (doc comments on `ProptestKeyDomain` and its method)
- Modify: `src/arbitrary_impls.rs` (module doc comment)
- Modify: `src/domain.rs` (add `#[cfg(feature = "proptest")]` note pointing to `ProptestKeyDomain`)
- Modify: `README.md` (if a features/optional-deps table exists — check first)

**Approach:**
- CHANGELOG: new `[Unreleased]` section entry citing the two new feature flags and the `ProptestKeyDomain` companion trait.
- `ProptestKeyDomain` doc comment: explain the explicit opt-in pattern (one empty impl for the constructive default), when to use the override (domains with complex custom validation), link to `HAS_CUSTOM_VALIDATION`, and include a working end-to-end example showing the override path.
- `proptest_strategy()` doc comment: describe the default `None` behavior, override semantics, and note that `None` triggers the three-path selection in U5.
- `ProptestKeyDomain` module (`src/proptest_impls.rs`): include at least one working doc-test example for both the empty-impl (default constructive) path and the override path.
- `src/domain.rs` `KeyDomain` trait: add a `#[cfg(feature = "proptest")]` doc comment note cross-referencing `ProptestKeyDomain` — "To use `Key<D>` in proptest property tests, implement `ProptestKeyDomain` for your domain (see `domain_key::ProptestKeyDomain`)." This surfaces the escape hatch in `KeyDomain` docs where domain authors will encounter it. *(Addresses origin SC3 — the escape hatch must be discoverable from the domain abstraction, not just from the companion trait.)*
- Module doc comments on `arbitrary_impls` and `proptest_impls` following the crate's existing doc style.
- `README.md`: if a features or optional deps table exists, add `arbitrary` and `proptest` rows mirroring the `serde`/`ulid` entries.

**Test scenarios:**
- Test expectation: none — documentation changes have no runtime behavior. Doc-test examples (if added) are run by `cargo test --features arbitrary,proptest --doc`.

**Verification:**
- `cargo doc --features arbitrary,proptest --no-deps` generates documentation with no broken intra-doc links and no `missing_docs` warnings.
- CHANGELOG `[Unreleased]` section is present with the new entry.

---

## System-Wide Impact

- **Interaction graph:** No callbacks, middleware, or observers affected. Both feature flags are purely additive — existing builds without the flags are unchanged.
- **Error propagation:** `arbitrary::Arbitrary` returns `arbitrary::Result<T>` — `EmptyChoose` errors (degenerate domain predicates) propagate to the fuzzing harness as expected. proptest `Strategy` failures propagate as `TestCaseError::Reject` or `TestCaseError::Fail` per proptest convention.
- **State lifecycle risks:** None — all generated types are value types with no shared mutable state.
- **API surface parity:** `CompositeKey<A, B>` will need corresponding impls when it ships in v0.8; deferred to follow-up. No other public types are affected.
- **Integration coverage:** The `proptest!` macro test in U5 verification is a cross-layer scenario — strategy construction → TestRunner → generation → Key validation — that unit tests alone cannot prove.
- **Unchanged invariants:** `KeyDomain`, `Key<D>`, `Id<D>`, `Uuid<D>`, `Ulid<D>` public APIs are completely unchanged in builds without the new feature flags. `ProptestKeyDomain` is an additive companion trait; it is explicitly not a supertrait of `KeyDomain`.

---

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| `arbitrary` v1 API surface changes before implementation | Pin to `"1"` in Cargo.toml; review release notes if significant time passes before implementation begins. |
| Constructive generation fails for a domain where consecutive-char rules make it impossible to complete a key of `min_length` | Return `arbitrary::Error::EmptyChoose` / proptest filter; document as a known limitation for degenerate domain configurations. |
| `no_std` regression in `arbitrary_impls.rs` | Explicitly test `cargo check --no-default-features --features arbitrary` in CI; mirror the v0.4.1 lesson. |
| `proptest_strategy()` return type evolves in a future `ProptestKeyDomain` version | `BoxedStrategy<Key<Self>>` uses type erasure — the internal strategy can change without breaking the interface. |
| uuid/ulid don't expose their own `Arbitrary` feature flags | Verify at U1 implementation time; write hand-rolled byte-based impls if so. |
| Domain with `HAS_CUSTOM_VALIDATION = true` and empty `examples()` produces invalid arbitrary outputs | Documented limitation in R8a and U3 — such domains should either add `examples()` or, for proptest, override `proptest_strategy()`. |

---

## Documentation / Operational Notes

- No runtime behavior change — no rollout, monitoring, or operational concerns.
- Feature flags are opt-in; all existing users are unaffected.
- The docs.rs features list update (U1) ensures both new flags appear in hosted documentation.

---

## Sources & References

- **Origin document:** `docs/brainstorms/arbitrary-derive-requirements.md`
- `src/key.rs` (Key<D> serde impls, Key::new(), SmartString construction)
- `src/domain.rs` (KeyDomain trait, all generation-relevant methods)
- `src/id.rs`, `src/uuid.rs`, `src/ulid.rs` (serde impl patterns for inner types)
- `src/integrations.rs` (feature-gated module architecture)
- `Cargo.toml` (ulid/uuid optional dep and feature entry patterns)
- `CHANGELOG.md` (v0.4.0 breaking change history, v0.4.1 no_std bugs)
- External: proptest v1.11.0 Cargo.toml (std requirement verified)
- External: arbitrary v1 docs (no_std capability verified)
