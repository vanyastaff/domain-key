---
title: "feat: Add CompositeKey<A, B, const SEP: char> value type"
type: feat
status: completed
date: 2026-05-19
origin: docs/brainstorms/2026-05-19-composite-key-requirements.md
---

# feat: Add CompositeKey\<A, B, const SEP: char\> value type

## Summary

Add `CompositeKey<A, B, const SEP: char = ':'>` — a first-class value type that pairs two typed
domain keys into a single composite identifier. The type carries the same ergonomic surface as
`Key<T>` (string round-trip, serde, sqlx, axum, actix-web) while preserving per-component type
safety. It is not a `KeyDomain` implementor; framework integrations are wired directly on the
new type. Targets v0.8.

---

## Problem Frame

`Key<A>` and `Key<B>` are typed wrappers around validated strings. Code that needs to address a
resource scoped to a parent (comment-under-post, tenant-partitioned row, path segment pair)
currently reaches for ad-hoc string concatenation — losing type safety, separator convention,
and reliable round-trip parsing. A first-class `CompositeKey` type closes this gap and eliminates
per-site reimplementation of splitting and error handling. See origin document for full problem
frame. (see origin: `docs/brainstorms/2026-05-19-composite-key-requirements.md`)

---

## Assumptions

*This plan was authored without synchronous user confirmation. The items below are agent inferences
that fill gaps in the input — un-validated bets that should be reviewed before implementation
proceeds.*

- `CompositeKeyParseError` is placed in `src/error.rs` alongside all other error types in the
  crate (`KeyParseError`, `IdParseError`, `UuidParseError`, `UlidParseError`). This is the
  unambiguous convention: every existing error type lives there. An alternative (co-location in
  `src/composite_key.rs`) is structurally defensible but breaks the established pattern.

---

## Requirements

**Type structure (R1–R4)**
- R1. Define `CompositeKey<A, B, const SEP: char = ':'>` where `A: KeyDomain`, `B: KeyDomain`.
- R2. Store components as `(Key<A>, Key<B>)` tuple — no cached composite string field.
- R3. Constructor `CompositeKey::new(first: Key<A>, second: Key<B>) -> Self`; `debug_assert!` fires when `first.as_str()` contains `SEP`.
- R4. Read-only accessors `fn first(&self) -> &Key<A>` and `fn second(&self) -> &Key<B>`.

**String representation (R5–R6)**
- R5. `Display` produces `"{first}{SEP}{second}"` on demand.
- R6. `FromStr` (Err = `CompositeKeyParseError`): split on first `SEP`; `MissingSeparator` when absent; propagate component parse errors as `InvalidFirst` / `InvalidSecond`.

**Traits (R7–R10)**
- R7. Derive or implement `Debug`, `Clone`.
- R8. Implement `PartialEq` + `Eq` comparing both component keys.
- R9. Implement `Hash` by sequential component hashing (zero-allocation).
- R10. Implement `PartialOrd` + `Ord` via component-wise comparison (zero-allocation).

**Error (R11–R12)**
- R11. `CompositeKeyParseError`: `#[non_exhaustive]` enum with `MissingSeparator { separator: char }`, `InvalidFirst(KeyParseError)`, `InvalidSecond(KeyParseError)`.
- R12. `CompositeKeyParseError` implements `std::error::Error`, `Display`, `Debug`, `Clone`, `PartialEq`, `Eq`.

**serde (R13–R14)**
- R13. Serialize `CompositeKey` as `"{first}{SEP}{second}"` string.
- R14. Deserialize via `CompositeKey::from_str`, surfacing `CompositeKeyParseError` as serde error. Use `is_human_readable()` branch to support binary formats (bincode, postcard).

**sqlx (R15–R16)**
- R15. sqlx `Type<DB>` / `Encode<'q, DB>` / `Decode<'r, DB>` — generic over all backends via `String: Type/Encode/Decode<DB>` bounds, following the `Key<D>` pattern.
- R16. Encode via `self.to_string()`; decode via `CompositeKey::from_str`.

**axum (R17–R18)**
- R17. No explicit `FromRequestParts` impl needed for axum — `Path<T>` uses `DeserializeOwned`; the `serde` feature must be enabled for axum path extraction to work.
- R18. `IntoResponse` for `CompositeKeyParseError` (400 Bad Request + `Display` body), mirroring `KeyParseError`.

