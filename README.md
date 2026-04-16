# domain-key

Type-safe domain identifiers for Rust.

[![Crates.io](https://img.shields.io/crates/v/domain-key.svg)](https://crates.io/crates/domain-key)
[![Documentation](https://docs.rs/domain-key/badge.svg)](https://docs.rs/domain-key)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/vanyastaff/domain-key/workflows/CI/badge.svg)](https://github.com/vanyastaff/domain-key/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.86+-blue.svg)](https://www.rust-lang.org)

`domain-key` helps you avoid mixing IDs from different domains.
`UserId` and `OrderId` are different types at compile time, even if both wrap numbers or UUIDs.

## Install

```toml
[dependencies]
# Recommended default: DoS-resistant hashing
domain-key = { version = "0.5.2", features = ["secure"] }

# Max performance (requires AES-capable CPU)
# domain-key = { version = "0.5.2", features = ["fast"] }
```

## Quick Start

```rust
use domain_key::{Domain, Key, KeyDomain};

#[derive(Debug)]
struct UserDomain;
impl Domain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
}
impl KeyDomain for UserDomain {}

#[derive(Debug)]
struct OrderDomain;
impl Domain for OrderDomain {
    const DOMAIN_NAME: &'static str = "order";
}
impl KeyDomain for OrderDomain {}

type UserKey = Key<UserDomain>;
type OrderKey = Key<OrderDomain>;

let user = UserKey::new("user_123")?;
let order = OrderKey::new("order_456")?;

// This does not compile:
// let bad = user == order;

# Ok::<(), domain_key::KeyParseError>(())
```

## Identifier Types

| Type | Storage | Typical use |
|------|---------|-------------|
| `Key<D>` | `SmartString` | Human-readable validated keys |
| `Id<D>` | `NonZeroU64` | Numeric DB/entity identifiers |
| `Uuid<D>` | `uuid::Uuid` | Typed UUID identifiers (`uuid` feature) |
| `Ulid<D>` | `ulid::Ulid` | Typed prefixed ULIDs (`ulid` feature) |

## Macro-first API

```rust
use domain_key::prelude::*;

define_id!(UserIdDomain => UserId);
define_uuid!(OrderUuidDomain => OrderUuid);
define_ulid!(pub ExecutionIdDomain => ExecutionId, prefix = "exe");

let user = UserId::new(42).unwrap();
let order = OrderUuid::nil();
let exec = ExecutionId::new();

assert_eq!(user.get(), 42);
assert!(exec.to_string().starts_with("exe_"));
```

## Key Features

- compile-time domain safety
- fast, low-allocation string key storage
- optional typed UUID and ULID wrappers
- optional SQLx + web framework integrations
- `no_std` support for core key logic

## Feature Flags

### Hash Profiles

- `secure` (recommended): `AHash`
- `fast`: `GxHash` (AES-required)
- `crypto`: `Blake3`

### Identifier Features

- `uuid`, `uuid-v4`, `uuid-v7`
- `ulid`, `ulid-monotonic`

### Integration Features

- `sqlx-postgres`, `sqlx-sqlite`, `sqlx-mysql`
- `axum`
- `actix-web`

## SQLx Storage Model

| Type | Postgres | SQLite/MySQL |
|------|----------|--------------|
| `Key<D>` | string carrier | string carrier |
| `Id<D>` | `BIGINT` (`i64`) | integer (`i64`) |
| `Uuid<D>` | native UUID | UUID string |
| `Ulid<D>` | native UUID (binary) | prefixed ULID string |

## Byte Accessor Note (UUID/ULID)

- `Uuid<D>` uses `as_bytes()`.
- `Ulid<D>` now also exposes `as_bytes()`.
- `Ulid<D>::to_bytes()` is kept as a deprecated compatibility shim.

## Docs

- [Guide](docs/guide.md)
- [API docs](https://docs.rs/domain-key)
- [Migration guide](docs/migration.md)
- [Performance guide](docs/performance.md)
- [Security policy](SECURITY.md)

## Development

```bash
cargo test --features std,serde
cargo clippy --features std,serde -- -D warnings
cargo fmt --all -- --check
```

## License

MIT. See [LICENSE](LICENSE).
