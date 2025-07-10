//! Core Key implementation for domain-key
//!
//! This module contains the main `Key<T>` structure and its implementation,
//! providing high-performance, type-safe key handling with extensive optimizations.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::str::FromStr;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use smartstring::alias::String as SmartString;

use crate::domain::KeyDomain;
use crate::error::KeyParseError;
use crate::utils;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default maximum allowed length for any key
///
/// This is a reasonable default that balances usability with performance.
/// Keys up to this length can benefit from stack allocation optimizations.
/// Domains can override this with their own limits.
pub const DEFAULT_MAX_KEY_LENGTH: usize = 64;

/// Buffer size for stack allocation optimizations
///
/// This size is chosen to accommodate the vast majority of real-world keys
/// while remaining reasonable for stack usage.
pub const STACK_BUFFER_SIZE: usize = 128;

// ============================================================================
// SPLIT ITERATOR TYPES
// ============================================================================

/// Split cache type for consistent API
pub type SplitCache<'a> = core::str::Split<'a, char>;

/// Split iterator with consistent API
#[derive(Debug)]
pub enum SplitIterator<'a> {
    /// Cached split iterator
    Cached(SplitCache<'a>),
}

impl<'a> Iterator for SplitIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SplitIterator::Cached(iter) => iter.next(),
        }
    }
}

// ============================================================================
// FAST CHARACTER VALIDATION
// ============================================================================

/// Fast character validation function
#[inline(always)]
#[allow(clippy::inline_always)]
const fn is_ascii_allowed_fast(c: char) -> bool {
    // Simple lookup for common ASCII characters
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.')
}

// ============================================================================
// CORE KEY IMPLEMENTATION
// ============================================================================

/// High-performance generic key type with advanced optimizations
///
/// This is the core key type that provides type safety through the domain
/// marker `T`. Keys are immutable after creation and use `SmartString` for
/// optimal memory usage (stack allocation for short keys, heap for longer ones).
///
/// # Performance Characteristics
///
/// - **Memory Layout**: 32 bytes total (fits in single cache line)
/// - **Hash Access**: O(1) via pre-computed hash
/// - **Length Access**: O(1) via cached length field
/// - **String Access**: Direct reference to internal storage
/// - **Clone**: Efficient via `SmartString`'s copy-on-write semantics
///
/// # Type Parameters
///
/// * `T` - A domain marker type that implements `KeyDomain`
///
/// # Memory Layout
///
/// ```text
/// Key<T> struct (32 bytes, cache-line friendly):
/// ┌─────────────────────┬──────────┬─────────┬─────────────┐
/// │ SmartString (24B)   │ hash (8B)│ len (4B)│ marker (0B) │
/// └─────────────────────┴──────────┴─────────┴─────────────┘
/// ```
///
/// Keys use `SmartString` which stores strings up to 23 bytes inline on the stack,
/// only allocating on the heap for longer strings. Additionally, the pre-computed
/// hash is stored for O(1) hash operations.
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, KeyDomain};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct UserDomain;
///
/// impl KeyDomain for UserDomain {
///     const DOMAIN_NAME: &'static str = "user";
///     const MAX_LENGTH: usize = 32;
/// }
///
/// type UserKey = Key<UserDomain>;
///
/// let key = UserKey::new("john_doe")?;
/// assert_eq!(key.as_str(), "john_doe");
/// assert_eq!(key.domain(), "user");
/// assert_eq!(key.len(), 8);
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key<T: KeyDomain> {
    /// Internal string storage using `SmartString` for optimal memory usage
    inner: SmartString,

    /// Pre-computed hash value for O(1) hash operations
    ///
    /// This hash is computed once during key creation and cached for the
    /// lifetime of the key, providing significant performance benefits
    /// for hash-based collections.
    hash: u64,

    /// Cached length for O(1) length access
    ///
    /// This optimization eliminates the need to traverse the string to
    /// determine its length, which can be a significant performance
    /// improvement in hot paths.
    length: u32,

    /// Zero-sized type marker for compile-time type safety
    ///
    /// This field provides compile-time type safety without any runtime
    /// overhead. Different domain types cannot be mixed or compared.
    _marker: PhantomData<T>,
}

// Manual Clone implementation to ensure optimal performance
impl<T: KeyDomain> Clone for Key<T> {
    /// Efficient clone implementation
    ///
    /// Cloning a key is efficient due to `SmartString`'s optimizations:
    /// - For inline strings (≤23 chars): Simple memory copy
    /// - For heap strings: Reference counting or copy-on-write
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            hash: self.hash,
            length: self.length,
            _marker: PhantomData,
        }
    }
}

