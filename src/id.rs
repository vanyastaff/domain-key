//! Typed numeric identifier for domain-key
//!
//! This module provides `Id<D>`, a lightweight, type-safe numeric identifier
//! parameterized by a domain marker. It wraps a [`NonZeroU64`] value, enforcing
//! the invariant that identifiers are always non-zero (as is standard for
//! database primary keys, entity IDs, etc.).
//!
//! # Niche Optimization
//!
//! Because the inner value is `NonZeroU64`, `Option<Id<D>>` has the same size
//! as `Id<D>` itself (8 bytes) — zero-cost optionality.
//!
//! # Examples
//!
//! ```rust
//! use domain_key::{Domain, IdDomain, Id};
//!
//! #[derive(Debug)]
//! struct UserDomain;
//!
//! impl Domain for UserDomain {
//!     const DOMAIN_NAME: &'static str = "user";
//! }
//! impl IdDomain for UserDomain {}
//!
//! type UserId = Id<UserDomain>;
//!
//! let id = UserId::new(42).unwrap();
//! assert_eq!(id.get(), 42);
//! assert_eq!(id.to_string(), "42");
//!
//! // Zero is rejected:
//! assert!(UserId::new(0).is_none());
//!
//! // Option<Id> has the same size as Id:
//! assert_eq!(std::mem::size_of::<Option<UserId>>(), std::mem::size_of::<UserId>());
//! ```

use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroU64;
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::domain::IdDomain;
use crate::error::IdParseError;

// ============================================================================
// CORE ID IMPLEMENTATION
// ============================================================================

/// Lightweight, type-safe numeric identifier
///
/// `Id<D>` wraps a [`NonZeroU64`] with a phantom domain marker, providing
/// compile-time type safety for numeric identifiers. It is `Copy`, making it
/// ideal for use as database primary keys, entity IDs, and similar use cases.
///
/// The non-zero invariant means `Option<Id<D>>` is the same size as `Id<D>`
/// (niche optimization), and zero can never accidentally represent a valid ID.
///
/// # Domain Trait
///
/// `Id<D>` uses [`IdDomain`] — a lightweight marker trait with only a domain name.
/// No string validation or normalization needed, unlike [`KeyDomain`](crate::KeyDomain).
///
/// # Memory Layout
///
/// ```text
/// Id<D> struct (8 bytes, niche-optimized):
/// ┌──────────────────┬─────────────┐
/// │ NonZeroU64 (8B)  │ marker (0B) │
/// └──────────────────┴─────────────┘
///
/// Option<Id<D>>: also 8 bytes (None = 0)
/// ```
///
/// # Type Safety
///
/// Different domains produce incompatible ID types at compile time:
///
/// ```rust,compile_fail
/// use domain_key::{Id, Domain, IdDomain};
///
/// #[derive(Debug)]
/// struct UserDomain;
/// impl Domain for UserDomain { const DOMAIN_NAME: &'static str = "user"; }
/// impl IdDomain for UserDomain {}
///
/// #[derive(Debug)]
/// struct OrderDomain;
/// impl Domain for OrderDomain { const DOMAIN_NAME: &'static str = "order"; }
/// impl IdDomain for OrderDomain {}
///
/// type UserId = Id<UserDomain>;
/// type OrderId = Id<OrderDomain>;
///
/// let user_id = UserId::new(1).unwrap();
/// let order_id: OrderId = user_id; // Compile error!
/// ```
pub struct Id<D: IdDomain> {
    value: NonZeroU64,
    _marker: PhantomData<D>,
}

// Manual trait impls to avoid requiring D: Copy/Clone/PartialEq/Hash/etc.
// The phantom marker D is a ZST and never contributes to these operations.

impl<D: IdDomain> fmt::Debug for Id<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", D::DOMAIN_NAME, self.value)
    }
}

impl<D: IdDomain> Copy for Id<D> {}

impl<D: IdDomain> Clone for Id<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: IdDomain> PartialEq for Id<D> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<D: IdDomain> Eq for Id<D> {}

impl<D: IdDomain> PartialOrd for Id<D> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<D: IdDomain> Ord for Id<D> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<D: IdDomain> core::hash::Hash for Id<D> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<D: IdDomain> Id<D> {
    /// Creates a new typed identifier from a `u64` value.
    ///
    /// Returns `None` if `value` is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Id, Domain, IdDomain};
    ///
    /// #[derive(Debug)]
    /// struct UserDomain;
    /// impl Domain for UserDomain { const DOMAIN_NAME: &'static str = "user"; }
    /// impl IdDomain for UserDomain {}
    ///
    /// type UserId = Id<UserDomain>;
    ///
    /// assert!(UserId::new(1).is_some());
    /// assert!(UserId::new(0).is_none());
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(nz) => Some(Self {
                value: nz,
                _marker: PhantomData,
            }),
            None => None,
        }
    }

    /// Creates a new typed identifier from a [`NonZeroU64`] value.
    #[inline]
    #[must_use]
    pub const fn from_non_zero(value: NonZeroU64) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying value as `u64`.
    #[inline]
    #[must_use]
    pub const fn get(&self) -> u64 {
        self.value.get()
    }

    /// Returns the underlying [`NonZeroU64`] value.
    #[inline]
    #[must_use]
    pub const fn non_zero(&self) -> NonZeroU64 {
        self.value
    }

    /// Returns the domain name for this identifier type.
    #[inline]
    #[must_use]
    pub fn domain(&self) -> &'static str {
        D::DOMAIN_NAME
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS
// ============================================================================

