//! # domain-key 🚀
//!
//! **High-performance, type-safe, domain-agnostic key system for Rust applications.**
//!
//! domain-key provides a flexible and efficient foundation for creating domain-specific keys
//! with compile-time type safety, runtime validation, and extensive performance optimizations.
//! This library focuses on zero-cost abstractions and maximum performance through feature-based
//! optimization profiles.
//!
//! ## ✨ Key Features
//!
//! - **🔒 Type Safety**: Different key types cannot be mixed at compile time
//! - **🏎️ High Performance**: Up to 75% performance improvements through advanced optimizations
//! - **🎯 Domain Agnostic**: No built-in assumptions about specific domains
//! - **💾 Memory Efficient**: Smart string handling with stack allocation for short keys
//! - **🛡️ `DoS` Resistant**: Optional protection against `HashDoS` attacks
//! - **🔧 Extensible**: Easy to add new domains and validation rules
//! - **📦 Zero-Cost Abstractions**: No runtime overhead for type separation
//!
//! ## 🏗️ Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     APPLICATION LAYER                          │
//! │  Business Logic  │  Domain Models  │  API Endpoints            │
//! └─────────────────┬───────────────────┬───────────────────────────┘
//!                   │                   │
//!                   ▼                   ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   TYPE SAFETY LAYER                            │
//! │  Key<UserDomain> │ Key<SessionDomain> │ Key<CacheDomain>        │
//! └─────────────────┬───────────────────────────────────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                 PERFORMANCE LAYER                              │
//! │  Stack Alloc │ Caching │ Specialized Ops │ Thread-Local        │
//! └─────────────────┬───────────────────────────────────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  STORAGE LAYER                                 │
//! │  SmartString + Cached Hash + Cached Length + Optimizations     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 🚀 Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! domain-key = { version = "0.2", features = ["fast"] }
//! ```
//!
//! Define a domain and create keys:
//!
//! ```rust
//! use domain_key::{Key, Domain, KeyDomain};
//!
//! // 1. Define your domain
//! #[derive(Debug)]
//! struct UserDomain;
//!
//! impl Domain for UserDomain {
//!     const DOMAIN_NAME: &'static str = "user";
//! }
//!
//! impl KeyDomain for UserDomain {
//!     const MAX_LENGTH: usize = 32;
//!     const TYPICALLY_SHORT: bool = true; // Optimization hint
//! }
//!
//! // 2. Create a type alias
//! type UserKey = Key<UserDomain>;
//!
//! // 3. Use it!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let user_key = UserKey::new("john_doe")?;
//! let composed_key = UserKey::from_parts(&["user", "123", "profile"], "_")?;
//!
//! println!("Domain: {}", user_key.domain());
//! println!("Length: {}", user_key.len()); // O(1) with optimizations
//! println!("Key: {}", user_key.as_str());
//! # Ok(())
//! # }
//! ```
//!
//! ## Identifier Types
//!
//! domain-key provides three typed identifier wrappers:
//!
//! | Type | Storage | Use case |
//! |------|---------|----------|
//! | [`Key<D>`] | `SmartString` | Human-readable keys with validation |
//! | [`Id<D>`] | `NonZeroU64` | Numeric database IDs (8 bytes, `Copy`) |
//! | [`Uuid<D>`] | `uuid::Uuid` | UUID identifiers (16 bytes, `Copy`, feature `uuid`) |
//!
//! ```rust
//! use domain_key::prelude::*;
//!
//! // Numeric IDs
//! define_id_domain!(UserIdDomain, "user");
//! id_type!(UserId, UserIdDomain);
//!
//! let id = UserId::new(42).unwrap();
//! assert_eq!(id.get(), 42);
//!
//! // Or use the shorthand:
//! define_id!(OrderIdDomain => OrderId);
//! let order = OrderId::new(1).unwrap();
//! ```
//!
//! All three types are domain-typed: `UserId` and `OrderId` are incompatible
//! at compile time even though both wrap a `NonZeroU64`.
//!
//! ## Performance Features
//!
//! ### Feature-Based Optimization Profiles
//!
//! ```toml
//! # Maximum performance (modern CPUs with AES-NI)
//! features = ["fast"]
//!
//! # DoS protection + good performance
//! features = ["secure"]
//!
//! # Cryptographic security
//! features = ["crypto"]
//!
//! # All optimizations enabled
//! features = ["fast", "std", "serde"]
//! ```
//!
//! ### Build for Maximum Performance
//!
//! ```bash
//! # Enable CPU-specific optimizations
//! RUSTFLAGS="-C target-cpu=native" cargo build --release --features="fast"
//! ```
//!
//! ### Performance Improvements
//!
//! | Operation | Standard | Optimized | Improvement |
//! |-----------|----------|-----------|-------------|
//! | Key Creation (short) | 100% | 128% | **28% faster** |
//! | String Operations | 100% | 175% | **75% faster** |
//! | Hash Operations | 100% | 140% | **40% faster** |
//! | Length Access | O(n) | O(1) | **Constant time** |
//!
//! ## 📖 Advanced Examples
//!
//! ### Performance-Optimized Usage
//!
//! ```rust
//! use domain_key::{Key, Domain, KeyDomain};
//!
//! #[derive(Debug)]
//! struct OptimizedDomain;
//!
//! impl Domain for OptimizedDomain {
//!     const DOMAIN_NAME: &'static str = "optimized";
//! }
//!
//! impl KeyDomain for OptimizedDomain {
//!     const EXPECTED_LENGTH: usize = 16; // Optimization hint
//!     const TYPICALLY_SHORT: bool = true; // Enable stack allocation
//! }
//!
//! type OptimizedKey = Key<OptimizedDomain>;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Basic optimized key creation
//! let user_key = OptimizedKey::new("user_12345")?;
//! let session_key = OptimizedKey::new("session_abc123")?;
//!
//! // Batch operations with from_parts
//! let user_ids = vec![1, 2, 3, 4, 5];
//! let user_keys: Result<Vec<_>, _> = user_ids.iter()
//!     .map(|&id| OptimizedKey::from_parts(&["user", &id.to_string()], "_"))
//!     .collect();
//! let user_keys = user_keys?;
//!
//! // Optimized operations for repeated use
//! let key = OptimizedKey::new("user_profile_settings_theme")?;
//! let parts: Vec<&str> = key.split('_').collect(); // Uses optimizations when available
//! # Ok(())
//! # }
//! ```
//!
//! ## 🔧 Feature Flags Reference
//!
//! ### Hash Algorithm Features (choose one for best results)
//!
//! - `fast` - `GxHash` (40% faster, requires modern CPU with AES-NI)
//! - `secure` - `AHash` (`DoS` protection, balanced performance)
//! - `crypto` - Blake3 (cryptographically secure)
//! - Default - Standard hasher (good compatibility)
//!
//! ### Identifier Features
//!
//! - `uuid` — enables `Uuid<D>` typed UUID identifiers
//! - `uuid-v4` — enables `Uuid::v4()` random generation
//! - `uuid-v7` — enables `Uuid::now_v7()` time-ordered generation
//!
//! ### Core Features
//!
//! - `std` - Standard library support (enabled by default)
//! - `serde` - Serialization support (enabled by default)
//! - `no_std` - No standard library (disables std-dependent features)
//!
//! ## 🛡️ Security Considerations
//!
//! domain-key provides multiple levels of security depending on your needs:
//!
//! - **`DoS` Protection**: Use `secure` feature for `AHash` with `DoS` resistance
//! - **Cryptographic Security**: Use `crypto` feature for Blake3 cryptographic hashing
//! - **Input Validation**: Comprehensive validation pipeline with custom rules
//! - **Type Safety**: Compile-time prevention of key type mixing
//! - **Memory Safety**: Rust's ownership system + additional optimizations

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

