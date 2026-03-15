# domain-key

**High-performance, type-safe, domain-driven key system for Rust applications**

[![Crates.io](https://img.shields.io/crates/v/domain-key.svg)](https://crates.io/crates/domain-key)
[![Documentation](https://docs.rs/domain-key/badge.svg)](https://docs.rs/domain-key)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/vanyastaff/domain-key/workflows/CI/badge.svg)](https://github.com/vanyastaff/domain-key/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.86+-blue.svg)](https://www.rust-lang.org)

> Never mix up keys from different domains again!

## What is domain-key?

domain-key brings **Domain-Driven Design** principles to key management in Rust. It provides compile-time guarantees that keys from different business domains cannot be accidentally mixed or compared, while delivering exceptional performance through advanced optimizations.

```rust
use domain_key::{Key, Domain, KeyDomain};

// Define your business domains
#[derive(Debug)]
struct UserDomain;

#[derive(Debug)]
struct OrderDomain;

impl Domain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
}
impl KeyDomain for UserDomain {}

impl Domain for OrderDomain {
    const DOMAIN_NAME: &'static str = "order";
}
impl KeyDomain for OrderDomain {}

// Create domain-specific key types
type UserKey = Key<UserDomain>;
type OrderKey = Key<OrderDomain>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use them safely
    let user_id = UserKey::new("user_123")?;
    let order_id = OrderKey::new("order_456")?;

    // This won't compile!
    // let mixed = user_id == order_id; // Compile error!
    
    println!("User: {}", user_id.as_str());
    println!("Order: {}", order_id.as_str());
    Ok(())
}
```

## Key Features

- **Type Safety**: Different key types cannot be mixed at compile time
- **High Performance**: Up to 75% performance improvements through advanced optimizations
- **Domain Agnostic**: No built-in assumptions about specific domains
- **Memory Efficient**: Smart string handling with stack allocation for short keys
- **DoS Resistant**: Optional protection against HashDoS attacks
- **Extensible**: Easy to add new domains and validation rules
- **Zero-Cost Abstractions**: No runtime overhead for type separation
- **Cross-Platform**: Works on all major platforms including WebAssembly
- **Compile-Time Validation**: Catch invalid key literals at compile time, not at runtime

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
# Recommended for most projects (DoS-resistant hashing)
domain-key = { version = "0.4", features = ["secure"] }

# Maximum performance (requires AES-NI capable CPU)
domain-key = { version = "0.4", features = ["fast"] }

# Bare minimum (standard hasher, no extra deps)
domain-key = "0.4"
```

Define a domain and create keys:

```rust
use domain_key::{Key, Domain, KeyDomain};

// 1. Define your domain
#[derive(Debug)]
struct UserDomain;

impl Domain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
}
impl KeyDomain for UserDomain {
    const MAX_LENGTH: usize = 32;
    const TYPICALLY_SHORT: bool = true; // Optimization hint
}

// 2. Create a type alias
type UserKey = Key<UserDomain>;

// 3. Use it!
let user_key = UserKey::new("john_doe")?;
let composed_key = UserKey::from_parts(&["user", "123", "profile"], "_")?;

println!("Domain: {}", user_key.domain());
println!("Length: {}", user_key.len()); // O(1) with optimizations
println!("Key: {}", user_key.as_str());
# Ok::<(), domain_key::KeyParseError>(())
```

```rust
// Or use macros for less boilerplate:
use domain_key::{define_domain, key_type};
define_domain!(pub UserDomain, "user");
key_type!(pub UserKey, UserDomain);

// Compile-time validation — invalid literals are compile errors, not panics
const _: () = assert!(UserDomain::is_valid_key("john_doe"));
```

## Identifier Types

domain-key provides three typed identifier wrappers:

| Type | Storage | Use case |
|------|---------|----------|
| `Key<D>` | `SmartString` | Human-readable keys with validation |
| `Id<D>` | `NonZeroU64` | Numeric database IDs (8 bytes, `Copy`) |
| `Uuid<D>` | `uuid::Uuid` | UUID identifiers (16 bytes, `Copy`, feature `uuid`) |

```rust
use domain_key::prelude::*;

// Numeric IDs — one macro does it all
define_id!(UserIdDomain => UserId);

let id = UserId::new(42).unwrap();
assert_eq!(id.get(), 42);

// Or define domain and alias separately
define_id_domain!(OrderIdDomain, "order");
id_type!(OrderId, OrderIdDomain);

let order = OrderId::new(1).unwrap();
assert_eq!(order.domain(), "order");
```

```rust
// UUID identifiers (requires `uuid` feature)
use domain_key::prelude::*;

define_uuid!(OrderUuidDomain => OrderUuid);

let uuid = OrderUuid::nil();
assert_eq!(uuid.domain(), "OrderUuid");

// With v4 random generation (requires `uuid-v4` feature)
// Prefer the unified `new()` API:
// let uuid = OrderUuid::new();
```

All three types are domain-typed: `UserId` and `OrderId` are incompatible at compile time even though both wrap a `NonZeroU64`.

## Performance Features

### Feature-Based Optimization Profiles

```toml
# Maximum performance (modern CPUs with AES-NI)
features = ["fast"]

# DoS protection + good performance
features = ["secure"]

# Cryptographic security
features = ["crypto"]

# All optimizations enabled
features = ["fast", "std", "serde"]
```

### Build for Maximum Performance

```bash
# Enable CPU-specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --features="fast"

# For Apple Silicon Macs
RUSTFLAGS="-C target-cpu=native -C target-feature=+aes,+neon" cargo build --release --features="fast"
```

### Performance Improvements

| Operation | Standard | Optimized | Improvement |
|-----------|----------|-----------|-------------|
| Key Creation (short) | 100ns | 72ns | **28% faster** |
| String Operations | 100% baseline | 175% | **75% faster** |
| Struct Size | 40 bytes (old) | 32 bytes | Compact, cache-line friendly |
| HashMap Lookup | by Key (alloc) | by `&str` | **zero-alloc via `Borrow<str>`** |
| Collection Lookup | 35ns | 21ns | **40% faster** |

## Advanced Examples

### E-commerce Domain
```rust
use domain_key::{Key, Domain, KeyDomain};

#[derive(Debug)]
struct ProductDomain;

#[derive(Debug)]
struct CartDomain;

impl Domain for ProductDomain {
    const DOMAIN_NAME: &'static str = "product";
}
impl KeyDomain for ProductDomain {
    const MAX_LENGTH: usize = 32;
}

impl Domain for CartDomain {
    const DOMAIN_NAME: &'static str = "cart";
}
impl KeyDomain for CartDomain {
    const MAX_LENGTH: usize = 64;
}

type ProductKey = Key<ProductDomain>;
type CartKey = Key<CartDomain>;

// Use in your application
let product = ProductKey::new("laptop_dell_xps13")?;
let cart = CartKey::from_parts(&["cart", "user123", "session456"], "_")?;
# Ok::<(), domain_key::KeyParseError>(())
```

### Multi-tenant SaaS
```rust
use domain_key::{Key, Domain, KeyDomain, KeyParseError};
use std::borrow::Cow;

#[derive(Debug)]
struct TenantDomain;

impl Domain for TenantDomain {
    const DOMAIN_NAME: &'static str = "tenant";
}
impl KeyDomain for TenantDomain {
    const HAS_CUSTOM_VALIDATION: bool = true;
    const HAS_CUSTOM_NORMALIZATION: bool = true;
    
    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        if !key.starts_with("tenant_") {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Tenant keys must start with 'tenant_'"
            ));
        }
        Ok(())
    }
    
    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        // Convert to lowercase for consistency
        if key.chars().any(|c| c.is_ascii_uppercase()) {
            Cow::Owned(key.to_ascii_lowercase())
        } else {
            key
        }
    }
}