// Manual Hash implementation using cached hash for maximum performance
impl<T: KeyDomain> Hash for Key<T> {
    /// O(1) hash implementation using pre-computed hash
    ///
    /// This is significantly faster than re-hashing the string content
    /// every time the key is used in hash-based collections.
    #[inline(always)]
    #[allow(clippy::inline_always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

// Conditional Serde support for serialization/deserialization
#[cfg(feature = "serde")]
impl<T: KeyDomain> Serialize for Key<T> {
    /// Serialize the key as its string representation
    ///
    /// Keys are serialized as their string content, not including
    /// the cached hash or length for efficiency and compatibility.
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: KeyDomain> Deserialize<'de> for Key<T> {
    /// Deserialize and validate a key from its string representation
    ///
    /// This implementation chooses the optimal deserialization strategy
    /// based on the format (human-readable vs binary) for best performance.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            // For human-readable formats (JSON, YAML), use zero-copy when possible
            let s = <&str>::deserialize(deserializer)?;
            Key::new(s).map_err(|e| serde::de::Error::custom(e.to_string()))
        } else {
            // For binary formats, deserialize as owned string
            let s = String::deserialize(deserializer)?;
            Key::from_string(s).map_err(|e| serde::de::Error::custom(e.to_string()))
        }
    }
}

// ============================================================================
// KEY IMPLEMENTATION - CORE METHODS
// ============================================================================

impl<T: KeyDomain> Key<T> {
    /// Creates a new key with comprehensive validation and optimization
    ///
    /// This method performs both common validation (length, characters) and
    /// domain-specific validation according to the key's domain type. It
    /// automatically chooses the optimal creation path based on the input
    /// characteristics and domain configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - String-like input that will be normalized and validated
    ///
    /// # Returns
    ///
    /// * `Ok(Key<T>)` if the key is valid
    /// * `Err(KeyParseError)` with the specific validation failure
    ///
    /// # Errors
    ///
    /// Returns `KeyParseError` if the key fails common validation (empty, too
    /// long, invalid characters) or domain-specific validation rules
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("valid_key")?;
    /// assert_eq!(key.as_str(), "valid_key");
    ///
    /// // Invalid keys return descriptive errors
    /// let error = TestKey::new("").unwrap_err();
    /// assert!(matches!(error, domain_key::KeyParseError::Empty));
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
    pub fn new(key: impl AsRef<str>) -> Result<Self, KeyParseError> {
        let key_str = key.as_ref();
        Self::new_optimized(key_str)
    }

    /// Optimized implementation for key creation
    ///
    /// This method uses performance optimizations when available:
    /// - Stack allocation for short keys
    /// - Fast validation paths
    /// - Cached operations
    ///
    /// # Errors
    ///
    /// Returns `KeyParseError` if the constructed key fails validation
    fn new_optimized(key: &str) -> Result<Self, KeyParseError> {
        // Step 1: Common validation (length, characters, structure)
        Self::validate_common::<T>(key)?;

        // Step 2: Normalization (trimming, lowercasing, domain-specific)
        let normalized = Self::normalize::<T>(key);

        // Step 3: Domain-specific validation
        T::validate_domain_rules(&normalized).map_err(Self::fix_domain_error)?;

        // Step 4: Hash computation and storage
        let hash = Self::compute_hash(&normalized);
        let length = u32::try_from(normalized.len()).map_err(|_| KeyParseError::TooLong {
            max_length: u32::MAX as usize,
            actual_length: normalized.len(),
        })?;

        Ok(Self {
            inner: SmartString::from(normalized.as_ref()),
            hash,
            length,
            _marker: PhantomData,
        })
    }

    /// Creates a new key from an owned String with optimized handling
    ///
    /// This method is more efficient when you already have a `String` as it
    /// can reuse the allocation when possible.
    ///
    /// # Arguments
    ///
    /// * `key` - Owned string that will be normalized and validated
    ///
    /// # Errors
    ///
    /// Returns `KeyParseError` if the key fails validation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key_string = "test_key".to_string();
    /// let key = TestKey::from_string(key_string)?;
    /// assert_eq!(key.as_str(), "test_key");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    pub fn from_string(key: String) -> Result<Self, KeyParseError> {
        // Validate the original string
        Self::validate_common::<T>(&key)?;

        // Normalize efficiently, reusing allocation when possible
        let normalized = Self::normalize_owned::<T>(key);

        // Domain validation
        T::validate_domain_rules(&normalized).map_err(Self::fix_domain_error)?;

        let hash = Self::compute_hash(&normalized);
        let length = u32::try_from(normalized.len()).map_err(|_| KeyParseError::TooLong {
            max_length: u32::MAX as usize,
            actual_length: normalized.len(),
        })?;

