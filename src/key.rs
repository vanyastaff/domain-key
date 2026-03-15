//! Core Key implementation for domain-key
//!
//! This module contains the main `Key<T>` structure and its implementation,
//! providing high-performance, type-safe key handling with extensive optimizations.

use core::borrow::Borrow;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::Deref;
use core::str::FromStr;

#[cfg(not(feature = "std"))]
use alloc::borrow::{Cow, ToOwned};
#[cfg(not(feature = "std"))]
use alloc::string::String;

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

// ============================================================================
// SPLIT ITERATOR TYPES
// ============================================================================

/// Split cache type for consistent API
pub type SplitCache<'a> = core::str::Split<'a, char>;

/// Split iterator with consistent API
#[derive(Debug)]
pub struct SplitIterator<'a>(SplitCache<'a>);

impl<'a> Iterator for SplitIterator<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
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
/// - **Hash Access**: O(1) via pre-computed hash (`.hash() -> u64`)
/// - **Length Access**: O(1) via `SmartString` (inline length)
/// - **String Access**: Direct reference to internal storage
/// - **`HashMap` Lookup**: by `&str` via `Borrow<str>` — no temporary key needed
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
/// ┌─────────────────────┬──────────┬─────────────┐
/// │ SmartString (24B)   │ hash (8B)│ marker (0B) │
/// └─────────────────────┴──────────┴─────────────┘
/// ```
///
/// Keys use `SmartString` which stores strings up to 23 bytes inline on the stack,
/// only allocating on the heap for longer strings. The pre-computed hash (feature-
/// selected algorithm) is accessible via `.hash() -> u64`. The `Hash` trait
/// delegates to `str` so that `Borrow<str>` works correctly with `HashMap`.
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, Domain, KeyDomain};
///
/// #[derive(Debug)]
/// struct UserDomain;
///
/// impl Domain for UserDomain {
///     const DOMAIN_NAME: &'static str = "user";
/// }
///
/// impl KeyDomain for UserDomain {
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
#[derive(Debug)]
pub struct Key<T: KeyDomain> {
    /// Internal string storage using `SmartString` for optimal memory usage
    inner: SmartString,

    /// Pre-computed hash value accessible via [`Key::hash()`]
    ///
    /// This hash is computed once during key creation using the
    /// feature-selected algorithm (gxhash / ahash / blake3 / fnv-1a).
    /// It is **not** used by the [`Hash`] trait implementation — that
    /// one delegates to `self.inner` so that the `Borrow<str>` contract
    /// (`hash(key) == hash(key.borrow())`) is upheld.
    hash: u64,

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
            _marker: PhantomData,
        }
    }
}

// Manual PartialEq/Eq — compare only the key string, not cached fields
impl<T: KeyDomain> PartialEq for Key<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: KeyDomain> Eq for Key<T> {}

// Manual PartialOrd/Ord — compare only the key string
impl<T: KeyDomain> PartialOrd for Key<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: KeyDomain> Ord for Key<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