**actix-web (R19)**
- R19. `ResponseError` for `CompositeKeyParseError` (400 Bad Request), mirroring `KeyParseError`.

**Origin acceptance examples:** AE1 (round-trip with default separator), AE2 (missing separator error), AE3 (empty first segment error), AE4 (custom separator `'/'`), AE5 (equality and hash consistency)

---

## Scope Boundaries

- `CompositeKey` does **not** implement `KeyDomain`. Direct framework impls avoid the `Key<CompositeKey<A,B>>` double-wrap antipattern.
- No three-component variant. `CompositeKey<A, CompositeKey<B, C>>` nesting deferred — the `DOMAIN_NAME` const-concatenation problem and `allowed_characters` complexity make it non-trivial.
- No proc-macro or derive macros for `CompositeKey`.
- No runtime validation that `Key<A>` strings are free of `SEP` — only the `debug_assert!` in the constructor.
- No `Borrow<str>` impl in the initial release (deferred — requires cached composite string).
- Not `no_std` compatible in the initial release (depends on `std` through `FromStr` + allocation).

### Deferred to Follow-Up Work

- `no_std` compatibility: future iteration once `Borrow<str>` and allocation story are settled.
- `Borrow<str>` impl: requires struct layout change (cached composite string field).
- ROADMAP.md stale entry (v0.6 claims `impl FromRequestParts` for axum — already satisfied by `FromStr`): cosmetic cleanup, separate PR.

---

## Context & Research

### Relevant Code and Patterns

- `src/key.rs` — `Key<T>` struct layout, `Hash`/`Ord`/serde impl patterns to mirror; `Hash` delegates to `self.inner` (SmartString) for `Borrow<str>` contract; serde uses manual dual-path `is_human_readable()` impl
- `src/error.rs` — `KeyParseError` shape: `#[derive(Debug, Error, PartialEq, Eq, Clone)]` + `#[non_exhaustive]`; thiserror 2.0 `#[error("...")]` syntax; struct-variant form for context fields (e.g., `TooLong { max_length, actual_length }`)
- `src/integrations.rs` lines 24–67 — sqlx generic `Type/Encode/Decode` pattern; lines 418–453 axum `IntoResponse`; lines 467–507 actix-web `ResponseError`
- `src/lib.rs` — module declaration pattern (`pub mod composite_key;`); `pub use` re-export chain; prelude block (lines 397–431); `[package.metadata.docs.rs]` features list (line 128)
- `arbitrary_impls.rs`, `proptest_impls.rs` — precedent for new `src/<topic>.rs` files added alongside the crate's existing flat module layout

### Institutional Learnings

- All error types live in `src/error.rs` — convention unambiguous across `KeyParseError`, `IdParseError`, `UuidParseError`, `UlidParseError`.
- Integration impls (sqlx/axum/actix-web) live **inside** the existing private inner mods of `src/integrations.rs` — never in a parallel file.
- `#[package.metadata.docs.rs] features` currently excludes `sqlx`, `axum`, `actix-web` — these must be added when the framework impls ship, or `CompositeKeyParseError`'s integration impls will be invisible on docs.rs.
- `const SEP: char = ':'` (const-parameter default) stable since Rust 1.59; MSRV 1.86 is sufficient — no nightly gate needed.
- serde for `CompositeKey` simplifies relative to `Key<T>`: no zero-copy path possible since composite string always allocates; deserialize as `&str` then call `from_str`.

### External References

- None required — local patterns are complete and directly applicable.

---

## Key Technical Decisions