// ============================================================================
// EXTERNAL DEPENDENCIES
// ============================================================================

#[cfg(not(feature = "std"))]
extern crate alloc;

// ============================================================================
// COMPILE-TIME FEATURE VALIDATION
// ============================================================================

// Improved feature validation that allows testing with --all-features
// but warns about suboptimal configurations

#[cfg(all(
    feature = "fast",
    feature = "secure",
    not(test),  // Allow all features during testing
    not(doc),
    not(debug_assertions),
))]
compile_error!("Both 'fast' and 'secure' features are enabled. For optimal performance, choose only 'fast'. For security, choose only 'secure'.");

#[cfg(all(
    feature = "fast",
    feature = "crypto",
    not(test),  // Allow all features during testing
    not(doc),
    not(debug_assertions),
))]
compile_error!("Both 'fast' and 'crypto' features are enabled. For optimal performance, choose only 'fast'. For cryptographic security, choose only 'crypto'.");

#[cfg(all(
    feature = "secure",
    feature = "crypto",
    not(test),  // Allow all features during testing
    not(doc),
    not(debug_assertions),
))]
compile_error!("Both 'secure' and 'crypto' features are enabled. Choose one hash algorithm based on your security requirements.");

// ============================================================================
// INTERNAL MODULES
// ============================================================================