// Hash implementation delegates to the inner string so that the
// Borrow<str> contract is satisfied: hash(key) == hash(key.borrow()).
// This allows HashMap<Key<T>, V>::get("some_str") to work correctly.
impl<T: KeyDomain> Hash for Key<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
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
            Key::new(s).map_err(serde::de::Error::custom)
        } else {
            // For binary formats, deserialize as owned string
            let s = String::deserialize(deserializer)?;
            Key::from_string(s).map_err(serde::de::Error::custom)
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
        // Step 1: Reject truly empty input before normalization; whitespace-only
        // strings are handled by normalize() + validate_common() below.
        if key.is_empty() {
            return Err(KeyParseError::Empty);
        }

        // Step 2: Normalization (trimming, lowercasing, domain-specific)
        let normalized = Self::normalize(key);

        // Step 3: Common validation on the normalized result
        Self::validate_common(&normalized)?;

        // Step 4: Domain-specific validation
        T::validate_domain_rules(&normalized).map_err(Self::fix_domain_error)?;

        // Step 5: Hash computation and storage
        let hash = Self::compute_hash(&normalized);

        Ok(Self {
            inner: SmartString::from(normalized.as_ref()),
            hash,
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key_string = "test_key".to_string();
    /// let key = TestKey::from_string(key_string)?;
    /// assert_eq!(key.as_str(), "test_key");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    pub fn from_string(key: String) -> Result<Self, KeyParseError> {
        // Reject obviously empty input before normalization
        if key.trim().is_empty() {
            return Err(KeyParseError::Empty);
        }

        // Normalize efficiently, reusing allocation when possible
        let normalized = Self::normalize_owned(key);

        // Validate the normalized result
        Self::validate_common(&normalized)?;

        // Domain validation
        T::validate_domain_rules(&normalized).map_err(Self::fix_domain_error)?;

        let hash = Self::compute_hash(&normalized);

        Ok(Self {
            inner: SmartString::from(normalized),
            hash,
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    /// # Warning
    ///
    /// The caller must ensure that the static string follows all validation
    /// rules for the domain (allowed characters, length limits, normalization,
    /// domain-specific rules). Invalid keys created this way will violate
    /// internal invariants and may cause unexpected behavior.
    ///
    /// Prefer [`try_from_static`](Self::try_from_static) or the [`static_key!`](macro@crate::static_key) macro
    /// for safe creation of static keys.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the key is empty or exceeds `T::MAX_LENGTH`.
    ///
    /// # Arguments
    ///
    /// * `key` - A static string literal that represents a valid key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::from_static_unchecked("static_key");
    /// assert_eq!(key.as_str(), "static_key");
    /// ```
    #[must_use]
    pub fn from_static_unchecked(key: &'static str) -> Self {
        debug_assert!(
            !key.is_empty(),
            "from_static_unchecked: key must not be empty"
        );
        debug_assert!(
            key.len() <= T::MAX_LENGTH,
            "from_static_unchecked: key length {} exceeds domain max {}",
            key.len(),
            T::MAX_LENGTH
        );

        let hash = Self::compute_hash(key);

        Self {
            inner: SmartString::from(key),
            hash,
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
        // Validate and create via the normal path
        Self::new(key)
    }

    /// Try to create a key, returning None on validation failure
    ///
    /// This is a convenience method for when you want to handle validation
    /// failures by ignoring invalid keys rather than handling errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    /// Check whether a string satisfies the **default** [`KeyDomain`] validation
    /// rules for this domain at compile time.
    ///
    /// This is a thin `const fn` wrapper around [`is_valid_key_default`] that
    /// automatically uses `T::MAX_LENGTH` as the length limit, so you do not
    /// need to repeat the constant at every call site.
    ///
    /// Only the *default* rules are checked (character set, length, consecutive
    /// separators, end-character constraint).  Custom domain rules added via
    /// [`KeyDomain::validate_domain_rules`] are **not** verified — those are
    /// enforced at runtime by [`Key::new`] / [`Key::try_from_static`].
    ///
    /// # Use cases
    ///
    /// * Compile-time `const` assertions to document that a literal is valid:
    ///
    /// ```rust
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct MyDomain;
    /// impl Domain for MyDomain { const DOMAIN_NAME: &'static str = "my"; }
    /// impl KeyDomain for MyDomain {}
    /// type MyKey = Key<MyDomain>;
    ///
    /// // Evaluated at compile time — zero runtime cost, compile error on failure
    /// const _: () = assert!(MyKey::is_valid_key_const("hello_world"));
    /// const _: () = assert!(!MyKey::is_valid_key_const(""));
    /// ```
    ///
    /// * Used internally by [`static_key!`] to turn invalid literals into
    ///   **compile errors** rather than runtime panics.
    ///
    /// [`is_valid_key_default`]: crate::is_valid_key_default
    #[must_use]
    pub const fn is_valid_key_const(s: &str) -> bool {
        crate::validation::is_valid_key_default(s, T::MAX_LENGTH)
    }

    /// Returns the key as a string slice
    ///
    /// This is the primary way to access the string content of a key.
    /// The returned reference is valid for the lifetime of the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// assert_eq!(key.as_str(), "example");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct UserDomain;
    /// impl Domain for UserDomain {
    ///     const DOMAIN_NAME: &'static str = "user";
    /// }
    /// impl KeyDomain for UserDomain {}
    /// type UserKey = Key<UserDomain>;
    ///
    /// let key = UserKey::new("john")?;
    /// assert_eq!(key.domain(), "user");
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub const fn domain(&self) -> &'static str {
        T::DOMAIN_NAME
    }

    /// Returns the length of the key string in bytes
    ///
    /// This is an O(1) operation — `SmartString` stores the length inline.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// assert_eq!(key.len(), 7);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// assert!(!key.is_empty());
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the pre-computed hash value
    ///
    /// This hash is computed once during key creation using the
    /// feature-selected algorithm (gxhash / ahash / blake3 / fnv-1a)
    /// and cached for the lifetime of the key.
    ///
    /// **Important:** This is *not* the hash used by [`Hash`] trait /
    /// `HashMap`.  The `Hash` trait delegates to `str`'s implementation
    /// so that `Borrow<str>` works correctly.  Use this method when you
    /// need a deterministic, feature-dependent hash for your own data
    /// structures or protocols.
    ///
    /// **Note:** The hash algorithm depends on the active feature flags
    /// (`fast`, `secure`, `crypto`, or the default hasher). Keys created
    /// with different feature configurations will produce different hash
    /// values. Do not persist or compare hash values across builds with
    /// different features.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    #[inline]
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("abc")?;
    /// let chars: Vec<char> = key.chars().collect();
    /// assert_eq!(chars, vec!['a', 'b', 'c']);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[inline]
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("user_profile_settings")?;
    /// let parts: Vec<&str> = key.split('_').collect();
    /// assert_eq!(parts, vec!["user", "profile", "settings"]);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[must_use]
    pub fn split(&self, delimiter: char) -> SplitIterator<'_> {
        SplitIterator(utils::new_split_cache(&self.inner, delimiter))
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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

        let result = utils::add_prefix_optimized(&self.inner, prefix);

        // Full structural validation (start char, end char, consecutive chars at junction)
        Self::validate_common(&result)?;

        T::validate_domain_rules(&result).map_err(Self::fix_domain_error)?;

        let hash = Self::compute_hash(&result);

        Ok(Self {
            inner: result,
            hash,
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {}
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

        let result = utils::add_suffix_optimized(&self.inner, suffix);

        // Full structural validation (start char, end char, consecutive chars at junction)
        Self::validate_common(&result)?;

        T::validate_domain_rules(&result).map_err(Self::fix_domain_error)?;

        let hash = Self::compute_hash(&result);

        Ok(Self {
            inner: result,
            hash,
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
    /// use domain_key::{Key, Domain, KeyDomain};
    ///
    /// #[derive(Debug)]
    /// struct TestDomain;
    /// impl Domain for TestDomain {
    ///     const DOMAIN_NAME: &'static str = "test";
    /// }
    /// impl KeyDomain for TestDomain {
    ///     const MAX_LENGTH: usize = 32;
    ///     const HAS_CUSTOM_VALIDATION: bool = true;
    /// }
    /// type TestKey = Key<TestDomain>;
    ///
    /// let key = TestKey::new("example")?;
    /// let info = key.validation_info();
    ///
    /// assert_eq!(info.domain_info.name, "test");
    /// assert_eq!(info.domain_info.max_length, 32);
    /// assert_eq!(info.length, 7);
    /// assert!(info.domain_info.has_custom_validation);
    /// # Ok::<(), domain_key::KeyParseError>(())
    /// ```
    #[must_use]
    pub fn validation_info(&self) -> KeyValidationInfo {
        KeyValidationInfo {
            domain_info: crate::domain::domain_info::<T>(),
            length: self.len(),
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
    pub(crate) fn validate_common(key: &str) -> Result<(), KeyParseError> {
        // The caller (new_optimized / from_string) has already normalized
        // the key (which includes trimming), so we validate `key` directly
        // without an extra .trim() pass.

        if key.is_empty() {
            return Err(KeyParseError::Empty);
        }

        if key.len() > T::MAX_LENGTH {
            return Err(KeyParseError::TooLong {
                max_length: T::MAX_LENGTH,
                actual_length: key.len(),
            });
        }

        if key.len() < T::min_length() {
            return Err(KeyParseError::TooShort {
                min_length: T::min_length(),
                actual_length: key.len(),
            });
        }

        // Use fast validation
        Self::validate_fast(key)
    }

    /// Fast validation path using optimized algorithms
    /// # Errors
    ///
    /// Returns `KeyParseError` if the prefixed key would be invalid or too long
    fn validate_fast(key: &str) -> Result<(), KeyParseError> {
        let mut chars = key.char_indices();
        let mut prev_char = None;

        // Validate first character
        if let Some((pos, first)) = chars.next() {
            let char_allowed = crate::utils::char_validation::is_key_char_fast(first)
                || T::allowed_start_character(first);

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
            let char_allowed = T::allowed_characters(c);

            if !char_allowed {
                return Err(KeyParseError::InvalidCharacter {
                    character: c,
                    position: pos,
                    expected: Some("allowed by domain"),
                });
            }

            if let Some(prev) = prev_char {
                if !T::allowed_consecutive_characters(prev, c) {
                    return Err(KeyParseError::InvalidStructure {
                        reason: "consecutive characters not allowed",
                    });
                }
            }
            prev_char = Some(c);
        }

        // Check last character
        if let Some(last) = prev_char {
            if !T::allowed_end_character(last) {
                return Err(KeyParseError::InvalidStructure {
                    reason: "invalid end character",
                });
            }
        }

        Ok(())
    }

    /// Normalize a borrowed string
    pub(crate) fn normalize(key: &str) -> Cow<'_, str> {
        let trimmed = key.trim();

        let needs_lowercase =
            T::CASE_INSENSITIVE && trimmed.chars().any(|c| c.is_ascii_uppercase());

        let base = if needs_lowercase {
            Cow::Owned(trimmed.to_ascii_lowercase())
        } else {
            // Borrow the trimmed slice — no allocation needed
            Cow::Borrowed(trimmed)
        };

        // Apply domain-specific normalization
        T::normalize_domain(base)
    }

    /// Normalize an owned string efficiently
    fn normalize_owned(mut key: String) -> String {
        // In-place trim: remove leading whitespace by draining, then truncate trailing
        let start = key.len() - key.trim_start().len();
        if start > 0 {
            key.drain(..start);
        }
        let trimmed_len = key.trim_end().len();
        key.truncate(trimmed_len);

        if T::CASE_INSENSITIVE {
            key.make_ascii_lowercase();
        }

        // Apply domain normalization
        match T::normalize_domain(Cow::Owned(key)) {
            Cow::Owned(s) => s,
            Cow::Borrowed(s) => s.to_owned(),
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

        Self::compute_hash_inner(key.as_bytes())
    }

    /// Inner hash computation dispatched by feature flags
    ///
    /// Separated to keep each cfg branch as the sole return path,
    /// avoiding mixed `return` statements and dead-code warnings.
    fn compute_hash_inner(bytes: &[u8]) -> u64 {
        // Priority: fast > secure > crypto > default

        #[cfg(feature = "fast")]
        {
            #[cfg(any(
                all(target_arch = "x86_64", target_feature = "aes"),
                all(
                    target_arch = "aarch64",
                    target_feature = "aes",
                    target_feature = "neon"
                )
            ))]
            {
                gxhash::gxhash64(bytes, 0)
            }

            #[cfg(not(any(
                all(target_arch = "x86_64", target_feature = "aes"),
                all(
                    target_arch = "aarch64",
                    target_feature = "aes",
                    target_feature = "neon"
                )
            )))]
            {
                use core::hash::Hasher;
                let mut hasher = ahash::AHasher::default();
                hasher.write(bytes);
                hasher.finish()
            }
        }

        #[cfg(all(feature = "secure", not(feature = "fast")))]
        {
            use core::hash::Hasher;
            let mut hasher = ahash::AHasher::default();
            hasher.write(bytes);
            hasher.finish()
        }

        #[cfg(all(feature = "crypto", not(any(feature = "fast", feature = "secure"))))]
        {
            let hash = blake3::hash(bytes);
            let h = hash.as_bytes();
            u64::from_le_bytes([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]])
        }

        #[cfg(not(any(feature = "fast", feature = "secure", feature = "crypto")))]
        {
            #[cfg(feature = "std")]
            {
                use core::hash::Hasher;
                use std::collections::hash_map::DefaultHasher;
                let mut hasher = DefaultHasher::new();
                hasher.write(bytes);
                hasher.finish()
            }

            #[cfg(not(feature = "std"))]
            {
                Self::fnv1a_hash(bytes)
            }
        }
    }

    /// FNV-1a hash implementation for `no_std` environments
    #[expect(
        dead_code,
        reason = "fallback hash used only when no hash feature is enabled"
    )]
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
/// and what domain-specific features are enabled. Domain-level configuration
/// is available through the embedded [`DomainInfo`](crate::DomainInfo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValidationInfo {
    /// Full domain configuration
    pub domain_info: crate::domain::DomainInfo,
    /// Actual length of the key
    pub length: usize,
}