type TenantKey = Key<TenantDomain>;

let tenant = TenantKey::new("TENANT_acme_corp")?;
assert_eq!(tenant.as_str(), "tenant_acme_corp"); // normalized
# Ok::<(), domain_key::KeyParseError>(())
```

### Advanced Validation
```rust
use domain_key::{Key, Domain, KeyDomain, KeyParseError};

#[derive(Debug)]
struct EmailDomain;

impl Domain for EmailDomain {
    const DOMAIN_NAME: &'static str = "email";
}
impl KeyDomain for EmailDomain {
    const MAX_LENGTH: usize = 254;
    const HAS_CUSTOM_VALIDATION: bool = true;

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        if !key.contains('@') {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Email must contain @ symbol"
            ));
        }

        let parts: Vec<&str> = key.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Invalid email format"
            ));
        }

        Ok(())
    }

    fn allowed_characters(c: char) -> bool {
        c.is_ascii_alphanumeric() || "@._+-".contains(c)
    }
}

type EmailKey = Key<EmailDomain>;

let email = EmailKey::new("user@example.com")?;
assert_eq!(email.as_str(), "user@example.com");

// This will fail validation
let invalid = EmailKey::new("not-an-email");
assert!(invalid.is_err());
# Ok::<(), domain_key::KeyParseError>(())
```

> **Error variants**: `KeyParseError` includes both `TooLong { max_length, actual_length }` and `TooShort { min_length, actual_length }` variants (new in v0.4). Keys shorter than `T::min_length()` now produce a dedicated, pattern-matchable `TooShort` error instead of the generic `InvalidStructure`. Exhaustive `match` expressions on `KeyParseError` must handle both arms.

### Static Keys

Create static keys where invalid literals are compile errors (not runtime panics) since v0.4.2:

```rust
use domain_key::{define_domain, Key, static_key};