pub mod domain;
pub mod error;
pub mod id;
pub mod key;
pub mod utils;
pub mod validation;

#[cfg(feature = "uuid")]
pub mod uuid;

// IMPORTANT: Macros module must be declared but not re-exported with pub use
// because macros are automatically exported with #[macro_export]
#[macro_use]
mod macros;

// ============================================================================
// PUBLIC RE-EXPORTS
// ============================================================================

// Core types
#[cfg(feature = "uuid")]
pub use domain::UuidDomain;
pub use domain::{
    domain_info, DefaultDomain, Domain, DomainInfo, IdDomain, IdentifierDomain, KeyDomain,
    PathDomain,
};
#[cfg(feature = "uuid")]
pub use error::UuidParseError;
pub use error::{ErrorCategory, IdParseError, KeyParseError};
pub use id::Id;
pub use key::Key;
#[cfg(feature = "uuid")]
pub use uuid::Uuid;

// Helper types
pub use key::{KeyValidationInfo, SplitCache, SplitIterator};
pub use validation::IntoKey;

// Utility functions
pub use utils::{hash_algorithm, new_split_cache};
pub use validation::*;

// Constants
pub use key::DEFAULT_MAX_KEY_LENGTH;

// Hidden re-exports for macro hygiene (so macros work without caller imports)
#[doc(hidden)]
pub mod __private {
    #[cfg(not(feature = "std"))]
    pub use alloc::string::ToString;
    #[cfg(feature = "std")]
    pub use std::string::ToString;

    #[cfg(not(feature = "std"))]
    pub use alloc::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;
}

// Note: Macros are exported automatically by #[macro_export] in macros.rs
// They don't need to be re-exported here

// ============================================================================
// CONVENIENCE TYPE ALIASES
// ============================================================================

/// Result type for key operations
pub type KeyResult<T> = Result<T, KeyParseError>;

// ============================================================================
// PRELUDE MODULE
// ============================================================================

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits, allowing
/// users to easily import everything they need with a single `use` statement.
///
/// # Examples
///
/// ```rust
/// use domain_key::prelude::*;
///
/// #[derive(Debug)]
/// struct MyDomain;
///
/// impl Domain for MyDomain {
///     const DOMAIN_NAME: &'static str = "my";
/// }
/// impl KeyDomain for MyDomain {}
///
/// type MyKey = Key<MyDomain>;
///
/// let key = MyKey::new("example")?;
/// # Ok::<(), KeyParseError>(())
/// ```
pub mod prelude {
    pub use crate::{
        Domain, DomainInfo, ErrorCategory, Id, IdDomain, IdParseError, IntoKey, Key, KeyDomain,
        KeyParseError, KeyResult, KeyValidationInfo,
    };

    #[cfg(feature = "uuid")]
    pub use crate::{Uuid, UuidDomain, UuidParseError};

    // Re-export the macros in prelude for convenience
    // Note: These are already available at crate root due to #[macro_export]
    // but users might want them in prelude
    #[doc(hidden)]
    pub use crate::{
        batch_keys, define_domain, define_id, define_id_domain, id_type, key_type, static_key,
        test_domain,
    };

    #[cfg(feature = "uuid")]
    #[doc(hidden)]
    pub use crate::{define_uuid, define_uuid_domain, uuid_type};
}