// ============================================================================
// STANDARD TRAIT IMPLEMENTATIONS
// ============================================================================

/// Display implementation shows the key value
///
/// Outputs just the key string, consistent with `AsRef<str>`, `From<Key<T>> for String`,
/// and serde serialization. Use [`Key::domain`] separately when domain context is needed.
impl<T: KeyDomain> fmt::Display for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner)
    }
}

/// `Deref` implementation for automatic coercion to `&str`
///
/// This allows `&key` to automatically coerce to `&str` in contexts
/// that expect a string slice, eliminating the need for explicit
/// `.as_ref()` or `.as_str()` calls in most situations.
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, Domain, KeyDomain};
///
/// #[derive(Debug)]
/// struct TestDomain;
/// impl Domain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
/// impl KeyDomain for TestDomain {}
/// type TestKey = Key<TestDomain>;
///
/// let key = TestKey::new("example")?;
///
/// // Automatic coercion — no .as_ref() needed
/// let s: &str = &key;
/// assert_eq!(s, "example");
///
/// // Works with functions expecting &str
/// fn takes_str(_s: &str) {}
/// takes_str(&key);
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
impl<T: KeyDomain> Deref for Key<T> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.inner
    }
}

/// `AsRef` implementation for string conversion
impl<T: KeyDomain> AsRef<str> for Key<T> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

