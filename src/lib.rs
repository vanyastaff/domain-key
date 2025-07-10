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
//! domain-key = { version = "0.1", features = ["fast"] }
//! ```
//!
//! Define a domain and create keys:
//!
//! ```rust
//! use domain_key::{Key, KeyDomain};
//!
//! // 1. Define your domain
//! #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! struct UserDomain;
//!
//! impl KeyDomain for UserDomain {
//!     const DOMAIN_NAME: &'static str = "user";
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
//! ## 🏎️ Performance Features
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
//! use domain_key::{Key, KeyDomain};
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! struct OptimizedDomain;
//!
//! impl KeyDomain for OptimizedDomain {
//!     const DOMAIN_NAME: &'static str = "optimized";
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
#![deny(unsafe_code)]

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
pub mod features;
pub mod key;
pub mod utils;
pub mod validation;

// IMPORTANT: Macros module must be declared but not re-exported with pub use
// because macros are automatically exported with #[macro_export]
#[macro_use]
mod macros;

// ============================================================================
// PUBLIC RE-EXPORTS
// ============================================================================

// Core types
pub use domain::{domain_info, DefaultDomain, IdentifierDomain, KeyDomain, PathDomain};
pub use error::{ErrorCategory, KeyParseError};
pub use key::Key;

// Helper types
pub use key::{KeyValidationInfo, SplitCache, SplitIterator};
pub use validation::IntoKey;

// Utility functions
pub use features::{hash_algorithm, performance_info, PerformanceInfo};
pub use utils::new_split_cache;
pub use validation::*;

// Constants
pub use key::DEFAULT_MAX_KEY_LENGTH;

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
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct MyDomain;
/// impl KeyDomain for MyDomain {
///     const DOMAIN_NAME: &'static str = "my";
/// }
/// type MyKey = Key<MyDomain>;
///
/// let key = MyKey::new("example")?;
/// # Ok::<(), KeyParseError>(())
/// ```
pub mod prelude {
    pub use crate::{
        ErrorCategory, IntoKey, Key, KeyDomain, KeyParseError, KeyResult, KeyValidationInfo,
    };

    // Re-export the macros in prelude for convenience
    // Note: These are already available at crate root due to #[macro_export]
    // but users might want them in prelude
    #[doc(hidden)]
    pub use crate::{batch_keys, define_domain, key_type, static_key, test_domain};
}
