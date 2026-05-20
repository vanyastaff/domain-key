# domain-key Roadmap

## Vision

`domain-key` was created to solve one concrete, recurring pain point: every Rust project
ends up with a dozen copies of `struct UserId(String)` and 20+ lines of boilerplate
around each one. The goal of this library is to make that pattern disappear — one line,
fully type-safe, validated, and ready to plug into any framework or database without
friction.

The path to `1.0` is therefore not just "stable API" — it is:

> A library that a developer opens, understands in 5 minutes, drops into their
> Axum + SQLx project, and never writes `struct UserId(String)` by hand again.

---

## Guiding Principles

- **Zero friction adoption** — the first example must be immediately obvious
- **Integrate, don't isolate** — works with SQLx, Axum, Serde, OpenAPI out of the box
via feature flags, no extra crates for the user to add
- **One crate, one dependency** — all integrations live inside `domain-key` behind
optional features; the only exception is `domain-key-derive` (proc-macro requirement),
but it is transparent to the user via `features = ["derive"]`
- **No surprises** — `no_std` works, `Copy` types stay `Copy`, errors carry context
- **Performance is a feature** — every change must not regress benchmarks

---

## Integration Strategy: Feature Flags, Not Extra Crates

All framework and tool integrations are shipped inside this single crate as optional
dependencies activated by feature flags. The user writes:

```toml
[dependencies]
domain-key = { version = "0.5", features = ["sqlx", "axum", "derive", "secure"] }
```

No `domain-key-sqlx`, no `domain-key-axum`. The single exception is the proc-macro
crate `domain-key-derive`, which Rust requires to be a separate crate — but it is
pulled in automatically via `features = ["derive"]`, exactly like `serde`/`serde_derive`.

Target `Cargo.toml` shape at `1.0`:

```toml
[dependencies]
# Core (always)
smartstring = { version = "1.0", default-features = false }
thiserror   = { version = "2.0", default-features = false }

# Serialization
serde = { version = "1.0", optional = true, default-features = false, features = ["derive"] }

# Identifiers
uuid = { version = "1", optional = true, default-features = false }

# Hash algorithms (choose one)
ahash  = { version = "0.8", optional = true, default-features = false }
blake3 = { version = "1.5", optional = true, default-features = false }
gxhash = { version = "3.0", optional = true, default-features = false }

# Framework integrations
sqlx          = { version = "0.8", optional = true, default-features = false }
axum          = { version = "0.7", optional = true, default-features = false }
actix-web     = { version = "4",   optional = true, default-features = false }
poem          = { version = "3",   optional = true, default-features = false }

# API / Schema
utoipa        = { version = "4",   optional = true, default-features = false }
async-graphql = { version = "7",   optional = true, default-features = false }

# Error presentation
miette = { version = "5", optional = true, default-features = false, features = ["fancy"] }

# Derive macros — transparent to user via features = ["derive"]
domain-key-derive = { version = "=1.0", optional = true }

[features]
default = ["std", "serde"]

# Core
std    = ["thiserror/std", "serde?/std", "smartstring/std"]
serde  = ["dep:serde", "smartstring/serde", "uuid?/serde"]
derive = ["dep:domain-key-derive"]

# Hash
fast   = ["dep:gxhash", "dep:ahash"]
secure = ["dep:ahash"]
crypto = ["dep:blake3"]

# Identifiers
uuid    = ["dep:uuid", "uuid/std"]
uuid-v4 = ["uuid", "uuid/v4"]
uuid-v7 = ["uuid", "uuid/v7"]

# Integrations
sqlx          = ["dep:sqlx", "std"]
axum          = ["dep:axum", "std"]
actix-web     = ["dep:actix-web", "std"]
poem          = ["dep:poem", "std"]
utoipa        = ["dep:utoipa", "serde"]
async-graphql = ["dep:async-graphql", "std"]
miette        = ["dep:miette", "std"]

# Convenience bundles
web  = ["axum", "serde"]
full = ["std", "serde", "derive", "secure", "uuid-v4", "miette"]
```

---

## Release Plan

### v0.5 — Proc-macro Derive: Remove the Last Barrier

**Goal:** make adoption instant. A developer should be able to define a typed key in
one line using a familiar `#[derive(...)]` syntax.

New crate `domain-key-derive` (transparent via `features = ["derive"]`):