/// `Borrow<str>` implementation enabling `HashMap<Key<T>, V>::get("str")`
///
/// This is sound because the [`Hash`] trait implementation for `Key<T>`
/// delegates to `str`'s hash, satisfying the contract
/// `hash(key) == hash(key.borrow())`.
impl<T: KeyDomain> Borrow<str> for Key<T> {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

/// From implementation for converting to String
impl<T: KeyDomain> From<Key<T>> for String {
    fn from(key: Key<T>) -> Self {
        key.inner.into()
    }
}

/// Creates a `Key` from a pre-validated [`SmartString`] **without re-validation**.
///
/// This is intended for internal or advanced usage where the caller has
/// already ensured that the string satisfies all domain rules (length,
/// allowed characters, normalization, etc.).  Hash is computed
/// automatically, but **no validation or normalization is performed**.
///
/// # Safety (logical)
///
/// If the string does not satisfy the domain's invariants the resulting
/// key will silently violate those invariants.  Prefer [`Key::new`] or
/// [`Key::from_string`] unless you are certain the input is valid.
impl<T: KeyDomain> From<SmartString> for Key<T> {
    #[inline]
    fn from(inner: SmartString) -> Self {
        let hash = Self::compute_hash(&inner);

        Self {
            inner,
            hash,
            _marker: PhantomData,
        }
    }
}

/// `TryFrom<String>` implementation for owned string conversion
///
/// This avoids re-borrowing through `&str` when you already have a `String`.
impl<T: KeyDomain> TryFrom<String> for Key<T> {
    type Error = KeyParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Key::from_string(s)
    }
}