        Ok(Self {
            inner: SmartString::from(normalized),
            hash,
            length,
            _marker: PhantomData,
        })
    }

    /// Create a key from multiple parts separated by a delimiter
    ///
    /// This method efficiently constructs a key from multiple string parts,
    /// using pre-calculated sizing to minimize allocations.
    ///
    /// # Arguments
    ///
    /// * `parts` - Array of string parts to join
    /// * `delimiter` - String to use as separator between parts
    ///
    /// # Returns
    ///
    /// * `Ok(Key<T>)` if the constructed key is valid
    /// * `Err(KeyParseError)` if validation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::from_parts(&["user", "123", "profile"], "_")?;
    /// assert_eq!(key.as_str(), "user_123_profile");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    /// # Errors
    ///
    /// Returns `KeyParseError` if the constructed key fails validation
    pub fn from_parts(parts: &[&str], delimiter: &str) -> Result<Self, KeyParseError> {
        if parts.is_empty() {
            return Err(KeyParseError::Empty);
        }

        if parts.iter().any(|part| part.is_empty()) {
            return Err(KeyParseError::InvalidStructure {
                reason: "Parts cannot contain empty strings",
            });
        }

        let joined = parts.join(delimiter);

        if joined.is_empty() {
            return Err(KeyParseError::Empty);
        }

        Self::from_string(joined)
    }

    /// Try to create a key from multiple parts, returning None on failure
    ///
    /// This is a convenience method for when you want to handle validation
    /// failures by ignoring invalid keys rather than handling errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let valid = TestKey::try_from_parts(&["user", "123"], "_").unwrap();
    /// let invalid = TestKey::try_from_parts(&["", ""], "_"); // Returns None
    /// assert!(invalid.is_none());
    /// ```
    #[must_use]
    pub fn try_from_parts(parts: &[&str], delimiter: &str) -> Option<Self> {
        Self::from_parts(parts, delimiter).ok()
    }

    /// Creates a key from a static string without runtime validation
    ///
    /// # Safety
    ///
    /// The caller must ensure that the static string follows all validation
    /// rules for the domain. Invalid keys created this way may cause
    /// undefined behavior in other parts of the system that assume all
    /// keys are valid.
    ///
    /// Use the `static_key!` macro instead for compile-time checked static keys.
    ///
    /// # Arguments
    ///
    /// * `key` - A static string literal that represents a valid key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// // SAFETY: "static_key" is a valid key for TestDomain
    /// let key = TestKey::from_static_unchecked("static_key");
    /// assert_eq!(key.as_str(), "static_key");
    /// ```
    #[must_use]
    pub fn from_static_unchecked(key: &'static str) -> Self {
        let hash = Self::compute_hash(key);
        #[allow(clippy::cast_possible_truncation)]
        let length = key.len() as u32;

        Self {
            inner: SmartString::from(key),
            hash,
            length,
            _marker: PhantomData,
        }
    }

    /// Creates a key from a static string with validation
    ///
    /// This is a safer alternative to `from_static_unchecked` that validates
    /// the key at runtime. The validation cost is paid once, and subsequent
    /// uses of the key are as fast as the unchecked version.
    ///
    /// # Arguments
    ///
    /// * `key` - A static string literal to validate and convert
    ///
    /// # Errors
    ///
    /// Returns `KeyParseError` if the static key fails validation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::try_from_static("static_key")?;
    /// assert_eq!(key.as_str(), "static_key");
    ///
    /// let invalid = TestKey::try_from_static("");
    /// assert!(invalid.is_err());
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    /// # Errors
    ///
    /// Returns `KeyParseError` if the constructed key fails validation
    pub fn try_from_static(key: &'static str) -> Result<Self, KeyParseError> {
        // First validate that the key is correct
        Self::new(key)?;

        // We just validated that the key is correct above
        Ok(Self::from_static_unchecked(key))
    }

    /// Try to create a key, returning None on validation failure
    ///
    /// This is a convenience method for when you want to handle validation
    /// failures by ignoring invalid keys rather than handling errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let valid = TestKey::try_new("valid_key").unwrap();
    /// let invalid = TestKey::try_new(""); // Returns None
    /// assert!(invalid.is_none());
    /// ```
    #[inline]
    pub fn try_new(key: impl AsRef<str>) -> Option<Self> {
        Self::new(key).ok()
    }
}

// ============================================================================
// KEY IMPLEMENTATION - ACCESSOR METHODS
// ============================================================================

