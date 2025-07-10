# domain-key 🚀

**High-performance, type-safe, domain-driven key system for Rust applications**

[![Crates.io](https://img.shields.io/crates/v/domain-key.svg)](https://crates.io/crates/domain-key)
[![Documentation](https://docs.rs/domain-key/badge.svg)](https://docs.rs/domain-key)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/vanyastaff/domain-key/workflows/CI/badge.svg)](https://github.com/vanyastaff/domain-key/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)

> Never mix up keys from different domains again! 🎯

## ✨ What is domain-key?

domain-key brings **Domain-Driven Design** principles to key management in Rust. It provides compile-time guarantees that keys from different business domains cannot be accidentally mixed or compared, while delivering exceptional performance through advanced optimizations.

```rust
use domain_key::{Key, KeyDomain};

// Define your business domains
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct OrderDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
}

impl KeyDomain for OrderDomain {
    const DOMAIN_NAME: &'static str = "order";
}

// Create domain-specific key types
type UserKey = Key<UserDomain>;
type OrderKey = Key<OrderDomain>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use them safely
    let user_id = UserKey::new("user_123")?;
    let order_id = OrderKey::new("order_456")?;

    // This won't compile! 🛡️
    // let mixed = user_id == order_id; // Compile error!
    
    println!("User: {}", user_id.as_str());
    println!("Order: {}", order_id.as_str());
    Ok(())
}
```

## 🎯 Key Features

- **🔒 Type Safety**: Different key types cannot be mixed at compile time
- **🏎️ High Performance**: Up to 75% performance improvements through advanced optimizations
- **🎯 Domain Agnostic**: No built-in assumptions about specific domains
- **💾 Memory Efficient**: Smart string handling with stack allocation for short keys
- **🛡️ DoS Resistant**: Optional protection against HashDoS attacks
- **🔧 Extensible**: Easy to add new domains and validation rules
- **📦 Zero-Cost Abstractions**: No runtime overhead for type separation
- **🌐 Cross-Platform**: Works on all major platforms including WebAssembly

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
domain-key = "0.1"

# For maximum performance
domain-key = { version = "0.1", features = ["fast"] }

# For security-critical applications  
domain-key = { version = "0.1", features = ["secure"] }
```

Define a domain and create keys:

```rust
use domain_key::{Key, KeyDomain};

// 1. Define your domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
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

## 🏎️ Performance Features

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
| Hash Operations | 25ns | 15ns | **40% faster** |
| Length Access | O(n) | O(1) | **Constant time** |
| Collection Lookup | 35ns | 21ns | **40% faster** |

## 📖 Advanced Examples

### E-commerce Domain
```rust
use domain_key::{Key, KeyDomain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ProductDomain;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CartDomain;

impl KeyDomain for ProductDomain {
    const DOMAIN_NAME: &'static str = "product";
    const MAX_LENGTH: usize = 32;
}

impl KeyDomain for CartDomain {
    const DOMAIN_NAME: &'static str = "cart";
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
use domain_key::{Key, KeyDomain, KeyParseError};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TenantDomain;

impl KeyDomain for TenantDomain {
    const DOMAIN_NAME: &'static str = "tenant";
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
use domain_key::{Key, KeyDomain, KeyParseError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct EmailDomain;

impl KeyDomain for EmailDomain {
    const DOMAIN_NAME: &'static str = "email";
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

## 🔧 Feature Flags Reference

### Hash Algorithm Features (choose one for best results)

- `fast` - GxHash (40% faster, requires modern CPU with AES-NI)
- `secure` - AHash (DoS protection, balanced performance)
- `crypto` - Blake3 (cryptographically secure)
- Default - Standard hasher (good compatibility)

### Core Features

- `std` - Standard library support (enabled by default)
- `serde` - Serialization support (enabled by default)
- `no_std` - No standard library support

## 🛡️ Security Considerations

domain-key provides multiple levels of security depending on your needs:

- **DoS Protection**: Use `secure` feature for AHash with DoS resistance
- **Cryptographic Security**: Use `crypto` feature for Blake3 cryptographic hashing
- **Input Validation**: Comprehensive validation pipeline with custom rules
- **Type Safety**: Compile-time prevention of key type mixing
- **Memory Safety**: Rust's ownership system + additional optimizations

See [SECURITY.md](SECURITY.md) for detailed security information.

## 📚 Documentation

- **📖 [User Guide](docs/guide.md)** - Comprehensive usage guide
- **🎓 [API Documentation](https://docs.rs/domain-key)** - Complete API reference
- **🚀 [Examples](examples/)** - Real-world usage examples
- **📋 [Migration Guide](docs/migration.md)** - Migrating from string keys
- **🏎️ [Performance Guide](docs/performance.md)** - Optimization strategies
- **🔒 [Security Policy](SECURITY.md)** - Security considerations and reporting

## 🧪 Testing

Run the comprehensive test suite:

```bash
# All tests with all features
cargo test --all-features

# Property-based tests
cargo test --features std,serde --release -- prop_

# Benchmarks
cargo bench --features fast

# Security audit
cargo audit
```

## 📈 Benchmarks

```bash
# Run realistic benchmarks
cargo bench --features fast

# Memory usage analysis
cargo test --release memory_usage

# Cross-platform performance
cargo test --features fast --target wasm32-unknown-unknown
```

## 🔄 Migration from String Keys

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

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Development Setup

```bash
git clone https://github.com/vanyastaff/domain-key.git
cd domain-key

# Install development dependencies
rustup target add wasm32-unknown-unknown
cargo install cargo-audit cargo-hack

# Run tests
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt
```

## 🌟 Platform Support

| Platform | Status | Hash Algorithm | Notes |
|----------|--------|---------------|-------|
| Linux x86_64 | ✅ Full | GxHash/AHash | Best performance with AES-NI |
| Windows x86_64 | ✅ Full | GxHash/AHash | Full feature support |
| macOS Intel | ✅ Full | GxHash/AHash | All features supported |
| macOS Apple Silicon | ✅ Full | GxHash/AHash | Requires explicit AES+NEON flags |
| WebAssembly | ✅ Core | DefaultHasher | no_std support |
| ARM64 Linux | ✅ Full | GxHash/AHash | Server deployments |
| ARM Embedded | ✅ Core | FNV-1a | no_std + no_alloc |

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by Domain-Driven Design principles by Eric Evans
- Built on the excellent [`smartstring`](https://crates.io/crates/smartstring) crate for memory efficiency
- Performance-focused hash algorithms from the Rust ecosystem:
  - [`ahash`](https://crates.io/crates/ahash) for DoS-resistant hashing
  - [`gxhash`](https://crates.io/crates/gxhash) for maximum performance
  - [`blake3`](https://crates.io/crates/blake3) for cryptographic security

## 📊 Project Stats

- **Lines of Code**: ~3,000 (including comprehensive tests)
- **Test Coverage**: >95%
- **Documentation Coverage**: >98%
- **Benchmark Coverage**: 20+ realistic scenarios
- **no_std Support**: ✅
- **MSRV**: Rust 1.75+
- **Platforms**: 7+ supported targets

---

**domain-key** - Because your keys should know their place in your domain! 🏗️