define_domain!(pub UserDomain, "user");
type UserKey = Key<UserDomain>;

// Invalid literals are compile errors (not runtime panics) since v0.4.2
let key = static_key!(UserKey, "system_admin");
assert_eq!(key.as_str(), "system_admin");
```

## Macros

The `define_domain!`, `key_type!`, `define_id!`, `define_id_domain!`, `id_type!`, `define_uuid!`, and related macros all accept an optional leading visibility specifier (`$vis:vis`). This means you can control the visibility of the generated types just like any other Rust item:

```rust
use domain_key::{define_domain, key_type};

define_domain!(pub UserDomain, "user");       // public
key_type!(pub(crate) InternalKey, UserDomain); // crate-visible
define_domain!(PrivateDomain, "private");      // private (module-local)
```

## Compile-Time Key Validation

Production code often creates keys from known-good string literals — yet the idiomatic
`Key::new("value").expect("valid")` hides a panic path that only triggers at runtime.
Version 0.4.2 eliminates that problem entirely.

### The tools

| Tool | Where | What it checks |
|------|-------|---------------|
| `is_valid_key_default(s, max)` | free `const fn` (crate root) | default rules against any length |
| `Key::<T>::is_valid_key_const(s)` | inherent `const fn` on `Key<T>` | default rules with `T::MAX_LENGTH` |
| `MyDomain::is_valid_key(s)` | generated by `define_domain!` | default rules with this domain's `MAX_LENGTH` |
| `static_key!(T, "lit")` | macro | default rules **at compile time** + custom rules at runtime |

All four work in `const` contexts — the compiler evaluates them before your binary runs.

### Before / After

**Before** — hidden panic paths scattered through the codebase:

```rust
let admin   = AdminKey::new("system_admin").expect("valid key");
let health  = CacheKey::new("health_check").expect("valid key");
let default = UserKey::new("anonymous").expect("valid key");
```

**After** — compile errors for invalid literals, zero runtime panics:

```rust
let admin   = static_key!(AdminKey,  "system_admin");
let health  = static_key!(CacheKey,  "health_check");
let default = static_key!(UserKey,   "anonymous");

// Invalid literals are caught by the compiler, not at runtime:
// let bad = static_key!(UserKey, "bad key!"); // ← compile error
```

### Document invariants next to your domain

```rust
use domain_key::{define_domain, Key};

define_domain!(pub UserDomain, "user", 64);
type UserKey = Key<UserDomain>;

// These const assertions compile to zero bytes — they are pure proof obligations.
// If MAX_LENGTH or the rules change, the assertions break the build immediately.
const _: () = assert!(UserDomain::is_valid_key("anonymous"));
const _: () = assert!(UserDomain::is_valid_key("system_admin"));
const _: () = assert!(!UserDomain::is_valid_key(""));
const _: () = assert!(!UserDomain::is_valid_key("trailing_"));
const _: () = assert!(!UserDomain::is_valid_key("bad key!"));
```

### Low-level const fn

For `build.rs`, procedural macros, or any context where you need raw validation without a
domain type:

```rust
use domain_key::{is_valid_key_default, DEFAULT_MAX_KEY_LENGTH};

const fn is_config_key_valid(s: &str) -> bool {
    is_valid_key_default(s, 32)
}