impl<T: KeyDomain> Key<T> {
    /// Returns the key as a string slice
    ///
    /// This is the primary way to access the string content of a key.
    /// The returned reference is valid for the lifetime of the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// assert_eq!(key.as_str(), "example");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline(always)]
    #[allow(clippy::inline_always)]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Returns the domain name for this key type
    ///
    /// This is a compile-time constant that identifies which domain
    /// this key belongs to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct UserDomain;
    /// impl KeyDomain for UserDomain {
    ///     const DOMAIN_NAME: &'static str = "user";
    /// }
    /// type UserKey = Key<UserDomain>;
    ///
    /// let key = UserKey::new("john")?;
    /// assert_eq!(key.domain(), "user");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline(always)]
    #[allow(clippy::inline_always, clippy::unused_self)]
    #[must_use]
    pub const fn domain(&self) -> &'static str {
        T::DOMAIN_NAME
    }

    /// Returns the length of the key string
    ///
    /// This is an O(1) operation using a cached length value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// assert_eq!(key.len(), 7);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline(always)]
    #[allow(clippy::inline_always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.length as usize
    }

    /// Returns true if the key is empty (this should never happen for valid keys)
    ///
    /// Since empty keys are rejected during validation, this method should
    /// always return `false` for properly constructed keys. It's provided
    /// for completeness and debugging purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// assert!(!key.is_empty());
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline(always)]
    #[allow(clippy::inline_always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns the cached hash value
    ///
    /// This hash is computed once during key creation and cached for the
    /// lifetime of the key. It's used internally for hash-based collections
    /// and can be useful for custom hash-based data structures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key1 = TestKey::new("example")?;
    /// let key2 = TestKey::new("example")?;
    /// let key3 = TestKey::new("different")?;
    ///
    /// // Same keys have same hash
    /// assert_eq!(key1.hash(), key2.hash());
    /// // Different keys have different hashes (with high probability)
    /// assert_ne!(key1.hash(), key3.hash());
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline(always)]
    #[allow(clippy::inline_always)]
    #[must_use]
    pub const fn hash(&self) -> u64 {
        self.hash
    }

    /// Checks if this key starts with the given prefix
    ///
    /// This is a simple string prefix check that can be useful for
    /// categorizing or filtering keys.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix string to check for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("user_profile")?;
    /// assert!(key.starts_with("user_"));
    /// assert!(!key.starts_with("admin_"));
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.inner.starts_with(prefix)
    }

    /// Checks if this key ends with the given suffix
    ///
    /// This is a simple string suffix check that can be useful for
    /// categorizing or filtering keys.
    ///
    /// # Arguments
    ///
    /// * `suffix` - The suffix string to check for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("user_profile")?;
    /// assert!(key.ends_with("_profile"));
    /// assert!(!key.ends_with("_settings"));
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.inner.ends_with(suffix)
    }

    /// Checks if this key contains the given substring
    ///
    /// This performs a substring search within the key.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The substring to search for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("user_profile_settings")?;
    /// assert!(key.contains("profile"));
    /// assert!(!key.contains("admin"));
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn contains(&self, pattern: &str) -> bool {
        self.inner.contains(pattern)
    }

    /// Returns an iterator over the characters of the key
    ///
    /// This provides access to individual characters in the key string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("abc")?;
    /// let chars: Vec<char> = key.chars().collect();
    /// assert_eq!(chars, vec!['a', 'b', 'c']);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    pub fn chars(&self) -> core::str::Chars<'_> {
        self.inner.chars()
    }

    /// Splits the key by a delimiter and returns an iterator
    ///
    /// This method provides consistent split functionality.
    ///
    /// # Arguments
    ///
    /// * `delimiter` - Character to split on
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("user_profile_settings")?;
    /// let parts: Vec<&str> = key.split('_').collect();
    /// assert_eq!(parts, vec!["user", "profile", "settings"]);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[must_use]
    pub fn split(&self, delimiter: char) -> SplitIterator<'_> {
        SplitIterator::Cached(utils::new_split_cache(&self.inner, delimiter))
    }

    /// Split operation for consistent API
    ///
    /// This method provides the same functionality as `split()` but with explicit naming
    /// for cases where caching behavior needs to be clear.
    #[must_use]
    pub fn split_cached(&self, delimiter: char) -> SplitCache<'_> {
        utils::new_split_cache(&self.inner, delimiter)
    }

    /// Splits the key by a string delimiter and returns an iterator
    ///
    /// This method splits the key using a string pattern rather than a single character.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("user-and-profile-and-settings")?;
    /// let parts: Vec<&str> = key.split_str("-and-").collect();
    /// assert_eq!(parts, vec!["user", "profile", "settings"]);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[must_use]
    pub fn split_str<'a>(&'a self, delimiter: &'a str) -> core::str::Split<'a, &'a str> {
        self.inner.split(delimiter)
    }

    /// Returns the key with a prefix if it doesn't already have it
    ///
    /// This method efficiently adds a prefix to a key if it doesn't already
    /// start with that prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to ensure is present
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("profile")?;
    /// let prefixed = key.ensure_prefix("user_")?;
    /// assert_eq!(prefixed.as_str(), "user_profile");
    ///
    /// // If prefix already exists, returns the same key
    /// let already_prefixed = prefixed.ensure_prefix("user_")?;
    /// assert_eq!(already_prefixed.as_str(), "user_profile");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    /// # Errors
    ///
    /// Returns `KeyParseError` if the prefixed key would be invalid or too long
    pub fn ensure_prefix(&self, prefix: &str) -> Result<Self, KeyParseError> {
        if self.starts_with(prefix) {
            return Ok(self.clone());
        }

        let new_len = prefix.len() + self.len();
        if new_len > T::MAX_LENGTH {
            return Err(KeyParseError::TooLong {
                max_length: T::MAX_LENGTH,
                actual_length: new_len,
            });
        }

        let result = utils::add_prefix_optimized(&self.inner, prefix, T::MAX_LENGTH);

        // Quick validation of prefix only
        for (i, c) in prefix.chars().enumerate() {
            if !T::allowed_characters(c) {
                return Err(KeyParseError::InvalidCharacter {
                    character: c,
                    position: i,
                    expected: Some("allowed by domain"),
                });
            }
        }

        T::validate_domain_rules(&result).map_err(Self::fix_domain_error)?;

        let hash = Self::compute_hash(&result);
        let length = u32::try_from(new_len).map_err(|_| KeyParseError::TooLong {
            max_length: u32::MAX as usize,
            actual_length: new_len,
        })?;

        Ok(Self {
            inner: result,
            hash,
            length,
            _marker: PhantomData,
        })
    }

    /// Returns the key with a suffix if it doesn't already have it
    ///
    /// This method efficiently adds a suffix to a key if it doesn't already
    /// end with that suffix.
    ///
    /// # Arguments
    ///
    /// * `suffix` - The suffix to ensure is present
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("user")?;
    /// let suffixed = key.ensure_suffix("_profile")?;
    /// assert_eq!(suffixed.as_str(), "user_profile");
    ///
    /// // If suffix already exists, returns the same key
    /// let already_suffixed = suffixed.ensure_suffix("_profile")?;
    /// assert_eq!(already_suffixed.as_str(), "user_profile");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    /// # Errors
    ///
    /// Returns `KeyParseError` if the prefixed key would be invalid or too long
    pub fn ensure_suffix(&self, suffix: &str) -> Result<Self, KeyParseError> {
        if self.ends_with(suffix) {
            return Ok(self.clone());
        }

        let new_len = self.len() + suffix.len();
        if new_len > T::MAX_LENGTH {
            return Err(KeyParseError::TooLong {
                max_length: T::MAX_LENGTH,
                actual_length: new_len,
            });
        }

        let result = utils::add_suffix_optimized(&self.inner, suffix, T::MAX_LENGTH);

        // Quick validation of suffix only
        for (i, c) in suffix.chars().enumerate() {
            if !T::allowed_characters(c) {
                return Err(KeyParseError::InvalidCharacter {
                    character: c,
                    position: self.len() + i,
                    expected: Some("allowed by domain"),
                });
            }
        }

        T::validate_domain_rules(&result).map_err(Self::fix_domain_error)?;

        let hash = Self::compute_hash(&result);
        let length = new_len.try_into().map_err(|_| KeyParseError::TooLong {
            max_length: u32::MAX as usize,
            actual_length: new_len,
        })?;

        Ok(Self {
            inner: result,
            hash,
            length,
            _marker: PhantomData,
        })
    }

    /// Get validation rules that this key satisfies
    ///
    /// Returns detailed information about the validation characteristics
    /// of this key and its domain, useful for debugging and introspection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, KeyDomain};
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// struct TestDomain;
    /// impl KeyDomain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    ///     const MAX_LENGTH: usize = 32;
    ///     const HAS_CUSTOM_VALIDATION: bool = true;
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// let info = key.validation_info();
    ///
    /// assert_eq!(info.domain, "test");
    /// assert_eq!(info.max_length, 32);
    /// assert_eq!(info.length, 7);
    /// assert!(info.has_custom_validation);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[must_use]
    pub fn validation_info(&self) -> KeyValidationInfo {
        KeyValidationInfo {
            domain: T::DOMAIN_NAME,
            max_length: T::MAX_LENGTH,
            length: self.len(),
            has_custom_validation: T::HAS_CUSTOM_VALIDATION,
            has_custom_normalization: T::HAS_CUSTOM_NORMALIZATION,
        }
    }
}