```rust
// Before — even with macros, you need to know the API
define_domain!(pub UserDomain, "user", 64);
type UserId = Key<UserDomain>;

// After — familiar to every Rust developer
#[derive(KeyDomain)]
#[domain(name = "user", max_length = 64)]
pub struct UserDomain;

type UserId = Key<UserDomain>;

// Or the ultra-short one-liner:
#[derive(TypedKey)]
pub struct UserId;   // domain name inferred from struct name: "user_id" → "userid"
```

- `#[derive(KeyDomain)]` — generates `impl Domain + impl KeyDomain`
- `#[derive(TypedKey)]` — one-liner: generates domain + type alias
- `#[derive(TypedId)]` — one-liner for `Id<D>`
- `#[derive(TypedUuid)]` — one-liner for `Uuid<D>`
- Attributes: `#[domain(name, max_length, min_length, case_insensitive, separator)]`
- Attribute `#[domain(validate = "my_fn")]` — custom validation via function pointer
- Remove deprecated `Key::split_cached` (deprecated since 0.4.0)
- Unify `Key::from_parts` / `try_from_parts` into a single `Result`-returning method
- `KeyParseError` variants carry the original input string in all variants

---

### v0.6 — Framework Integrations (SQLx + Axum)

**Goal:** `Key<D>` and `Id<D>` work transparently in the most common Rust stack
(Axum web API + SQLx database) with zero extra code from the user.

`**features = ["sqlx"]`:**

```rust
// Transparent DB column mapping
#[derive(sqlx::FromRow)]
struct User {
    id:    UserId,    // Id<UserDomain>  →  BIGINT / INTEGER
    email: EmailKey,  // Key<EmailDomain> → TEXT
    slug:  SlugKey,   // Key<SlugDomain>  → TEXT
}

// Works with query macros too
let user = sqlx::query_as!(User, "SELECT id, email, slug FROM users WHERE id = $1", id.get())
    .fetch_one(&pool)
    .await?;
```

- `impl sqlx::Type<DB>` for `Key<D>`, `Id<D>`, `Uuid<D>`
- `impl sqlx::Encode<DB>` for all three types
- `impl sqlx::Decode<DB>` for all three types
- PostgreSQL, SQLite, MySQL driver support via `sqlx` feature flags
- `impl sqlx::postgres::PgHasArrayType` for `Key<D>` (array columns)

`**features = ["axum"]`:**

```rust
// Path extractor — already validated by the time handler runs
async fn get_user(Path(user_id): Path<UserId>) -> impl IntoResponse {
    // user_id: UserId — already validated, no manual parsing
}

// Query parameters
async fn search(Query(params): Query<SearchParams>) -> impl IntoResponse {
    // params.user_id: Option<UserId>
}
```

- `impl axum::extract::FromRequestParts` via `FromStr` — automatic `Path<Key<D>>`
- `impl axum::response::IntoResponse` for `KeyParseError` (returns 400 Bad Request)
- Example in docs: full Axum handler with `Key<D>` path + `Id<D>` path

`**features = ["actix-web"]`:**

- `impl actix_web::FromRequest` for `Key<D>` and `Id<D>`
- `impl actix_web::ResponseError` for `KeyParseError`

---

### v0.7 — API / Schema Integrations (OpenAPI + GraphQL)

**Goal:** typed keys appear correctly in generated API schemas without any extra
annotations.

`**features = ["utoipa"]`:**

```rust
// Schema generated automatically
#[derive(TypedKey, utoipa::ToSchema)]
#[domain(name = "user")]
pub struct UserKey;
// Generates: { "type": "string", "description": "user domain key, max 64 chars" }
```

- `impl utoipa::ToSchema` for `Key<D>`, `Id<D>`, `Uuid<D>`
- Schema description includes domain name, max length, validation hint
- Example in docs

`**features = ["async-graphql"]`:**

```rust
// GraphQL scalar — works transparently
#[Object]
impl Query {
    async fn user(&self, id: UserId) -> Option<User> { ... }
}
```

- `impl async_graphql::Scalar` for `Key<D>` and `Id<D>`
- Scalar description pulled from `KeyDomain::validation_help()`
- Example in docs

`**features = ["poem"]`:**

- `impl poem::web::FromRequest` for `Key<D>` and `Id<D>`
- `poem-openapi` schema impl behind nested feature

---

### v0.8 — Built-in Domain Presets