- **Error type in `src/error.rs`**: follows the codebase convention where every error type resides in `src/error.rs`. `CompositeKeyParseError` is re-exported at crate root alongside `KeyParseError`. (see origin: Key Decisions)
- **sqlx via generic-DB pattern**: `CompositeKey` is always encoded/decoded as a string, so the single generic `impl<DB: Database, ...>` covers all three backends (Postgres, MySQL, SQLite) without per-backend variants — same as `Key<D>`. No `sqlx-postgres`/`sqlx-mysql`/`sqlx-sqlite` conditional forks.
- **`is_human_readable` branching in serde Deserialize**: `Key<T>` uses `is_human_readable()` to select the deserialization path. `CompositeKey` must do the same — the human-readable path deserializes as `&str` → `from_str`; the binary path (bincode, postcard) deserializes as `String` → `from_str`. Skipping the branch causes binary-format deserialization failures. See `src/key.rs` lines 196–231 for the exact pattern.
- **`Hash` via sequential component hashing (zero-allocation)**: `Key<T>` hashes via `self.inner.hash(state)` (zero allocation, `src/key.rs` lines 185–193). `CompositeKey` cannot delegate to a single inner field, but the zero-allocation equivalent is: hash `self.first().as_str()`, then `SEP as char`, then `self.second().as_str()` sequentially. The prior `to_string()` approach was incorrect — it misrepresented `Key<T>` behavior and allocated on every hash call.
- **`Ord` via component-wise comparison (zero-allocation)**: `Key<T>` uses `self.inner.cmp(&other.inner)` (zero allocation, `src/key.rs` lines 178–183). `CompositeKey` equivalent: `self.first().cmp(other.first()).then_with(|| self.second().cmp(other.second()))`. This preserves lexicographic ordering without allocating two full strings per comparison.
- **Integration impls appended to existing inner mods in `src/integrations.rs`**: not a separate file. This keeps the integration module's architectural contract intact (`integrations.rs` = single home for all framework impls, private mod declarations, private inner mods per feature).

---

## Open Questions

### Resolved During Planning

- **Module location**: `src/composite_key.rs` — consistent with `uuid.rs`, `ulid.rs`, `arbitrary_impls.rs` precedents. `src/key.rs` (60 KB) is already large; separation is warranted.
- **Error type location**: `src/error.rs` — all 4 existing error types reside there; co-location in `composite_key.rs` would be an outlier.
- **MSRV for const-parameter defaults**: Stable since Rust 1.59; MSRV 1.86 is sufficient. Closed.
- **`Hash` collision risk**: No risk — `Key<T>` and `CompositeKey<A,B>` are distinct types and cannot appear as interchangeable map keys. Closed.

### Deferred to Implementation

- Exact `#[error("...")]` message wording for `CompositeKeyParseError` variants — implementation detail.

---

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The implementing agent should treat it as context, not code to reproduce.*

```
CompositeKey<A, B, const SEP: char = ':'>
├── fields: (Key<A>, Key<B>)   ← no cached composite string
├── new(first, second) → Self  ← debug_assert!(first doesn't contain SEP)
├── first() → &Key<A>
├── second() → &Key<B>
│
├── Display: "{first}{SEP}{second}"    ← builds string on demand
├── FromStr → CompositeKeyParseError   ← split on first(SEP), parse halves
├── Hash: self.first.as_str() + SEP + self.second.as_str() (sequential, zero-alloc)
├── Ord/PartialOrd: first.cmp(other.first).then_with(|| second.cmp(other.second)) (zero-alloc)
├── PartialEq/Eq: first == other.first && second == other.second
│
├── #[cfg(feature = "serde")]
│   ├── Serialize: self.to_string().serialize(serializer)
│   └── Deserialize: is_human_readable() → &str or String → from_str → map_err(custom)
│
├── In src/integrations.rs::sqlx_support
│   ├── Type<DB>: delegates to String::type_info()
│   ├── Encode<'q, DB>: self.to_string() → String::encode
│   └── Decode<'r, DB>: String::decode → from_str → map_err(Box as BoxDynError)
│
├── In src/integrations.rs::axum_support
│   └── IntoResponse for CompositeKeyParseError: (400, self.to_string())
│
└── In src/integrations.rs::actix_web_support
    └── ResponseError for CompositeKeyParseError: 400 + body(self.to_string())

CompositeKeyParseError (in src/error.rs)
├── MissingSeparator { separator: char }
├── InvalidFirst(KeyParseError)
└── InvalidSecond(KeyParseError)
```

---

## Implementation Units

### U1. `CompositeKeyParseError` in `src/error.rs`

**Goal:** Introduce the public error type for composite key parsing, following the crate's established error conventions.

**Requirements:** R11, R12

**Dependencies:** None

**Files:**
- Modify: `src/error.rs`
- Modify: `src/lib.rs` (add `CompositeKeyParseError` to `pub use error::...`)