// ============================================================================
// KEY IMPLEMENTATION - HELPER METHODS
// ============================================================================

impl<T: KeyDomain> Key<T> {
    /// Fix domain name in domain validation errors
    ///
    /// This helper ensures that domain validation errors have the correct
    /// domain name, even when they're created generically.
    #[inline]
    fn fix_domain_error(e: KeyParseError) -> KeyParseError {
        match e {
            KeyParseError::DomainValidation { message, .. } => KeyParseError::DomainValidation {
                domain: T::DOMAIN_NAME,
                message,
            },
            other => other,
        }
    }

    /// Common validation pipeline
    ///
    /// Performs validation that's common to all domains: length checking,
    /// character validation, and structural validation.
    ///
    /// # Errors
    ///
    /// Returns `KeyParseError` if the prefixed key would be invalid or too long
    pub(crate) fn validate_common<D: KeyDomain>(key: &str) -> Result<(), KeyParseError> {
        let trimmed = key.trim();

        if trimmed.is_empty() {
            return Err(KeyParseError::Empty);
        }

        if trimmed.len() > D::MAX_LENGTH {
            return Err(KeyParseError::TooLong {
                max_length: D::MAX_LENGTH,
                actual_length: trimmed.len(),
            });
        }

        if trimmed.len() < D::min_length() {
            return Err(KeyParseError::TooLong {
                max_length: D::min_length(),
                actual_length: trimmed.len(),
            });
        }

        // Use fast validation
        Self::validate_fast::<D>(trimmed)
    }

