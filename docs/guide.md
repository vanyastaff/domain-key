# domain-key User Guide

Welcome to the comprehensive user guide for domain-key! This guide will walk you through everything you need to know to effectively use domain-key in your Rust applications.

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Getting Started](#getting-started)
4. [Numeric and UUID Identifiers](#numeric-and-uuid-identifiers)
5. [Domain Design](#domain-design)
6. [Advanced Features](#advanced-features)
7. [Compile-Time Validation](#compile-time-validation)
8. [Performance Optimization](#performance-optimization)
9. [Common Patterns](#common-patterns)
10. [Troubleshooting](#troubleshooting)
11. [Best Practices](#best-practices)

## Introduction

domain-key is a library that brings Domain-Driven Design (DDD) principles to key management in Rust. It provides:

- **Type Safety**: Different key types cannot be mixed at compile time
- **Performance**: Optimized operations with minimal overhead
- **Flexibility**: Customizable validation and normalization per domain
- **Security**: DoS protection and cryptographic options

## Core Concepts

### Domains

A **domain** represents a bounded context in your application. Each domain has its own key type that cannot be mixed with other domains.

```rust
// User domain for user-related keys
#[derive(Debug)]
struct UserDomain;

// Order domain for order-related keys  
#[derive(Debug)]
struct OrderDomain;
```

### Domain and KeyDomain Traits

The `Domain` supertrait defines the domain name, and `KeyDomain` extends it with key-specific behavior:

```rust
use domain_key::{Domain, KeyDomain};

impl Domain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
}

impl KeyDomain for UserDomain {
    const MAX_LENGTH: usize = 32;
    const EXPECTED_LENGTH: usize = 16;
    const TYPICALLY_SHORT: bool = true;
}
```

### Keys

Keys are strongly-typed identifiers associated with a specific domain:

```rust
type UserKey = Key<UserDomain>;
type OrderKey = Key<OrderDomain>;

let user_key = UserKey::new("john_doe")?;
let order_key = OrderKey::new("order_12345")?;

// This won't compile!
// let comparison = user_key == order_key; // Compile error!
# Ok::<(), domain_key::KeyParseError>(())
```

## Getting Started

### Installation

Add domain-key to your `Cargo.toml`:

```toml
[dependencies]
domain-key = "0.4"

# Optional: Choose a feature set
domain-key = { version = "0.4", features = ["fast"] }
```

### Basic Usage

1. **Define your domains**:

```rust
use domain_key::{Key, Domain, KeyDomain};

#[derive(Debug)]
struct UserDomain;

impl Domain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
}

impl KeyDomain for UserDomain {}
```

2. **Create type aliases**:

```rust
type UserKey = Key<UserDomain>;
```

3. **Use your keys**:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user_key = UserKey::new("alice_wonderland")?;
    
    println!("User key: {}", user_key.as_str());
    println!("Domain: {}", user_key.domain());
    println!("Length: {}", user_key.len());
    
    Ok(())
}
```

### Numeric and UUID Identifiers

Beyond string keys, domain-key provides lightweight typed wrappers for numeric and UUID identifiers:

```rust
use domain_key::prelude::*;

// Numeric IDs (NonZeroU64, 8 bytes, Copy)
define_id_domain!(UserIdDomain, "user");
id_type!(UserId, UserIdDomain);

let id = UserId::new(42).unwrap();
assert_eq!(id.get(), 42);
// UserId(42) — readable Debug output

// Or use the one-liner macro:
define_id!(OrderIdDomain => OrderId);
```

For UUID identifiers, enable the `uuid` feature:

```toml
domain-key = { version = "0.4", features = ["uuid", "uuid-v4"] }
```

```rust
use domain_key::prelude::*;

define_uuid!(OrderUuidDomain => OrderUuid);
let uuid = OrderUuid::new();
```

## Domain Design

### Simple Domain

For basic use cases, minimal configuration is needed:

```rust
use domain_key::{Key, Domain, KeyDomain};

#[derive(Debug)]
struct ProductDomain;

impl Domain for ProductDomain {
    const DOMAIN_NAME: &'static str = "product";
}

impl KeyDomain for ProductDomain {
    const MAX_LENGTH: usize = 64;
}

type ProductKey = Key<ProductDomain>;
```

Or with macros:

```rust
// Or with macros:
define_domain!(ProductDomain, "product", 64);
key_type!(ProductKey, ProductDomain);
```

### Domain with Custom Validation

Add custom validation rules for your business logic:

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
        // Must contain exactly one @ symbol
        let at_count = key.chars().filter(|&c| c == '@').count();
        if at_count != 1 {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Email must contain exactly one @ symbol"
            ));
        }

        // Split into local and domain parts
        let parts: Vec<&str> = key.split('@').collect();
        let (local, domain) = (parts[0], parts[1]);

        if local.is_empty() {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Email local part cannot be empty"
            ));
        }

        if domain.is_empty() {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Email domain part cannot be empty"
            ));
        }

        Ok(())
    }

    fn allowed_characters(c: char) -> bool {
        c.is_ascii_alphanumeric() || "@._+-".contains(c)
    }
}

type EmailKey = Key<EmailDomain>;
```

### Domain with Normalization

Implement custom normalization to ensure consistency:

```rust
use std::borrow::Cow;
use domain_key::{Key, Domain, KeyDomain};

#[derive(Debug)]
struct SlugDomain;

impl Domain for SlugDomain {
    const DOMAIN_NAME: &'static str = "slug";
}

impl KeyDomain for SlugDomain {
    const HAS_CUSTOM_NORMALIZATION: bool = true;

    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        // Convert spaces to hyphens and lowercase
        let needs_normalization = key.chars().any(|c| c.is_ascii_uppercase() || c == ' ');
        
        if needs_normalization {
            let normalized = key
                .to_ascii_lowercase()
                .replace(' ', "-")
                .replace('_', "-");
            Cow::Owned(normalized)
        } else {
            key
        }
    }
}

type SlugKey = Key<SlugDomain>;

// Usage
let slug = SlugKey::new("Hello World Example")?;
assert_eq!(slug.as_str(), "hello-world-example");
# Ok::<(), domain_key::KeyParseError>(())
```

## Advanced Features

### Multi-part Keys

Create keys from multiple components:

```rust
let cache_key = CacheKey::from_parts(&[
    "user_profile",
    "123", 
    "settings"
], "_")?;

assert_eq!(cache_key.as_str(), "user_profile_123_settings");
# Ok::<(), domain_key::KeyParseError>(())
```

### Prefix and Suffix Operations

Ensure keys have required prefixes or suffixes:

```rust
let base_key = UserKey::new("alice")?;

// Add prefix if not present
let prefixed = base_key.ensure_prefix("user_")?;
assert_eq!(prefixed.as_str(), "user_alice");

// Add suffix if not present  
let versioned = prefixed.ensure_suffix("_v1")?;
assert_eq!(versioned.as_str(), "user_alice_v1");
# Ok::<(), domain_key::KeyParseError>(())
```

### Static Keys

Create compile-time validated static keys:

```rust
use domain_key::{define_domain, Key, static_key};

define_domain!(pub AdminDomain, "admin");
type AdminKey = Key<AdminDomain>;

// Since v0.4.2: invalid literals are compile errors, not runtime panics.
// The compiler evaluates is_valid_key_const at build time.
let key = static_key!(AdminKey, "system_admin");
assert_eq!(key.as_str(), "system_admin");

// You can also assert invariants at the domain definition site:
const _: () = assert!(AdminDomain::is_valid_key("system_admin"));
const _: () = assert!(!AdminDomain::is_valid_key("bad key!"));
```

### Splitting Keys

Split keys into components:

```rust
let complex_key = UserKey::new("user_123_profile_settings")?;
let parts: Vec<&str> = complex_key.split('_').collect();
assert_eq!(parts, vec!["user", "123", "profile", "settings"]);
# Ok::<(), domain_key::KeyParseError>(())
```

## Compile-Time Validation

domain-key v0.4.2 introduces a set of `const fn` tools that let you verify key literals at
compile time. This section explains each tool and when to reach for it.

### The problem

A common pattern in production Rust code is:

```rust
let admin_key   = AdminKey::new("system_admin").expect("valid");
let health_key  = CacheKey::new("health_check").expect("valid");
let default_key = UserKey::new("anonymous").expect("valid");
```

Each `.expect()` is a hidden panic path that only fires at runtime — during program startup
or the first time the function is called. If a key literal is accidentally broken (e.g. a
typo, a domain rule change, a `MAX_LENGTH` reduction), you find out in production, not at
`cargo build`.

### Solution overview

| Tool | Kind | Use case |
|------|------|---------|
| `is_valid_key_default(s, max)` | free `const fn` | raw validation, `build.rs`, proc-macros |
| `Key::<T>::is_valid_key_const(s)` | inherent `const fn` | any key type, `const` assertions |
| `MyDomain::is_valid_key(s)` | generated `const fn` | per-domain assertions, self-documenting code |
| `static_key!(T, "lit")` | macro | production static keys with compile-error guarantee |

### `is_valid_key_default`

The foundation. A pure `const fn` that checks the *default* `KeyDomain` rules:

- `s` is non-empty
- `s.len() <= max_length`
- every byte is ASCII alphanumeric, `_`, `-`, or `.`
- the last byte is alphanumeric (not `_`, `-`, `.`)
- no consecutive identical separators (`__`, `--`, `..`)

```rust
use domain_key::{is_valid_key_default, DEFAULT_MAX_KEY_LENGTH};

const VALID:        bool = is_valid_key_default("user_123",    DEFAULT_MAX_KEY_LENGTH);
const EMPTY:        bool = is_valid_key_default("",            DEFAULT_MAX_KEY_LENGTH);
const TRAILING_SEP: bool = is_valid_key_default("foo_",        DEFAULT_MAX_KEY_LENGTH);
const CONSECUTIVE:  bool = is_valid_key_default("a__b",        DEFAULT_MAX_KEY_LENGTH);

assert!(VALID);
assert!(!EMPTY);
assert!(!TRAILING_SEP);
assert!(!CONSECUTIVE);
```

Because it is a `const fn` these four evaluations happen entirely at compile time — the
binary contains only the four boolean constants, not the validation logic.

### `Key::<T>::is_valid_key_const`

A thin wrapper that supplies `T::MAX_LENGTH` automatically:

```rust
use domain_key::{Key, Domain, KeyDomain};

#[derive(Debug)]
struct UserDomain;
impl Domain for UserDomain { const DOMAIN_NAME: &'static str = "user"; }
impl KeyDomain for UserDomain { const MAX_LENGTH: usize = 32; }
type UserKey = Key<UserDomain>;

// Compile-time assertion: evaluates at build time, zero runtime cost
const _: () = assert!(UserKey::is_valid_key_const("alice"));
const _: () = assert!(!UserKey::is_valid_key_const(""));
const _: () = assert!(!UserKey::is_valid_key_const("alice_")); // trailing sep
```

### `MyDomain::is_valid_key` (auto-generated by `define_domain!`)

When you use `define_domain!`, an `is_valid_key` method is automatically generated on the
struct. It lets you place invariant assertions right next to the domain definition:

```rust
use domain_key::{define_domain, Key};

define_domain!(pub UserDomain, "user", 64);
type UserKey = Key<UserDomain>;

// These const assertions are zero-cost proof obligations.
// If MAX_LENGTH is reduced or allowed_characters change, the build breaks immediately.
const _: () = assert!(UserDomain::is_valid_key("anonymous"));
const _: () = assert!(UserDomain::is_valid_key("system_admin"));
const _: () = assert!(!UserDomain::is_valid_key(""));
const _: () = assert!(!UserDomain::is_valid_key("bad key!"));
const _: () = assert!(!UserDomain::is_valid_key("trailing_"));
```

### Enhanced `static_key!`

`static_key!` now embeds a `const` assertion, so invalid literals are compile errors:

```rust
use domain_key::{define_domain, Key, static_key};

define_domain!(pub UserDomain, "user", 64);
type UserKey = Key<UserDomain>;

// ✅ Compiles — literal passes default rules
let key = static_key!(UserKey, "anonymous");

// ❌ Does NOT compile — space is not an allowed character
// let bad = static_key!(UserKey, "bad key");
```

Before v0.4.2, an invalid literal in `static_key!` caused a runtime panic on the first
call. Now it is a hard compile error — impossible to ship.

### What is NOT checked at compile time

Custom domain rules added via `validate_domain_rules` are enforced at runtime by
`Key::new` / `Key::try_from_static`. The const fns only verify the *default* rules.
`static_key!` still calls `try_from_static` after the compile-time check to enforce any
custom rules — a panic there means the key literal violates custom rules and must be fixed.

### Recommended patterns

**Pattern 1 — document invariants at domain definition:**
```rust
define_domain!(pub SessionDomain, "session", 128);
// Every valid session key satisfies these constraints at compile time:
const _: () = assert!(SessionDomain::is_valid_key("sess_abc123"));
const _: () = assert!(!SessionDomain::is_valid_key(""));
```

**Pattern 2 — production static keys without expect:**
```rust
// Before (hidden panic):
fn default_session() -> SessionKey {
    SessionKey::new("anonymous").expect("valid key")
}

// After (compile error if invalid):
fn default_session() -> SessionKey {
    static_key!(SessionKey, "anonymous")
}
```

**Pattern 3 — const in build.rs or proc-macros:**
```rust
use domain_key::is_valid_key_default;

const fn validate_config(key: &str) -> bool {
    is_valid_key_default(key, 64)
}
const _: () = assert!(validate_config("my_service_key"));
```

## Performance Optimization

### Feature Selection

Choose the right features for your use case:

```toml
# Maximum performance (production)
domain-key = { version = "0.4", features = ["fast"] }

# Security-focused
domain-key = { version = "0.4", features = ["secure"] }

# Cryptographic applications
domain-key = { version = "0.4", features = ["crypto"] }
```

### Domain Configuration

Optimize domains for your usage patterns:

```rust
use domain_key::{Domain, KeyDomain};

#[derive(Debug)]
struct HighPerformanceDomain;

impl Domain for HighPerformanceDomain {
    const DOMAIN_NAME: &'static str = "fast";
}

impl KeyDomain for HighPerformanceDomain {
    const MAX_LENGTH: usize = 32;          // Reasonable limit
    const EXPECTED_LENGTH: usize = 16;     // Optimization hint
    const TYPICALLY_SHORT: bool = true;    // Enable stack allocation
}
```

### Bulk Operations

Process multiple keys efficiently:

```rust
// Efficient batch creation
let user_ids: Result<Vec<UserKey>, _> = (1..=1000)
    .map(|id| UserKey::new(format!("user_{}", id)))
    .collect();

let users = user_ids?;
# Ok::<(), domain_key::KeyParseError>(())
```

## Common Patterns

### Web Application Keys

```rust
use std::borrow::Cow;
use domain_key::{Key, Domain, KeyDomain, KeyParseError};

// Session management
#[derive(Debug)]
struct SessionDomain;

impl Domain for SessionDomain {
    const DOMAIN_NAME: &'static str = "session";
}

impl KeyDomain for SessionDomain {
    const MAX_LENGTH: usize = 64;
    
    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        // Sessions should be alphanumeric only
        if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Session keys must be alphanumeric"
            ));
        }
        Ok(())
    }
}

type SessionKey = Key<SessionDomain>;

// API endpoint routing
#[derive(Debug)]
struct RouteDomain;

impl Domain for RouteDomain {
    const DOMAIN_NAME: &'static str = "route";
}

impl KeyDomain for RouteDomain {
    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        // Normalize routes to lowercase with forward slashes
        let normalized = key.to_ascii_lowercase().replace('\\', "/");
        if normalized != key.as_ref() {
            Cow::Owned(normalized)
        } else {
            key
        }
    }
}

type RouteKey = Key<RouteDomain>;
```

### Database Keys

```rust
use domain_key::{Key, Domain, KeyDomain, KeyParseError};

// Primary keys
#[derive(Debug)]
struct EntityIdDomain;

impl Domain for EntityIdDomain {
    const DOMAIN_NAME: &'static str = "entity_id";
}

impl KeyDomain for EntityIdDomain {
    const MAX_LENGTH: usize = 36; // UUID length
    
    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        // Validate UUID format (simplified)
        if key.len() == 36 && key.chars().filter(|&c| c == '-').count() == 4 {
            Ok(())
        } else {
            Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Must be a valid UUID format"
            ))
        }
    }
}

type EntityId = Key<EntityIdDomain>;

// Foreign keys
#[derive(Debug)]
struct ForeignKeyDomain;

impl Domain for ForeignKeyDomain {
    const DOMAIN_NAME: &'static str = "foreign_key";
}

impl KeyDomain for ForeignKeyDomain {
    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        // Foreign keys should reference valid entity IDs
        if !key.starts_with("ref_") {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Foreign keys must start with 'ref_'"
            ));
        }
        Ok(())
    }
}

type ForeignKey = Key<ForeignKeyDomain>;
```

### Cache Keys

```rust
use std::borrow::Cow;
use domain_key::{Key, Domain, KeyDomain};

#[derive(Debug)]
struct CacheDomain;

impl Domain for CacheDomain {
    const DOMAIN_NAME: &'static str = "cache";
}

impl KeyDomain for CacheDomain {
    const MAX_LENGTH: usize = 250; // Redis key limit
    
    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        // Replace problematic characters for cache systems
        if key.contains(':') || key.contains(' ') {
            let normalized = key.replace(':', "_").replace(' ', "_");
            Cow::Owned(normalized)
        } else {
            key
        }
    }
}

type CacheKey = Key<CacheDomain>;

// Usage patterns
let user_cache = CacheKey::from_parts(&["user", "123", "profile"], ":")?;
let session_cache = CacheKey::from_parts(&["session", "abc123"], ":")?;
# Ok::<(), domain_key::KeyParseError>(())
```

## Troubleshooting

### Common Compilation Errors

**Error**: Cannot compare keys from different domains
```rust
// This won't compile
let user_key = UserKey::new("alice")?;
let order_key = OrderKey::new("order_123")?;
// let comparison = user_key == order_key; // Error!
```

**Solution**: This is intentional! Use the string representation if you need to compare:
```rust
let comparison = user_key.as_str() == order_key.as_str();
```

**Error**: Key validation failed
```rust
let result = UserKey::new("invalid key with spaces");
// Returns Err(KeyParseError::InvalidCharacter { ... })
```

**Solution**: Check your domain's `allowed_characters` method and validation rules.

### Runtime Errors

**KeyParseError::Empty**
```rust
let empty_key = UserKey::new(""); // Error: Key cannot be empty
```

**KeyParseError::TooLong**
```rust
let long_key = UserKey::new(&"x".repeat(1000)); // Error: exceeds MAX_LENGTH
```

**KeyParseError::DomainValidation**
```rust
let invalid_email = EmailKey::new("not-an-email"); // Domain validation failed
```

### Performance Issues

**Slow key creation**: Check if you have expensive validation logic
**High memory usage**: Consider reducing MAX_LENGTH or using shorter keys
**Hash collisions**: Use the `secure` or `crypto` features

## Best Practices

### Domain Design

1. **Keep domains focused**: Each domain should represent a single bounded context
2. **Use descriptive names**: Domain names should clearly indicate their purpose
3. **Set appropriate limits**: Configure MAX_LENGTH based on your use case
4. **Validate early**: Implement validation rules that catch errors early
5. **Prefer macros for simple domains**: Use `define_domain!` and `key_type!` to reduce boilerplate
6. **Document key invariants with const assertions**: Use `MyDomain::is_valid_key` in `const _: ()` assertions next to your domain definition. This turns implicit assumptions about valid keys into compile-checked proof obligations.

### Performance

1. **Choose the right features**: Use `fast` for speed, `secure` for protection
2. **Optimize for your use case**: Configure EXPECTED_LENGTH and TYPICALLY_SHORT
3. **Batch operations**: Process multiple keys together when possible
4. **Cache keys**: Reuse keys instead of creating them repeatedly

### Security

1. **Use appropriate hash algorithms**: `secure` for web apps, `crypto` for sensitive data
2. **Validate inputs**: Don't trust external input, always validate
3. **Limit key length**: Prevent DoS attacks with reasonable length limits
4. **Sanitize output**: Be careful when logging or displaying keys

### Code Organization

1. **Centralize domain definitions**: Keep all domains in a dedicated module
2. **Use type aliases**: Make keys easy to use with meaningful aliases
3. **Document validation rules**: Explain why certain rules exist
4. **Test thoroughly**: Cover both valid and invalid cases

### Example Project Structure

```
src/
├── lib.rs
├── domains/
│   ├── mod.rs
│   ├── user.rs      // UserDomain definition
│   ├── order.rs     // OrderDomain definition
│   └── cache.rs     // CacheDomain definition
├── keys/
│   ├── mod.rs
│   └── aliases.rs   // Type aliases (UserKey, OrderKey, etc.)
└── validation/
    ├── mod.rs
    └── helpers.rs   // Common validation helpers
```

## Next Steps

- Read the [Migration Guide](migration.md) to convert from string keys
- Check out the [Performance Guide](performance.md) for optimization tips
- Browse the [examples/](../examples/) directory for real-world usage
- Explore the [API documentation](https://docs.rs/domain-key) for complete reference

---

Happy coding with domain-key!