**Approach:**
- Add `CompositeKeyParseError` enum to `src/error.rs` after `KeyParseError`.
- Apply `#[derive(Debug, Error, PartialEq, Eq, Clone)]` + `#[non_exhaustive]` — all four derives are safe because `KeyParseError: PartialEq + Eq + Clone`.
- Use struct-variant form for `MissingSeparator { separator: char }` (consistent with `KeyParseError::TooLong { max_length, actual_length }`).
- Tuple-variant form for `InvalidFirst(KeyParseError)` and `InvalidSecond(KeyParseError)`.
- Add `#[error("...")]` attributes for each variant via thiserror 2.0.
- Re-export from `lib.rs` in the `pub use error::{...}` statement alongside `KeyParseError`.
- Add to prelude (`src/lib.rs` prelude block).
- **`no_std` note**: `CompositeKeyParseError` is `std`-only for v0.8. Do NOT add `#[cfg(not(feature = "std"))] use alloc::...` guards — `CompositeKeyParseError` relies on `std::error::Error`. A future no_std iteration may revisit if `alloc` feature support is added.

**Patterns to follow:**
- `KeyParseError` in `src/error.rs` (lines 48–131) — exact shape to mirror
- `IdParseError` in `src/error.rs` (lines 464–474) — shorter variant for reference

**Test scenarios:**
- Happy path: construct `MissingSeparator { separator: ':' }` — `to_string()` produces a non-empty, human-readable message
- Happy path: `InvalidFirst(e).to_string()` surfaces the inner `KeyParseError` message
- Happy path: `InvalidSecond(e).to_string()` surfaces the inner `KeyParseError` message
- Edge case: `MissingSeparator { separator: '/' }` — `to_string()` includes the `'/'` character in its message (separator character is captured in the message)
- Happy path: `assert_eq!(MissingSeparator { separator: ':' }, MissingSeparator { separator: ':' })` — PartialEq holds
- Edge case: `MissingSeparator { separator: ':' } != MissingSeparator { separator: '/' }` — different separators are not equal

**Verification:**
- `cargo check` passes with `CompositeKeyParseError` added.
- `pub use` re-export is accessible at crate root (`domain_key::CompositeKeyParseError`).
- All error derives compile without issues.

---

### U2. `CompositeKey` core type in `src/composite_key.rs`

**Goal:** Introduce the core struct with constructor, accessors, and all standard trait impls. Wire it into `lib.rs`.

**Requirements:** R1–R10, AE1–AE5

**Dependencies:** U1

**Files:**
- Create: `src/composite_key.rs`
- Modify: `src/lib.rs` (add `pub mod composite_key;`, add `pub use composite_key::CompositeKey;`, update prelude)

**Approach:**
- Define `pub struct CompositeKey<A: KeyDomain, B: KeyDomain, const SEP: char = ':'>(Key<A>, Key<B>)` — tuple struct or named-field struct (named fields preferred for readability: `first: Key<A>`, `second: Key<B>`).
- `new()` constructor with `debug_assert!(!first.as_str().contains(SEP))` — fires when the first component contains the separator (the invalid case). **Note**: in release builds this check is elided; callers must ensure component keys do not contain `SEP`. Document this clearly in the constructor's rustdoc with `# Panics (debug only)` and `# Caller responsibility (release)` sections.
- `first()` and `second()` return `&Key<A>` / `&Key<B>` respectively.
- Implement `Display` formatting `"{}{}{}"` with `first`, `SEP as char`, `second` — no allocation required here beyond the formatter's buffer.
- Implement `FromStr`: find the first occurrence of `SEP` via `input.find(SEP)`; on `None` return `MissingSeparator { separator: SEP }`; split at position; parse left slice as `Key<A>` mapping `KeyParseError` to `InvalidFirst`; parse right slice as `Key<B>` mapping to `InvalidSecond`.
- Derive `Debug`, `Clone`.
- Implement `PartialEq`, `Eq` comparing `self.first == other.first && self.second == other.second`.
- Implement `Hash` via sequential zero-allocation hashing: `self.first().as_str().hash(state); SEP.hash(state); self.second().as_str().hash(state)`.
- Implement `PartialOrd` delegating to `Ord`; implement `Ord` via `self.first().cmp(other.first()).then_with(|| self.second().cmp(other.second()))` — zero allocation, preserves lexicographic semantics.
- Add `#[cfg(test)] mod tests` with named unit tests covering AE1–AE5.
- **`no_std` note**: declare `pub mod composite_key;` in `lib.rs` without any `#[cfg(feature = "std")]` gate — the module is `std`-only by virtue of its `FromStr` + `Display` + `std::error::Error` dependencies. No `alloc` fallback imports. Do not add `#[cfg(not(feature = "std"))] use alloc::...` guards in this file.
- Add `#[cfg(test)] mod tests` with named unit tests covering AE1–AE5.