**Goal:** the user should not have to invent `EmailDomain` or `SlugDomain` — common
patterns come batteries-included.

New module `domain_key::presets`:

```rust
use domain_key::presets::*;

type BlogSlug   = Key<SlugDomain>;     // kebab-case slug: "my-blog-post"
type UserEmail  = Key<EmailDomain>;    // basic email structure validation
type ApiToken   = Key<TokenDomain>;    // hex / base64url tokens
type AppVersion = Key<SemVerDomain>;   // semver-like: "1.2.3"
type LocaleKey  = Key<LocaleDomain>;   // BCP-47 locale: "en-US"
```

- `SlugDomain` — kebab-case, lowercase, no consecutive hyphens
- `TokenDomain` — alphanumeric + `_-`, fixed length range
- `EmailDomain` — structural email-like validation (no full RFC 5321, practical)
- `SemVerDomain` — major.minor.patch pattern
- `LocaleDomain` — `xx-XX` locale codes
- `CurrencyDomain` — ISO 4217 three-letter codes (USD, EUR, ...)
- Each preset: documented, with examples, `test_domain!` coverage

`**CompositeKey<A, B>`:**

```rust
type UserPostKey = CompositeKey<UserDomain, PostDomain>;
let key = UserPostKey::new("user-42", "post-7")?;  // → "user-42:post-7"
key.first()   // → Key<UserDomain>
key.second()  // → Key<PostDomain>
```

- `CompositeKey<A, B>` struct with typed accessors
- Configurable separator (default `:`)
- `serde`, `Display`, `FromStr` implementations
- `sqlx` / `axum` support via respective features

---

### v0.9 — Developer Experience: Errors + Diagnostics

**Goal:** when something goes wrong, the developer understands immediately what and why.

`**features = ["miette"]`:**

```
error[E001]: invalid character in key
  --> src/main.rs:5:20
   |
 5 | UserKey::new("bad key!")
   |              ^^^-----^^
   |              |  |
   |              |  invalid character ' ' at position 3
   |              key value starts here
  = help: keys may only contain [a-z0-9_.\-]
  = note: did you mean "bad_key"?
```

- `KeyParseError` implements `miette::Diagnostic`
- All variants carry `source_input: String` (the original string passed by the user)
- Each variant has at least one `#[help]` suggestion
- `miette`-based rendering opt-in via feature flag, plain `Display` unchanged

**Error improvements (no feature required):**

- `#![deny(missing_docs)]` — raise from `warn` to `deny`
- Every `KeyParseError` variant carries the original input string
- `KeyParseError::suggestion()` returns a concrete corrected string when possible
(e.g. trims whitespace, lowercases for case-insensitive domains)

---

### v0.10 — Testing, Hardening & Documentation

**Goal:** production confidence — fuzz-tested, miri-clean, coverage > 95%.

**Testing:**

- `proptest` — property-based tests for validator, normalizer, hash consistency
- `cargo-fuzz` target for `Key::new` and `Key::from_parts`
- Miri clean run (`cargo miri test`) — catch any UB even without `unsafe`
- Cross-platform CI matrix: Linux x86_64, Windows, macOS ARM, `wasm32-unknown-unknown`
- `no_std` smoke test on `thumbv7m-none-eabi` in CI
- Coverage > 95% via `cargo-tarpaulin`, reported in CI

**Quality gates:**

- `cargo-semver-checks` in CI — fail on accidental breaking changes
- `cargo-deny` fully configured: licenses, advisories, duplicate deps
- `cargo-mutants` — mutation testing to find undertested logic
- Criterion baseline locked in CI — fail on > 5% performance regression

**Documentation:**

- `mdBook` deployed to GitHub Pages from `docs/`
- Every public item has a runnable `# Examples` block
- `docs/adr/` — Architecture Decision Records for key design choices
(SmartString, hash strategy, PhantomData domain marker, feature flag design)
- `docs/comparison.md` — comparison with `nutype`, manual newtype, `typed-index`

**Real-world examples:**

- `examples/rest-api/` — full Axum + SQLx application
- `examples/graphql/` — async-graphql with typed keys
- `examples/event-sourcing/` — aggregate / event / snapshot keys
- `examples/cli-tool/` — clap integration, `FromStr` for CLI args

---

### v0.10 → v1.0 — API Freeze & Release

**Goal:** nothing new, only stabilisation. Announce API freeze publicly.