    /// Fast validation path using optimized algorithms
    /// # Errors
    ///
    /// Returns `KeyParseError` if the prefixed key would be invalid or too long
    fn validate_fast<D: KeyDomain>(key: &str) -> Result<(), KeyParseError> {
        let mut chars = key.char_indices();
        let mut prev_char = None;

        // Validate first character
        if let Some((pos, first)) = chars.next() {
            let char_allowed = is_ascii_allowed_fast(first) || D::allowed_start_character(first);

            if !char_allowed {
                return Err(KeyParseError::InvalidCharacter {
                    character: first,
                    position: pos,
                    expected: Some("allowed by domain"),
                });
            }

            prev_char = Some(first);
        }

        // Validate remaining characters
        for (pos, c) in chars {
            let char_allowed = is_ascii_allowed_fast(c) || D::allowed_characters(c);

            if !char_allowed {
                return Err(KeyParseError::InvalidCharacter {
                    character: c,
                    position: pos,
                    expected: Some("allowed by domain"),
                });
            }

            if let Some(prev) = prev_char {
                if !D::allowed_consecutive_characters(prev, c) {
                    return Err(KeyParseError::InvalidStructure {
                        reason: "consecutive characters not allowed",
                    });
                }
            }
            prev_char = Some(c);
        }

        // Check last character
        if let Some(last) = prev_char {
            if !D::allowed_end_character(last) {
                return Err(KeyParseError::InvalidStructure {
                    reason: "invalid end character",
                });
            }
        }

        Ok(())
    }

    /// Normalize a borrowed string
    pub(crate) fn normalize<D: KeyDomain>(key: &str) -> Cow<'_, str> {
        let trimmed = key.trim();

        let needs_lowercase =
            D::CASE_INSENSITIVE && trimmed.chars().any(|c| c.is_ascii_uppercase());

        let lowercased = if needs_lowercase {
            Cow::Owned(trimmed.to_ascii_lowercase())
        } else if trimmed.len() != key.len() {
            // Only trimming was needed
            Cow::Owned(trimmed.to_string())
        } else {
            // No changes needed
            Cow::Borrowed(trimmed)
        };

        // Apply domain-specific normalization
        D::normalize_domain(lowercased)
    }

    /// Normalize an owned string efficiently
    fn normalize_owned<D: KeyDomain>(mut key: String) -> String {
        // In-place operations when possible
        let trimmed = key.trim();
        if trimmed.len() != key.len() {
            key = trimmed.to_string();
        }

        key.make_ascii_lowercase();

        // Apply domain normalization
        match D::normalize_domain(Cow::Owned(key)) {
            Cow::Owned(s) => s,
            Cow::Borrowed(_) => unreachable!("We passed Cow::Owned"),
        }
    }

    /// Compute hash using the configured algorithm
    ///
    /// The hash algorithm is selected at compile time based on feature flags,
    /// allowing for different performance/security trade-offs.
    pub(crate) fn compute_hash(key: &str) -> u64 {
        if key.is_empty() {
            return 0;
        }
        // Priority: fast > secure > crypto > default
        // This ensures consistent behavior even when multiple features are enabled during testing

        #[cfg(feature = "fast")]
        {
            // Use GxHash for maximum performance on supported platforms
            // Falls back to AHash on unsupported platforms
            #[cfg(any(
                all(target_arch = "x86_64", target_feature = "aes"),
                all(target_arch = "aarch64", target_feature = "aes")
            ))]
            {
                // Дополнительная защита для GxHash
                if key.is_empty() {
                    return 0;
                }

                // Безопасный вызов GxHash с fallback
                #[cfg(feature = "std")]
                {
                    match std::panic::catch_unwind(|| gxhash::gxhash64(key.as_bytes(), 0)) {
                        Ok(hash) => hash,
                        Err(_) => {
                            // Fallback на простой хеш при панике GxHash
                            Self::fnv1a_hash(key.as_bytes())
                        }
                    }
                }

                #[cfg(not(feature = "std"))]
                {
                    // В no_std среде используем GxHash напрямую, но с проверками
                    if key.as_bytes().len() > 0 && key.as_bytes().len() < 1024 * 1024 {
                        gxhash::gxhash64(key.as_bytes(), 0)
                    } else {
                        // Fallback для edge cases
                        Self::fnv1a_hash(key.as_bytes())
                    }
                }
            }
            #[cfg(not(any(
                all(target_arch = "x86_64", target_feature = "aes"),
                all(target_arch = "aarch64", target_feature = "aes")
            )))]
            {
                // Fallback to AHash if GxHash requirements not met
                use core::hash::Hasher;
                let mut hasher = ahash::AHasher::default();
                hasher.write(key.as_bytes());
                return hasher.finish();
            }
        }

        #[cfg(all(feature = "secure", not(feature = "fast")))]
        {
            // Use AHash for balanced speed vs DoS resistance
            use core::hash::Hasher;
            let mut hasher = ahash::AHasher::default();
            hasher.write(key.as_bytes());
            return hasher.finish();
        }

        #[cfg(all(feature = "crypto", not(any(feature = "fast", feature = "secure"))))]
        {
            // Use Blake3 for cryptographic security
            let hash = blake3::hash(key.as_bytes());
            let bytes = hash.as_bytes();
            return u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
        }

        // Default case - no special hash features enabled
        #[cfg(not(any(feature = "fast", feature = "secure", feature = "crypto")))]
        {
            #[cfg(feature = "std")]
            {
                use std::collections::hash_map::DefaultHasher;
                let mut hasher = DefaultHasher::new();
                hasher.write(key.as_bytes());
                return hasher.finish();
            }

            #[cfg(not(feature = "std"))]
            {
                // Simple FNV-1a hash for no_std environments
                return Self::fnv1a_hash(key.as_bytes());
            }
        }
    }

    /// FNV-1a hash implementation for `no_std` environments
    #[allow(dead_code)]
    fn fnv1a_hash(bytes: &[u8]) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const FNV_PRIME: u64 = 0x0100_0000_01b3;

        let mut hash = FNV_OFFSET_BASIS;
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }
}