**Patterns to follow:**
- `Key<T>` Hash impl in `src/key.rs` (lines 185–193) — delegate to string
- `Key<T>` Ord/PartialOrd in `src/key.rs` (lines 170–183) — compare via inner string
- `Id<T>` or `Uuid<T>` in `src/uuid.rs` or `src/ulid.rs` — struct declaration pattern in a standalone file

**Test scenarios:**
- `Covers AE1.` Round-trip: `CompositeKey::new(user_key, post_key).to_string()` == `"user123:post456"`; parsing that string back gives `Ok(ck)` where `ck.first() == &user_key` and `ck.second() == &post_key`
- `Covers AE2.` `from_str("user123")` returns `Err(MissingSeparator { separator: ':' })`
- `Covers AE3.` `from_str(":post456")` returns `Err(InvalidFirst(KeyParseError::Empty))` (empty first segment)
- `Covers AE4.` `CompositeKey::<UserDomain, PostDomain, '/'>`: `new(user_key, post_key).to_string()` == `"user123/post456"`; `from_str("user123/post456")` succeeds; `from_str("user123:post456")` returns `MissingSeparator { separator: '/' }`
- `Covers AE5.` Two `CompositeKey` values built from identical component strings: `==` holds and `hash(ck1) == hash(ck2)`
- Edge case: `from_str("user123:post456:extra")` — splits on FIRST colon; parses as `user123` + `post456:extra`; succeeds if `Key<B>` allows colons in its string
- Edge case: `from_str("")` — returns `MissingSeparator { separator: ':' }` (empty input has no separator)
- Edge case: `from_str(":")` — empty first segment returns `InvalidFirst(KeyParseError::Empty)`; empty second segment returns `InvalidSecond(KeyParseError::Empty)` (order: first is parsed first)
- Happy path: `debug_assert!` fires in debug builds when constructing `CompositeKey::new` with a first key containing `SEP` (test via `assert!(std::panic::catch_unwind(|| {...}).is_err())` under `#[cfg(debug_assertions)]`)
- Integration: `Ord` ordering: `"a:b" < "a:c"` and `"a:b" < "b:a"` — lexicographic on composite string
- Happy path: `Clone` produces independent copies; modifying a cloned key does not affect original

**Verification:**
- `cargo test` passes all unit tests in `src/composite_key.rs`.
- `CompositeKey` is accessible at crate root.
- Clippy passes (no new warnings).

---

### U3. serde integration in `src/composite_key.rs`

**Goal:** Add `Serialize` and `Deserialize` impls behind the `serde` feature flag.

**Requirements:** R13, R14

**Dependencies:** U2

**Files:**
- Modify: `src/composite_key.rs` (add `#[cfg(feature = "serde")]` block)

**Approach:**
- Inside a `#[cfg(feature = "serde")] mod serde_support { ... }` block OR inline `#[cfg(feature = "serde")] impl ...` — follow whichever style `Key<T>` uses (inline `#[cfg]` on each impl).
- `Serialize`: call `self.to_string().serialize(serializer)` — no zero-copy shortcut available.
- `Deserialize`: use `is_human_readable()` branch (same as `Key<T>`, see `src/key.rs` lines 196–231):
  - Human-readable path (JSON, TOML, etc.): deserialize as `&str` → `CompositeKey::from_str(s)`.
  - Binary path (bincode, postcard): deserialize as `String` → `CompositeKey::from_str(&s)`.
  - Map parse errors to `serde::de::Error::custom` in both branches.
  - Skipping this branch causes binary-format deserialization failures (`<&str>::deserialize` fails when the encoder calls `visit_bytes`).

**Patterns to follow:**
- `Key<T>` serde impls in `src/key.rs` lines 196–231 — manual impls, inline `#[cfg(feature = "serde")]`

**Test scenarios:**
- Round-trip: `serde_json::to_string(&ck)` produces `"\"user123:post456\""` (JSON string, not object)
- Round-trip: `serde_json::from_str::<CompositeKey<...>>("\"user123:post456\"")` succeeds and equals original
- Error: `serde_json::from_str::<CompositeKey<...>>("\"user123\"")` returns an error (no separator)
- Custom separator round-trip: `CompositeKey<A, B, '/'>` serializes as `"user123/post456"` and deserializes correctly

**Verification:**
- `cargo test --features serde` passes all serde round-trip tests.
- `serde_json::to_string` and `serde_json::from_str` work correctly for `CompositeKey`.