- Remove all remaining `#[deprecated]` items
- Remove all `#[doc(hidden)]` from stable public API (or explicitly mark as unstable)
- Migrate to `edition = "2024"` (MSRV is already 1.86+)
- Final MSRV decision locked in `rust-version`
- CHANGELOG reviewed — every breaking change has a migration note
- crates.io metadata reviewed: keywords, categories, description
- Public announcement: "API is frozen, 1.0 imminent"
- Two-week baking period — community feedback only, no new features

---

## v1.0.0 — Release Criteria Checklist


| Category         | Requirement                                                               |
| ---------------- | ------------------------------------------------------------------------- |
| **API**          | No `#[deprecated]` items; all public API documented                       |
| **Derive**       | `#[derive(TypedKey/TypedId/TypedUuid)]` works via `features = ["derive"]` |
| **Integrations** | SQLx + Axum work out of the box via feature flags                         |
| **Presets**      | `SlugDomain`, `TokenDomain`, `EmailDomain` in `domain_key::presets`       |
| **Errors**       | Original input string in every error variant; `miette` feature available  |
| **Docs**         | `#![deny(missing_docs)]`; mdBook live; every item has runnable example    |
| **Testing**      | proptest, miri, fuzz, CI on 4 platforms, coverage > 95%                   |
| **Performance**  | Regression gate in CI; benchmark table up to date in README               |
| `**no_std`**     | Smoke test on embedded target in CI                                       |
| **Semver**       | `cargo-semver-checks` in CI; no accidental breaking changes               |
| **Security**     | `cargo-audit`, `cargo-deny`, SECURITY.md up to date                       |
| **Edition**      | `edition = "2024"`                                                        |


---

## Priority by User Impact

```
Highest adoption impact:
  1. #[derive(TypedKey)]       ← removes last boilerplate barrier
  2. SQLx integration          ← 90% of projects use a database
  3. Axum / Actix integration  ← 90% of projects build a web API
  4. domain_key::presets       ← no need to invent EmailDomain yourself
  5. miette errors             ← great DX during development
  6. Real-world examples       ← "show me working code"
```

---

## What Will NOT Be Separate Crates


| Integration           | Mechanism                                              | Separate crate?                                      |
| --------------------- | ------------------------------------------------------ | ---------------------------------------------------- |
| `sqlx`                | `optional` dep + `#[cfg(feature = "sqlx")]` impls      | No                                                   |
| `axum`                | `optional` dep + `#[cfg(feature = "axum")]` impls      | No                                                   |
| `actix-web`           | `optional` dep + `#[cfg(feature = "actix-web")]` impls | No                                                   |
| `poem`                | `optional` dep + `#[cfg(feature = "poem")]` impls      | No                                                   |
| `utoipa`              | `optional` dep + `impl ToSchema`                       | No                                                   |
| `async-graphql`       | `optional` dep + `impl Scalar`                         | No                                                   |
| `miette`              | `optional` dep + `impl Diagnostic`                     | No                                                   |
| `#[derive(TypedKey)]` | proc-macro — Rust requires separate crate              | **Yes**, but transparent via `features = ["derive"]` |


---

## Version Summary


| Version | Theme         | Key Deliverable                                               |
| ------- | ------------- | ------------------------------------------------------------- |
| `0.4.2` | *(current)*   | `const` validation, `static_key!` compile errors              |
| `0.5`   | Derive macros | `#[derive(TypedKey)]` via `features = ["derive"]`             |
| `0.6`   | DB + Web      | `sqlx`, `axum`, `actix-web` feature integrations              |
| `0.7`   | API schemas   | `utoipa`, `async-graphql`, `poem` feature integrations        |
| `0.8`   | Presets       | `domain_key::presets`, `CompositeKey<A, B>`                   |
| `0.9`   | DX            | `miette` errors, source input in errors, `deny(missing_docs)` |
| `0.10`  | Hardening     | proptest, fuzz, miri, CI matrix, mdBook                       |
| `1.0`   | Stable        | API freeze, edition 2024, release                             |


---

## Links

- [Repository](https://github.com/vanyastaff/domain-key)
- [Crates.io](https://crates.io/crates/domain-key)
- [Documentation](https://docs.rs/domain-key)
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)
- [User Guide](docs/guide.md)
- [Migration Guide](docs/migration.md)
- [Performance Guide](docs/performance.md)

