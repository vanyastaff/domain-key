# domain-key User Guide

Welcome to the comprehensive user guide for domain-key! This guide will walk you through everything you need to know to effectively use domain-key in your Rust applications.

## Table of Contents

1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Getting Started](#getting-started)
4. [Domain Design](#domain-design)
5. [Advanced Features](#advanced-features)
6. [Performance Optimization](#performance-optimization)
7. [Common Patterns](#common-patterns)
8. [Troubleshooting](#troubleshooting)
9. [Best Practices](#best-practices)

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

// Order domain for order-related keys  
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct OrderDomain;
```

### Key Domain Trait

The `KeyDomain` trait defines the behavior for each domain:

```rust
impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
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
domain-key = "0.1"

# Optional: Choose a feature set
domain-key = { version = "0.1", features = ["fast"] }
```

### Basic Usage

1. **Define your domains**:

```rust
use domain_key::{Key, KeyDomain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
}
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

## Domain Design

### Simple Domain

For basic use cases, minimal configuration is needed:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ProductDomain;

impl KeyDomain for ProductDomain {
    const DOMAIN_NAME: &'static str = "product";
    const MAX_LENGTH: usize = 64;
}

type ProductKey = Key<ProductDomain>;
```

### Domain with Custom Validation

Add custom validation rules for your business logic:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct EmailDomain;

impl KeyDomain for EmailDomain {
    const DOMAIN_NAME: &'static str = "email";
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct SlugDomain;

impl KeyDomain for SlugDomain {
    const DOMAIN_NAME: &'static str = "slug";
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
use domain_key::static_key;

// This is validated at compile time
let static_user = static_key!(UserKey, "system_admin");
```

### Splitting Keys

Split keys into components:

```rust
let complex_key = UserKey::new("user_123_profile_settings")?;
let parts: Vec<&str> = complex_key.split('_').collect();
assert_eq!(parts, vec!["user", "123", "profile", "settings"]);
# Ok::<(), domain_key::KeyParseError>(())
```

## Performance Optimization

### Feature Selection

Choose the right features for your use case:

```toml
# Maximum performance (production)
domain-key = { version = "0.1", features = ["fast"] }

# Security-focused
domain-key = { version = "0.1", features = ["secure"] }

# Cryptographic applications
domain-key = { version = "0.1", features = ["crypto"] }
```

### Domain Configuration

Optimize domains for your usage patterns:

```rust
impl KeyDomain for HighPerformanceDomain {
    const DOMAIN_NAME: &'static str = "fast";
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
// Session management
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct SessionDomain;

impl KeyDomain for SessionDomain {
    const DOMAIN_NAME: &'static str = "session";
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct RouteDomain;

impl KeyDomain for RouteDomain {
    const DOMAIN_NAME: &'static str = "route";
    
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
// Primary keys
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct EntityIdDomain;

impl KeyDomain for EntityIdDomain {
    const DOMAIN_NAME: &'static str = "entity_id";
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ForeignKeyDomain;

impl KeyDomain for ForeignKeyDomain {
    const DOMAIN_NAME: &'static str = "foreign_key";
    
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CacheDomain;

impl KeyDomain for CacheDomain {
    const DOMAIN_NAME: &'static str = "cache";
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

Happy coding with domain-key! �