/// `TryFrom<&str>` implementation for borrowed string conversion
impl<T: KeyDomain> TryFrom<&str> for Key<T> {
    type Error = KeyParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Key::new(s)
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
    use crate::domain::{DefaultDomain, Domain};
    #[cfg(not(feature = "std"))]
    use alloc::format;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    #[cfg(not(feature = "std"))]
    use alloc::vec::Vec;

    // Test domain
    #[derive(Debug)]
    struct TestDomain;

    impl Domain for TestDomain {
        const DOMAIN_NAME: &'static str = "test";
    }

    impl KeyDomain for TestDomain {
        const MAX_LENGTH: usize = 32;
        const HAS_CUSTOM_VALIDATION: bool = true;
        const HAS_CUSTOM_NORMALIZATION: bool = true;
        const CASE_INSENSITIVE: bool = true;

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
    fn new_key_stores_value_and_domain() {
        let key = TestKey::new("valid_key").unwrap();
        assert_eq!(key.as_str(), "valid_key");
        assert_eq!(key.domain(), "test");
        assert_eq!(key.len(), 9);
    }

    #[test]
    fn case_insensitive_domain_lowercases_and_normalizes() {
        let key = TestKey::new("Test-Key").unwrap();
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn domain_rules_reject_invalid_prefix() {
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
    fn rejects_empty_too_long_and_invalid_characters() {
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
    fn equal_keys_produce_same_hash() {
        use core::hash::{Hash, Hasher};

        let key1 = TestKey::new("test_key").unwrap();
        let key2 = TestKey::new("test_key").unwrap();

        // Pre-computed hashes should match
        assert_eq!(key1.hash(), key2.hash());

        let key3 = TestKey::new("different_key").unwrap();
        assert_ne!(key1.hash(), key3.hash());

        // Hash trait should produce same result as hashing the raw &str,
        // so Borrow<str> contract is upheld.
        #[cfg(feature = "std")]
        {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            Hash::hash(&key1, &mut h);
            let key_trait_hash = h.finish();
            let mut h = std::collections::hash_map::DefaultHasher::new();
            Hash::hash(key1.as_str(), &mut h);
            let str_trait_hash = h.finish();
            assert_eq!(key_trait_hash, str_trait_hash);
        }
    }

    #[test]
    fn string_query_methods_work_correctly() {
        let key = TestKey::new("test_key_example").unwrap();
        assert!(key.starts_with("test_"));
        assert!(key.ends_with("_example"));
        assert!(key.contains("_key_"));
        assert_eq!(key.len(), 16);
        assert!(!key.is_empty());
    }

    #[test]
    fn from_string_validates_owned_input() {
        let key = TestKey::from_string("test_key".to_string()).unwrap();
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn try_from_static_rejects_empty_string() {
        let key = TestKey::try_from_static("static_key").unwrap();
        assert_eq!(key.as_str(), "static_key");

        let invalid = TestKey::try_from_static("");
        assert!(invalid.is_err());
    }

    #[test]
    fn validation_info_reflects_domain_config() {
        let key = TestKey::new("test_key").unwrap();
        let info = key.validation_info();

        assert_eq!(info.domain_info.name, "test");
        assert_eq!(info.domain_info.max_length, 32);
        assert_eq!(info.length, 8);
        assert!(info.domain_info.has_custom_validation);
        assert!(info.domain_info.has_custom_normalization);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_preserves_key() {
        let key = TestKey::new("test_key").unwrap();

        // Test JSON serialization
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, r#""test_key""#);

        // Test JSON deserialization
        let deserialized: TestKey = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, key);
    }

    #[test]
    fn from_parts_joins_and_splits_roundtrip() {
        let key = TestKey::from_parts(&["user", "123", "profile"], "_").unwrap();
        assert_eq!(key.as_str(), "user_123_profile");

        let parts: Vec<&str> = key.split('_').collect();
        assert_eq!(parts, vec!["user", "123", "profile"]);
    }

    #[test]
    fn ensure_prefix_suffix_is_idempotent() {
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
    fn display_shows_raw_key_value() {
        let key = TestKey::new("example").unwrap();
        assert_eq!(format!("{key}"), "example");
    }

    #[test]
    fn into_string_extracts_value() {
        let key = TestKey::new("example").unwrap();
        let string: String = key.into();
        assert_eq!(string, "example");
    }

    #[test]
    fn parse_str_creates_validated_key() {
        let key: TestKey = "example".parse().unwrap();
        assert_eq!(key.as_str(), "example");
    }

    #[test]
    fn default_domain_accepts_simple_keys() {
        type DefaultKey = Key<DefaultDomain>;
        let key = DefaultKey::new("test_key").unwrap();
        assert_eq!(key.domain(), "default");
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn len_returns_consistent_cached_value() {
        let key = TestKey::new("test_key").unwrap();
        assert_eq!(key.len(), 8);
        assert_eq!(key.len(), 8); // Second call — same result
    }

    #[test]
    fn split_methods_produce_same_parts() {
        let key = TestKey::new("user_profile_settings").unwrap();

        let parts: Vec<&str> = key.split('_').collect();
        assert_eq!(parts, vec!["user", "profile", "settings"]);

        let cached_parts: Vec<&str> = key.split_cached('_').collect();
        assert_eq!(cached_parts, vec!["user", "profile", "settings"]);

        let str_parts: Vec<&str> = key.split_str("_").collect();
        assert_eq!(str_parts, vec!["user", "profile", "settings"]);
    }

    #[test]
    fn deref_coerces_to_str() {
        fn takes_str(s: &str) -> &str {
            s
        }
        let key = TestKey::new("hello").unwrap();
        // Deref allows &Key<T> → &str coercion
        let s: &str = &key;
        assert_eq!(s, "hello");

        // Works with functions that accept &str
        assert_eq!(takes_str(&key), "hello");
    }

    #[test]
    fn from_smartstring_creates_key_without_revalidation() {
        use smartstring::alias::String as SmartString;

        let smart = SmartString::from("pre_validated");
        let key: TestKey = TestKey::from(smart);
        assert_eq!(key.as_str(), "pre_validated");
        assert_eq!(key.len(), 13);
        // Hash should be computed correctly
        assert_ne!(key.hash(), 0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn borrow_str_enables_hashmap_get_by_str() {
        use std::collections::HashMap;

        let mut map: HashMap<TestKey, u32> = HashMap::new();
        let key = TestKey::new("lookup_test").unwrap();
        map.insert(key, 42);

        // Lookup by &str — works thanks to Borrow<str>
        assert_eq!(map.get("lookup_test"), Some(&42));
        assert_eq!(map.get("nonexistent"), None);
    }

    #[test]
    fn struct_is_32_bytes() {
        // SmartString(24) + u64 hash(8) + PhantomData(0) = 32 bytes
        assert_eq!(core::mem::size_of::<TestKey>(), 32);
    }
}