// Note: Id<D> intentionally does not implement Default
// because there is no meaningful default for a non-zero identifier.

impl<D: IdDomain> fmt::Display for Id<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<D: IdDomain> FromStr for Id<D> {
    type Err = IdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<NonZeroU64>()?;
        Ok(Self::from_non_zero(value))
    }
}

impl<D: IdDomain> From<NonZeroU64> for Id<D> {
    #[inline]
    fn from(value: NonZeroU64) -> Self {
        Self::from_non_zero(value)
    }
}

impl<D: IdDomain> From<Id<D>> for NonZeroU64 {
    #[inline]
    fn from(id: Id<D>) -> Self {
        id.value
    }
}

impl<D: IdDomain> From<Id<D>> for u64 {
    #[inline]
    fn from(id: Id<D>) -> Self {
        id.value.get()
    }
}

impl<D: IdDomain> TryFrom<u64> for Id<D> {
    type Error = IdParseError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(IdParseError::Zero)
    }
}

impl<D: IdDomain> TryFrom<&str> for Id<D> {
    type Error = IdParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl<D: IdDomain> TryFrom<String> for Id<D> {
    type Error = IdParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// ============================================================================
// SERDE SUPPORT
// ============================================================================

#[cfg(feature = "serde")]
impl<D: IdDomain> Serialize for Id<D> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, D: IdDomain> Deserialize<'de> for Id<D> {
    fn deserialize<De: serde::Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        let value = NonZeroU64::deserialize(deserializer)?;
        Ok(Self::from_non_zero(value))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestDomain;
    impl crate::Domain for TestDomain {
        const DOMAIN_NAME: &'static str = "test";
    }
    impl IdDomain for TestDomain {}

    type TestId = Id<TestDomain>;

    #[test]
    fn test_new_and_get() {
        let id = TestId::new(42).unwrap();
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn test_zero_rejected() {
        assert!(TestId::new(0).is_none());
    }

    #[test]
    fn test_try_from_u64_zero() {
        let result = TestId::try_from(0u64);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IdParseError::Zero));
    }

    #[test]
    fn test_try_from_u64_nonzero() {
        let id = TestId::try_from(42u64).unwrap();
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn test_from_non_zero() {
        let nz = NonZeroU64::new(7).unwrap();
        let id = TestId::from_non_zero(nz);
        assert_eq!(id.get(), 7);
        assert_eq!(id.non_zero(), nz);
    }

    #[test]
    fn test_debug_format() {
        let id = TestId::new(42).unwrap();
        assert_eq!(format!("{:?}", id), "test(42)");
    }

    #[test]
    fn test_domain() {
        let id = TestId::new(1).unwrap();
        assert_eq!(id.domain(), "test");
    }

    #[test]
    fn test_display() {
        let id = TestId::new(12345).unwrap();
        assert_eq!(id.to_string(), "12345");
    }

    #[test]
    fn test_from_str() {
        let id: TestId = "42".parse().unwrap();
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn test_from_str_zero() {
        let result: Result<TestId, _> = "0".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<TestId, _> = "not_a_number".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_non_zero_u64() {
        let nz = NonZeroU64::new(100).unwrap();
        let id: TestId = nz.into();
        assert_eq!(id.get(), 100);
    }

    #[test]
    fn test_into_non_zero_u64() {
        let id = TestId::new(99).unwrap();
        let nz: NonZeroU64 = id.into();
        assert_eq!(nz.get(), 99);
    }

    #[test]
    fn test_into_u64() {
        let id = TestId::new(99).unwrap();
        let value: u64 = id.into();
        assert_eq!(value, 99);
    }

    #[test]
    fn test_try_from_str() {
        let id = TestId::try_from("7").unwrap();
        assert_eq!(id.get(), 7);
    }

    #[test]
    fn test_try_from_string() {
        let id = TestId::try_from(String::from("123")).unwrap();
        assert_eq!(id.get(), 123);
    }

    #[test]
    fn test_copy() {
        let id1 = TestId::new(5).unwrap();
        let id2 = id1; // Copy
        assert_eq!(id1, id2); // id1 still valid
    }

    #[test]
    fn test_ord() {
        let a = TestId::new(1).unwrap();
        let b = TestId::new(2).unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_hash() {
        use core::hash::{Hash, Hasher};
        let id1 = TestId::new(42).unwrap();
        let id2 = TestId::new(42).unwrap();

        let hash = |id: &TestId| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            id.hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(hash(&id1), hash(&id2));
    }

    #[test]
    fn test_max_id() {
        let id = TestId::new(u64::MAX).unwrap();
        assert_eq!(id.get(), u64::MAX);
    }

    #[test]
    fn test_option_niche_optimization() {
        assert_eq!(
            core::mem::size_of::<Option<TestId>>(),
            core::mem::size_of::<TestId>()
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let id = TestId::new(42).unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
        let deserialized: TestId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_zero_rejected() {
        let result: Result<TestId, _> = serde_json::from_str("0");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_safety_different_domains() {
        #[derive(Debug)]
        struct DomainA;
        impl crate::Domain for DomainA {
            const DOMAIN_NAME: &'static str = "a";
        }
        impl IdDomain for DomainA {}

        #[derive(Debug)]
        struct DomainB;
        impl crate::Domain for DomainB {
            const DOMAIN_NAME: &'static str = "b";
        }
        impl IdDomain for DomainB {}

        let _a: Id<DomainA> = Id::new(1).unwrap();
        let _b: Id<DomainB> = Id::new(1).unwrap();

        // These are different types — cannot be compared or assigned
        // _a == _b would not compile
    }
}