// ============================================================================
// SUPPORTING TYPES
// ============================================================================

/// Information about a key's validation characteristics
///
/// This structure provides detailed information about how a key was validated
/// and what domain-specific features are enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValidationInfo {
    /// Domain name
    pub domain: &'static str,
    /// Maximum allowed length for this domain
    pub max_length: usize,
    /// Actual length of the key
    pub length: usize,
    /// Whether the domain has custom validation rules
    pub has_custom_validation: bool,
    /// Whether the domain has custom normalization rules
    pub has_custom_normalization: bool,
}

// ============================================================================
// STANDARD TRAIT IMPLEMENTATIONS
// ============================================================================

/// Display implementation shows domain and key
impl<T: KeyDomain> fmt::Display for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", T::DOMAIN_NAME, self.inner)
    }
}

/// `AsRef` implementation for string conversion
impl<T: KeyDomain> AsRef<str> for Key<T> {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

/// From implementation for converting to String
impl<T: KeyDomain> From<Key<T>> for String {
    fn from(key: Key<T>) -> Self {
        key.inner.into()
    }
}

/// `FromStr` implementation for parsing from strings
impl<T: KeyDomain> FromStr for Key<T> {
    type Err = KeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Key::new(s)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DefaultDomain;
    #[cfg(not(feature = "std"))]
    use alloc::format;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    #[cfg(not(feature = "std"))]
    use alloc::vec::Vec;

    // Test domain
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct TestDomain;

    impl KeyDomain for TestDomain {
        const DOMAIN_NAME: &'static str = "test";
        const MAX_LENGTH: usize = 32;
        const HAS_CUSTOM_VALIDATION: bool = true;
        const HAS_CUSTOM_NORMALIZATION: bool = true;

        fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
            if key.starts_with("invalid_") {
                return Err(KeyParseError::domain_error(
                    Self::DOMAIN_NAME,
                    "Keys cannot start with 'invalid_'",
                ));
            }
            Ok(())
        }

        fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
            if key.contains('-') {
                Cow::Owned(key.replace('-', "_"))
            } else {
                key
            }
        }

        fn allowed_characters(c: char) -> bool {
            c.is_ascii_alphanumeric() || c == '_' || c == '-'
        }