---

### U4. sqlx integration in `src/integrations.rs`

**Goal:** Add `Type<DB>`, `Encode<'q, DB>`, and `Decode<'r, DB>` impls for `CompositeKey` inside the existing `sqlx_support` inner mod.

**Requirements:** R15, R16

**Dependencies:** U2

**Files:**
- Modify: `src/integrations.rs` (append inside `#[cfg(feature = "sqlx")] mod sqlx_support { ... }`)

**Approach:**
- Mirror the exact generic-over-`DB` pattern from lines 24–67 for `Key<D>`.
- `Type<DB>`: bound `String: Type<DB>`; delegate `type_info()` and `compatible()` to `<String as Type<DB>>`.
- `Encode<'q, DB>`: bound `String: Encode<'q, DB>`; encode via `self.to_string()`. `size_hint()` returns `self.first.as_str().len() + SEP.len_utf8() + self.second.as_str().len()` (accurate for multi-byte separators).
- `Decode<'r, DB>`: bound `String: Decode<'r, DB>`; decode to `String`; call `CompositeKey::from_str(&decoded)`, map error via `Box::new(e) as BoxDynError`.
- No per-backend variants needed (`#[cfg(feature = "sqlx-postgres")]` etc.) — the generic approach covers all three backends.

**Patterns to follow:**
- `src/integrations.rs` lines 24–67 — verbatim analogue for `Key<D>`

**Test scenarios:**
- Compile-time trait assertion: a zero-cost function `fn assert_sqlx_traits<T: Type<DB> + Encode<'q, DB> + Decode<'r, DB>>()` instantiated with `CompositeKey<TestA, TestB>` confirms trait bounds are met at compile time
- Integration: `#[cfg(feature = "sqlx-postgres")] mod sqlx_postgres_traits { ... }` block with a `fn assert_composite_key_sqlx<'q, 'r>()` — confirms compile-time trait satisfaction for Postgres
- Encode round-trip (if test DB is available): bind a `CompositeKey` parameter and decode from a query row — the decoded value equals the original

**Verification:**
- `cargo check --features sqlx` compiles without errors.
- `cargo check --features sqlx-postgres` compiles without errors.
- Trait assertion tests compile under the appropriate feature flags.

---

### U5. axum + actix-web integration in `src/integrations.rs`

**Goal:** Add `IntoResponse` for `CompositeKeyParseError` (axum) and `ResponseError` for `CompositeKeyParseError` (actix-web). Fix the `docs.rs` metadata gap.

**Requirements:** R17, R18, R19

**Dependencies:** U1, U2, U4

**Files:**
- Modify: `src/integrations.rs` (append to `axum_support` and `actix_web_support` inner mods)
- Modify: `Cargo.toml` (`[package.metadata.docs.rs] features` — add `"sqlx"`, `"axum"`, `"actix-web"`; remove `wasm32-unknown-unknown` from `targets` since integration features do not compile to WASM)

**Approach:**
- **axum**: inside `#[cfg(feature = "axum")] mod axum_support`, add `use crate::CompositeKeyParseError;` and implement `IntoResponse` returning `(StatusCode::BAD_REQUEST, self.to_string()).into_response()` — identical pattern to `KeyParseError`.
- **actix-web**: inside `#[cfg(feature = "actix-web")] mod actix_web_support`, add `use crate::CompositeKeyParseError;` and implement `ResponseError` with `status_code()` → `BAD_REQUEST` and `error_response()` → `HttpResponse::build(self.status_code()).body(self.to_string())`.
- **docs.rs metadata**: `[package.metadata.docs.rs]` currently lists `["std", "serde", "secure", "uuid", "ulid", "ulid-monotonic", "arbitrary", "proptest"]`. Add `"sqlx"`, `"axum"`, `"actix-web"` so `CompositeKeyParseError`'s framework impls (and `Key<T>`'s existing ones) are visible in generated docs. **Also remove `wasm32-unknown-unknown` from the `targets` list** (line 130) — adding integration features while WASM target is present breaks the docs.rs build because axum/actix-web/sqlx do not compile to WASM. Either remove the targets line entirely or replace with `["x86_64-unknown-linux-gnu"]`.

**Patterns to follow:**
- `src/integrations.rs` lines 418–453 — axum `IntoResponse` for `KeyParseError`
- `src/integrations.rs` lines 467–507 — actix-web `ResponseError` for `KeyParseError`