const _: () = assert!(is_config_key_valid("my_service"));
```

> **Note**: `is_valid_key_default` and `Key::is_valid_key_const` check only the *default*
> `KeyDomain` rules. Custom rules added via `validate_domain_rules` are enforced at
> runtime by `Key::new` / `Key::try_from_static`.

## Feature Flags Reference

### Hash Algorithm Features (choose one for best results)

- `fast` - GxHash (40% faster, requires modern CPU with AES-NI)
- `secure` - AHash (DoS protection, balanced performance)
- `crypto` - Blake3 (cryptographically secure)
- Default - Standard hasher (good compatibility)

> **Note:** Without any hash feature enabled, the library falls back to a basic FNV-1a hasher which is **not DoS-resistant** and is slower than ahash. For most applications, we recommend enabling the `secure` feature.

### Identifier Features

- `uuid` - Typed UUID identifiers (`Uuid<D>`)
- `uuid-v4` - UUID v4 random generation (`Uuid::new()`, `Uuid::v4()` deprecated)
- `uuid-v7` - UUID v7 time-ordered generation (`Uuid::now_v7()`)

### Core Features

- `std` - Standard library support (enabled by default)
- `serde` - Serialization support (enabled by default)
- `no_std` - No standard library support

## Security Considerations

domain-key provides multiple levels of security depending on your needs:

- **DoS Protection**: Use `secure` feature for AHash with DoS resistance
- **Cryptographic Security**: Use `crypto` feature for Blake3 cryptographic hashing
- **Input Validation**: Comprehensive validation pipeline with custom rules
- **Type Safety**: Compile-time prevention of key type mixing
- **Memory Safety**: Rust's ownership system + additional optimizations

See [SECURITY.md](SECURITY.md) for detailed security information.

## Documentation

- [User Guide](docs/guide.md) - Comprehensive usage guide
- [API Documentation](https://docs.rs/domain-key) - Complete API reference
- [Examples](examples/) - Real-world usage examples
- [Migration Guide](docs/migration.md) - Migrating from string keys
- [Performance Guide](docs/performance.md) - Optimization strategies
- [Security Policy](SECURITY.md) - Security considerations and reporting

## Testing

Run the comprehensive test suite:

```bash
# Run tests with recommended features (hash features are mutually exclusive —
# passing --all-features triggers a compile_error)
cargo test --features std,serde

# Property-based tests
cargo test --features std,serde --release -- prop_

# Benchmarks
cargo bench --features fast

# Security audit
cargo audit
```

## Benchmarks

```bash
# Run realistic benchmarks
cargo bench --features fast

# Memory usage analysis
cargo test --release memory_usage

# Cross-platform performance
cargo test --features fast --target wasm32-unknown-unknown
```

## Migration from String Keys

### Before (String-based)
```rust
let user_id: String = "user_123".to_string();
let order_id: String = "order_456".to_string();

// Dangerous - no compile-time protection!
if user_id == order_id {
    // This could be a bug, but compiler won't catch it
}

let cache_key = format!("cache:{}:{}", user_id, order_id);
```

### After (domain-key)
```rust
type UserKey = Key<UserDomain>;
type OrderKey = Key<OrderDomain>;
type CacheKey = Key<CacheDomain>;

let user_id = UserKey::new("user_123")?;
let order_id = OrderKey::new("order_456")?;

// This won't compile - type safety!
// if user_id == order_id { } // Compile error!

let cache_key = CacheKey::from_parts(&[
    "cache", 
    user_id.as_str(), 
    order_id.as_str()
], ":")?;
# Ok::<(), domain_key::KeyParseError>(())
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Development Setup

```bash
git clone https://github.com/vanyastaff/domain-key.git
cd domain-key

# Install development dependencies
rustup target add wasm32-unknown-unknown
cargo install cargo-audit cargo-hack

# Run tests (hash features are mutually exclusive; use explicit feature list)
cargo test --features std,serde
cargo clippy --features std,serde -- -D warnings
cargo fmt
```

## Platform Support

| Platform | Status | Hash Algorithm | Notes |
|----------|--------|---------------|-------|
| Linux x86_64 | Full | GxHash/AHash | Best performance with AES-NI |
| Windows x86_64 | Full | GxHash/AHash | Full feature support |
| macOS Intel | Full | GxHash/AHash | All features supported |
| macOS Apple Silicon | Full | GxHash/AHash | Requires explicit AES+NEON flags |
| WebAssembly | Core | DefaultHasher | no_std support |
| ARM64 Linux | Full | GxHash/AHash | Server deployments |
| ARM Embedded | Core | FNV-1a | no_std + no_alloc |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Domain-Driven Design principles by Eric Evans
- Built on the excellent [`smartstring`](https://crates.io/crates/smartstring) crate for memory efficiency
- Performance-focused hash algorithms from the Rust ecosystem:
  - [`ahash`](https://crates.io/crates/ahash) for DoS-resistant hashing
  - [`gxhash`](https://crates.io/crates/gxhash) for maximum performance
  - [`blake3`](https://crates.io/crates/blake3) for cryptographic security

## Project Stats

- **Test Coverage**: >95%
- **Documentation Coverage**: >98%
- **Benchmark Coverage**: 20+ realistic scenarios
- **no_std Support**: Yes
- **MSRV**: Rust 1.86+
- **Platforms**: 7+ supported targets

---

**domain-key** - Because your keys should know their place in your domain!