        fn validation_help() -> Option<&'static str> {
            Some("Use alphanumeric characters, underscores, and hyphens. Cannot start with 'invalid_'.")
        }
    }

    type TestKey = Key<TestDomain>;

    #[test]
    fn test_key_creation() {
        let key = TestKey::new("valid_key").unwrap();
        assert_eq!(key.as_str(), "valid_key");
        assert_eq!(key.domain(), "test");
        assert_eq!(key.len(), 9);
    }

    #[test]
    fn test_key_normalization() {
        let key = TestKey::new("Test-Key").unwrap();
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn test_domain_validation() {
        let result = TestKey::new("invalid_key");
        assert!(result.is_err());

        if let Err(KeyParseError::DomainValidation { domain, message }) = result {
            assert_eq!(domain, "test");
            assert!(message.contains("invalid_"));
        } else {
            panic!("Expected domain validation error");
        }
    }

    #[test]
    fn test_common_validation() {
        // Empty key
        assert!(matches!(TestKey::new(""), Err(KeyParseError::Empty)));

        // Too long key
        let long_key = "a".repeat(50);
        assert!(matches!(
            TestKey::new(&long_key),
            Err(KeyParseError::TooLong {
                max_length: 32,
                actual_length: 50
            })
        ));

        // Invalid character
        let result = TestKey::new("key with spaces");
        assert!(matches!(
            result,
            Err(KeyParseError::InvalidCharacter {
                character: ' ',
                position: 3,
                ..
            })
        ));
    }

    #[test]
    fn test_hash_caching() {
        let key1 = TestKey::new("test_key").unwrap();
        let key2 = TestKey::new("test_key").unwrap();

        // Same keys should have same hash
        assert_eq!(key1.hash(), key2.hash());

        let key3 = TestKey::new("different_key").unwrap();
        // Different keys should have different hashes (with high probability)
        assert_ne!(key1.hash(), key3.hash());
    }

    #[test]
    fn test_key_methods() {
        let key = TestKey::new("test_key_example").unwrap();
        assert!(key.starts_with("test_"));
        assert!(key.ends_with("_example"));
        assert!(key.contains("_key_"));
        assert_eq!(key.len(), 16);
        assert!(!key.is_empty());
    }

    #[test]
    fn test_from_string() {
        let key = TestKey::from_string("test_key".to_string()).unwrap();
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn test_try_from_static() {
        let key = TestKey::try_from_static("static_key").unwrap();
        assert_eq!(key.as_str(), "static_key");

        let invalid = TestKey::try_from_static("");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_validation_info() {
        let key = TestKey::new("test_key").unwrap();
        let info = key.validation_info();

        assert_eq!(info.domain, "test");
        assert_eq!(info.max_length, 32);
        assert_eq!(info.length, 8);
        assert!(info.has_custom_validation);
        assert!(info.has_custom_normalization);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let key = TestKey::new("test_key").unwrap();

        // Test JSON serialization
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, r#""test_key""#);

        // Test JSON deserialization
        let deserialized: TestKey = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, key);
    }

    #[test]
    fn test_from_parts() {
        let key = TestKey::from_parts(&["user", "123", "profile"], "_").unwrap();
        assert_eq!(key.as_str(), "user_123_profile");

        let parts: Vec<&str> = key.split('_').collect();
        assert_eq!(parts, vec!["user", "123", "profile"]);
    }

    #[test]
    fn test_ensure_prefix_suffix() {
        let key = TestKey::new("profile").unwrap();

        let prefixed = key.ensure_prefix("user_").unwrap();
        assert_eq!(prefixed.as_str(), "user_profile");

        // Already has prefix
        let same = prefixed.ensure_prefix("user_").unwrap();
        assert_eq!(same.as_str(), "user_profile");

        let suffixed = key.ensure_suffix("_v1").unwrap();
        assert_eq!(suffixed.as_str(), "profile_v1");

        // Already has suffix
        let same = suffixed.ensure_suffix("_v1").unwrap();
        assert_eq!(same.as_str(), "profile_v1");
    }

    #[test]
    fn test_display_format() {
        let key = TestKey::new("example").unwrap();
        assert_eq!(format!("{key}"), "test:example");
    }

    #[test]
    fn test_string_conversion() {
        let key = TestKey::new("example").unwrap();
        let string: String = key.into();
        assert_eq!(string, "example");
    }

    #[test]
    fn test_from_str() {
        let key: TestKey = "example".parse().unwrap();
        assert_eq!(key.as_str(), "example");
    }

    #[test]
    fn test_default_domain() {
        type DefaultKey = Key<DefaultDomain>;
        let key = DefaultKey::new("test_key").unwrap();
        assert_eq!(key.domain(), "default");
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn test_length_caching() {
        let key = TestKey::new("test_key").unwrap();
        // Length should be cached and O(1)
        assert_eq!(key.len(), 8);
        assert_eq!(key.len(), 8); // Second call should use cache
    }

    #[test]
    fn test_split_operations() {
        let key = TestKey::new("user_profile_settings").unwrap();

        let parts: Vec<&str> = key.split('_').collect();
        assert_eq!(parts, vec!["user", "profile", "settings"]);

        let cached_parts: Vec<&str> = key.split_cached('_').collect();
        assert_eq!(cached_parts, vec!["user", "profile", "settings"]);

        let str_parts: Vec<&str> = key.split_str("_").collect();
        assert_eq!(str_parts, vec!["user", "profile", "settings"]);
    }
}