**Test scenarios:**
- axum: `CompositeKeyParseError::MissingSeparator { separator: ':' }.into_response()` has status 400 and a non-empty body string
- axum: Simulate `Path<CompositeKey<A, B>>::from_request_parts` with a malformed path — `into_response()` returns 400 (can be verified as a compile-time type-check that the impl exists)
- actix-web: `CompositeKeyParseError::MissingSeparator { separator: ':' }.status_code()` returns `StatusCode::BAD_REQUEST`
- actix-web: `error_response()` body equals `MissingSeparator { separator: ':' }.to_string()`

**Verification:**
- `cargo check --features axum` compiles; `CompositeKeyParseError: IntoResponse` bound satisfied.
- `cargo check --features actix-web` compiles; `CompositeKeyParseError: ResponseError` bound satisfied.
- `Cargo.toml` `[package.metadata.docs.rs] features` includes `"axum"`, `"actix-web"`, `"sqlx"`.

---

## System-Wide Impact

- **Interaction graph:** `CompositeKey` has no callbacks, middleware hooks, or observers. It does not implement `KeyDomain`, so it does not participate in the existing generic impls that dispatch on `KeyDomain` bounds (`Key<D>` sqlx/serde/axum/actix-web impls are unaffected).
- **Error propagation:** `CompositeKeyParseError` surfaces at `FromStr` call sites and propagates as a sqlx `BoxDynError` or an axum/actix-web 400 response. It wraps `KeyParseError` variants — callers can match inner variants if needed.
- **State lifecycle risks:** None — `CompositeKey` is an immutable value type with no interior mutability or shared state.
- **API surface parity:** `CompositeKeyParseError` needs `IntoResponse` (axum) and `ResponseError` (actix-web) in parity with every other parse-error type in the crate. Covered in U5.
- **Integration coverage:** sqlx Decode round-trip (`encode(ck)` → `decode(value)` == `ck`) should be verified as an integration scenario even if a live DB is not available in CI — the compile-time trait assertion in U4 confirms type-level correctness.
- **Unchanged invariants:** `Key<T>`, `KeyDomain`, and all existing framework impls are unchanged. The `integrations.rs` module structure (private inner mods) is preserved — only appended to, not restructured. The `error.rs` public API grows by one type; no existing types are modified.

---

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| `debug_assert!` in constructor is elided in release builds — a first key containing `SEP` silently produces a round-trip failure with no panic or error | **Endorsed mitigation: documentation**. The constructor rustdoc must include a `# Panics (debug only)` section and a `# Caller responsibility (release)` note stating that callers must ensure component keys do not contain `SEP`. Users who need hard validation should call `from_str` or validate component strings upstream. This is a deliberate design trade-off (requirement R3); runtime validation on every call is out of scope. |
| docs.rs feature list not updated — framework impls invisible in generated documentation | U5 explicitly patches `[package.metadata.docs.rs]`. Flag in PR checklist. |
| `CompositeKeyParseError` in `src/error.rs` adds to an already large file (600+ lines) | `error.rs` is the established home; file size is not a semantic concern. A future refactor could extract per-type error modules, but that is out of scope here. |

---

## Documentation / Operational Notes

- Add `/// # CompositeKey` doc comment to `src/composite_key.rs` top-level — describe the type, default separator, and note the `debug_assert!` constructor behavior.
- Document the "split on first occurrence" rule explicitly in `FromStr` rustdoc — this determines how colons in the second component are handled.
- `CHANGELOG.md` entry for v0.8: "Add `CompositeKey<A, B, const SEP: char = ':'>` value type with serde, sqlx, axum, and actix-web integrations."
- `ROADMAP.md` v0.8 entry for `CompositeKey` can be marked as landed after this ships.

---

## Sources & References

- **Origin document:** [`docs/brainstorms/2026-05-19-composite-key-requirements.md`](docs/brainstorms/2026-05-19-composite-key-requirements.md)
- Related code: `src/key.rs` (Key<T> patterns), `src/error.rs` (KeyParseError shape), `src/integrations.rs` (sqlx/axum/actix-web), `src/lib.rs` (module wiring)
- Prior plan: [`docs/plans/2026-05-18-001-feat-arbitrary-proptest-impls-plan.md`](docs/plans/2026-05-18-001-feat-arbitrary-proptest-impls-plan.md)
