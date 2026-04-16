//! Typed UUID identifier for domain-key
//!
//! This module provides `Uuid<D>`, a type-safe UUID identifier parameterized
//! by a domain marker. It wraps [`uuid::Uuid`] with zero overhead beyond
//! the phantom type parameter.
//!
//! # Feature Flags
//!
//! - `uuid` — enables `Uuid<D>` and `UuidParseError`
//! - `uuid-v4` — enables `Uuid::new()` for random UUID v4 generation
//! - `uuid-v7` — enables `Uuid::now_v7()` for time-ordered UUID generation
//!
//! # Examples
//!
//! ```rust
//! use domain_key::{Uuid, Domain, UuidDomain};
//!
//! #[derive(Debug)]
//! struct OrderDomain;
//!
//! impl Domain for OrderDomain {
//!     const DOMAIN_NAME: &'static str = "order";
//! }
//! impl UuidDomain for OrderDomain {}
//!
//! type OrderUuid = Uuid<OrderDomain>;
//!
//! // Parse from string
//! let id: OrderUuid = "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
//! assert_eq!(id.domain(), "order");
//!
//! // Nil UUID
//! let nil = OrderUuid::nil();
//! assert!(nil.is_nil());
//! ```

use core::borrow::Borrow;
use core::fmt;
use core::marker::PhantomData;
use core::str::FromStr;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::domain::UuidDomain;
use crate::error::UuidParseError;

// ============================================================================
// CORE UUID IMPLEMENTATION
// ============================================================================

/// Type-safe UUID identifier parameterized by domain
///
/// `Uuid<D>` wraps a [`uuid::Uuid`] with a phantom domain marker, providing
/// compile-time type safety for UUID-based identifiers. It is `Copy`, making
/// it efficient to pass around.
///
/// # Domain Trait
///
/// `Uuid<D>` uses [`UuidDomain`] — a lightweight marker trait with only a domain name.
/// No string validation or normalization needed, unlike [`KeyDomain`](crate::KeyDomain).
///
/// # Memory Layout
///
/// ```text
/// Uuid<D> struct (16 bytes):
/// ┌────────────────┬─────────────┐
/// │ uuid::Uuid(16B)│ marker (0B) │
/// └────────────────┴─────────────┘
/// ```
///
/// # Type Safety
///
/// Different domains produce incompatible UUID types at compile time:
///
/// ```rust,compile_fail
/// use domain_key::{Uuid, Domain, UuidDomain};
///
/// #[derive(Debug)]
/// struct UserDomain;
/// impl Domain for UserDomain { const DOMAIN_NAME: &'static str = "user"; }
/// impl UuidDomain for UserDomain {}
///
/// #[derive(Debug)]
/// struct OrderDomain;
/// impl Domain for OrderDomain { const DOMAIN_NAME: &'static str = "order"; }
/// impl UuidDomain for OrderDomain {}
///
/// type UserUuid = Uuid<UserDomain>;
/// type OrderUuid = Uuid<OrderDomain>;
///
/// let user_uuid = UserUuid::nil();
/// let order_uuid: OrderUuid = user_uuid; // Compile error!
/// ```
pub struct Uuid<D: UuidDomain> {
    inner: ::uuid::Uuid,
    _marker: PhantomData<D>,
}

// Manual trait impls to avoid requiring D: Copy/Clone/PartialEq/Hash/etc.
// The phantom marker D is a ZST and never contributes to these operations.

impl<D: UuidDomain> fmt::Debug for Uuid<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", D::DOMAIN_NAME, self.inner)
    }
}

impl<D: UuidDomain> Copy for Uuid<D> {}

impl<D: UuidDomain> Clone for Uuid<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: UuidDomain> PartialEq for Uuid<D> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<D: UuidDomain> Eq for Uuid<D> {}

impl<D: UuidDomain> PartialOrd for Uuid<D> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<D: UuidDomain> Ord for Uuid<D> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<D: UuidDomain> core::hash::Hash for Uuid<D> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<D: UuidDomain> Uuid<D> {
    /// Wraps an existing [`uuid::Uuid`] into a typed identifier.
    ///
    /// Prefer using the [`From`] implementation or `.into()` for better
    /// ergonomics:
    ///
    /// ```rust
    /// use domain_key::{Uuid, Domain, UuidDomain};
    ///
    /// #[derive(Debug)]
    /// struct UserDomain;
    /// impl Domain for UserDomain { const DOMAIN_NAME: &'static str = "user"; }
    /// impl UuidDomain for UserDomain {}
    ///
    /// type UserUuid = Uuid<UserDomain>;
    ///
    /// let raw = uuid::Uuid::nil();
    /// let typed = UserUuid::from(raw);
    /// assert_eq!(typed.get(), raw);
    /// ```
    ///
    /// Parses a UUID string into a typed identifier.
    ///
    /// Accepts any format supported by [`uuid::Uuid::parse_str`]:
    /// `550e8400-e29b-41d4-a716-446655440000`, `550e8400e29b41d4a716446655440000`, etc.
    ///
    /// # Errors
    ///
    /// Returns [`UuidParseError`] if the string is not a valid UUID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{Uuid, Domain, UuidDomain};
    ///
    /// #[derive(Debug)]
    /// struct UserDomain;
    /// impl Domain for UserDomain { const DOMAIN_NAME: &'static str = "user"; }
    /// impl UuidDomain for UserDomain {}
    ///
    /// type UserUuid = Uuid<UserDomain>;
    ///
    /// let id = UserUuid::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
    /// assert!(!id.is_nil());
    /// ```
    pub fn parse(s: &str) -> Result<Self, UuidParseError> {
        s.parse()
    }

    /// Compares this typed UUID with a string without allocating.
    ///
    /// Returns `true` when `s` parses as a UUID equal to this value.
    #[inline]
    #[must_use]
    pub fn eq_str(&self, s: &str) -> bool {
        ::uuid::Uuid::parse_str(s)
            .map(|parsed| parsed == self.inner)
            .unwrap_or(false)
    }

    /// Creates a nil (all zeros) UUID.
    #[inline]
    #[must_use]
    pub const fn nil() -> Self {
        Self {
            inner: ::uuid::Uuid::nil(),
            _marker: PhantomData,
        }
    }

    /// Creates a UUID from a 16-byte array.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            inner: ::uuid::Uuid::from_bytes(bytes),
            _marker: PhantomData,
        }
    }

    /// Returns `true` if this UUID is nil (all zeros).
    #[inline]
    #[must_use]
    pub fn is_nil(&self) -> bool {
        self.inner.is_nil()
    }

    /// Returns the inner [`uuid::Uuid`].
    #[inline]
    #[must_use]
    pub const fn get(&self) -> ::uuid::Uuid {
        self.inner
    }

    /// Returns the UUID as a 16-byte array.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.inner.as_bytes()
    }

    /// Returns the domain name for this identifier type.
    #[inline]
    #[must_use]
    pub fn domain(&self) -> &'static str {
        D::DOMAIN_NAME
    }

    /// Generates a new random UUID.
    ///
    /// This is the primary constructor for random UUID identifiers.
    ///
    /// Behavior depends on enabled features:
    /// - with `uuid-v7`, generates a time-ordered v7 UUID
    /// - otherwise (with `uuid-v4` only), generates a random v4 UUID
    ///
    /// Requires at least one of `uuid-v4` or `uuid-v7`.
    #[cfg(any(feature = "uuid-v4", feature = "uuid-v7"))]
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Self::generate_inner(),
            _marker: PhantomData,
        }
    }

    #[cfg(feature = "uuid-v7")]
    #[inline]
    fn generate_inner() -> ::uuid::Uuid {
        ::uuid::Uuid::now_v7()
    }

    #[cfg(all(feature = "uuid-v4", not(feature = "uuid-v7")))]
    #[inline]
    fn generate_inner() -> ::uuid::Uuid {
        ::uuid::Uuid::new_v4()
    }

    /// Generates a random UUID v4.
    ///
    /// Requires the `uuid-v4` feature.
    ///
    /// # Deprecated
    ///
    /// Use [`Uuid::<D>::new()`] instead.
    #[cfg(feature = "uuid-v4")]
    #[deprecated(
        since = "0.3.0",
        note = "Use Uuid::<D>::new() instead of Uuid::<D>::v4()"
    )]
    #[inline]
    #[must_use]
    pub fn v4() -> Self {
        Self {
            inner: ::uuid::Uuid::new_v4(),
            _marker: PhantomData,
        }
    }

    /// Generates a time-ordered UUID v7.
    ///
    /// Requires the `uuid-v7` feature.
    #[cfg(feature = "uuid-v7")]
    #[inline]
    #[must_use]
    pub fn now_v7() -> Self {
        Self {
            inner: ::uuid::Uuid::now_v7(),
            _marker: PhantomData,
        }
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS
// ============================================================================

impl<D: UuidDomain> Default for Uuid<D> {
    /// Returns a nil UUID (all zeros).
    fn default() -> Self {
        Self::nil()
    }
}

impl<D: UuidDomain> fmt::Display for Uuid<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<D: UuidDomain> FromStr for Uuid<D> {
    type Err = UuidParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = ::uuid::Uuid::parse_str(s)?;
        Ok(Self {
            inner: uuid,
            _marker: PhantomData,
        })
    }
}

impl<D: UuidDomain> From<::uuid::Uuid> for Uuid<D> {
    #[inline]
    fn from(uuid: ::uuid::Uuid) -> Self {
        Self {
            inner: uuid,
            _marker: PhantomData,
        }
    }
}

impl<D: UuidDomain> From<Uuid<D>> for ::uuid::Uuid {
    #[inline]
    fn from(typed: Uuid<D>) -> Self {
        typed.inner
    }
}

impl<D: UuidDomain> From<Uuid<D>> for [u8; 16] {
    #[inline]
    fn from(typed: Uuid<D>) -> Self {
        *typed.as_bytes()
    }
}

impl<D: UuidDomain> From<[u8; 16]> for Uuid<D> {
    #[inline]
    fn from(bytes: [u8; 16]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl<D: UuidDomain> TryFrom<&str> for Uuid<D> {
    type Error = UuidParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl<D: UuidDomain> TryFrom<String> for Uuid<D> {
    type Error = UuidParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl<D: UuidDomain> TryFrom<&[u8]> for Uuid<D> {
    type Error = UuidParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let inner = ::uuid::Uuid::from_slice(bytes)?;
        Ok(Self {
            inner,
            _marker: PhantomData,
        })
    }
}

impl<D: UuidDomain> AsRef<::uuid::Uuid> for Uuid<D> {
    fn as_ref(&self) -> &::uuid::Uuid {
        &self.inner
    }
}

impl<D: UuidDomain> AsRef<[u8; 16]> for Uuid<D> {
    #[inline]
    fn as_ref(&self) -> &[u8; 16] {
        self.as_bytes()
    }
}

impl<D: UuidDomain> Borrow<::uuid::Uuid> for Uuid<D> {
    #[inline]
    fn borrow(&self) -> &::uuid::Uuid {
        &self.inner
    }
}

impl<D: UuidDomain> PartialEq<str> for Uuid<D> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.eq_str(other)
    }
}

impl<D: UuidDomain> PartialEq<&str> for Uuid<D> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.eq_str(other)
    }
}

impl<D: UuidDomain> PartialEq<String> for Uuid<D> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.eq_str(other)
    }
}

impl<D: UuidDomain> PartialEq<Uuid<D>> for str {
    #[inline]
    fn eq(&self, other: &Uuid<D>) -> bool {
        other.eq_str(self)
    }
}

impl<D: UuidDomain> PartialEq<Uuid<D>> for &str {
    #[inline]
    fn eq(&self, other: &Uuid<D>) -> bool {
        other.eq_str(self)
    }
}

impl<D: UuidDomain> PartialEq<Uuid<D>> for String {
    #[inline]
    fn eq(&self, other: &Uuid<D>) -> bool {
        other.eq_str(self)
    }
}

impl<D: UuidDomain> PartialEq<::uuid::Uuid> for Uuid<D> {
    #[inline]
    fn eq(&self, other: &::uuid::Uuid) -> bool {
        self.inner == *other
    }
}

impl<D: UuidDomain> PartialEq<Uuid<D>> for ::uuid::Uuid {
    #[inline]
    fn eq(&self, other: &Uuid<D>) -> bool {
        *self == other.inner
    }
}

impl<D: UuidDomain> PartialEq<[u8; 16]> for Uuid<D> {
    #[inline]
    fn eq(&self, other: &[u8; 16]) -> bool {
        self.as_bytes() == other
    }
}

impl<D: UuidDomain> PartialEq<&[u8; 16]> for Uuid<D> {
    #[inline]
    fn eq(&self, other: &&[u8; 16]) -> bool {
        self.as_bytes() == *other
    }
}

impl<D: UuidDomain> PartialEq<Uuid<D>> for [u8; 16] {
    #[inline]
    fn eq(&self, other: &Uuid<D>) -> bool {
        self == other.as_bytes()
    }
}

impl<D: UuidDomain> PartialEq<Uuid<D>> for &[u8; 16] {
    #[inline]
    fn eq(&self, other: &Uuid<D>) -> bool {
        *self == other.as_bytes()
    }
}

// ============================================================================
// SERDE SUPPORT
// ============================================================================

#[cfg(feature = "serde")]
impl<D: UuidDomain> Serialize for Uuid<D> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Delegate to uuid's own Serialize — zero-alloc, uses a stack buffer
        self.inner.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, D: UuidDomain> Deserialize<'de> for Uuid<D> {
    fn deserialize<De: serde::Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        // Delegate to uuid's own Deserialize — handles all formats correctly
        let inner = ::uuid::Uuid::deserialize(deserializer)?;
        Ok(Self {
            inner,
            _marker: PhantomData,
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::{format, string::ToString};

    #[derive(Debug)]
    struct TestDomain;
    impl crate::Domain for TestDomain {
        const DOMAIN_NAME: &'static str = "test";
    }
    impl UuidDomain for TestDomain {}

    type TestUuid = Uuid<TestDomain>;

    const SAMPLE: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn test_nil() {
        let id = TestUuid::nil();
        assert!(id.is_nil());
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_default_is_nil() {
        let id = TestUuid::default();
        assert!(id.is_nil());
    }

    #[test]
    fn test_from_uuid() {
        let raw = ::uuid::Uuid::nil();
        let typed = TestUuid::from(raw);
        assert_eq!(typed.get(), raw);
    }

    #[test]
    fn test_parse() {
        let id = TestUuid::parse(SAMPLE).unwrap();
        assert!(!id.is_nil());
    }

    #[test]
    fn test_parse_invalid() {
        assert!(TestUuid::parse("not-a-uuid").is_err());
    }

    #[test]
    fn test_from_bytes() {
        let bytes = [1u8; 16];
        let id = TestUuid::from_bytes(bytes);
        assert_eq!(id.as_bytes(), &bytes);
    }

    #[test]
    fn test_debug_format() {
        let id = TestUuid::nil();
        assert_eq!(
            format!("{id:?}"),
            "test(00000000-0000-0000-0000-000000000000)"
        );
    }

    #[test]
    fn test_domain() {
        let id = TestUuid::nil();
        assert_eq!(id.domain(), "test");
    }

    #[test]
    fn test_display() {
        let id = TestUuid::nil();
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_from_str() {
        let id: TestUuid = SAMPLE.parse().unwrap();
        assert!(!id.is_nil());
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<TestUuid, _> = "not-a-uuid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_into_uuid() {
        let raw = ::uuid::Uuid::nil();
        let typed: TestUuid = raw.into();
        let back: ::uuid::Uuid = typed.into();
        assert_eq!(raw, back);
    }

    #[test]
    fn test_from_byte_array() {
        let bytes = [42u8; 16];
        let id: TestUuid = bytes.into();
        assert_eq!(id.as_bytes(), &bytes);
    }

    #[test]
    fn test_try_from_slice_bytes() {
        let bytes = [7u8; 16];
        let id = TestUuid::try_from(bytes.as_slice()).unwrap();
        assert_eq!(id.as_bytes(), &bytes);
    }

    #[test]
    fn test_try_from_slice_invalid_length() {
        let bytes = [0u8; 15];
        let result: Result<TestUuid, _> = TestUuid::try_from(bytes.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_str() {
        let id = TestUuid::try_from(SAMPLE).unwrap();
        assert!(!id.is_nil());
    }

    #[test]
    fn test_try_from_string() {
        let id = TestUuid::try_from(String::from(SAMPLE)).unwrap();
        assert!(!id.is_nil());
    }

    #[test]
    fn test_copy() {
        let id1 = TestUuid::nil();
        let id2 = id1; // Copy
        assert_eq!(id1, id2); // id1 still valid
    }

    #[test]
    fn test_ord() {
        let a = TestUuid::nil();
        let b: TestUuid = SAMPLE.parse().unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_as_ref() {
        let id = TestUuid::nil();
        let uuid_ref: &::uuid::Uuid = id.as_ref();
        assert!(uuid_ref.is_nil());
    }

    #[test]
    fn test_eq_str() {
        let id = TestUuid::parse(SAMPLE).unwrap();
        assert!(id.eq_str(SAMPLE));
        assert!(id.eq_str(&SAMPLE.to_ascii_uppercase()));
        assert!(!id.eq_str("not-a-uuid"));
    }

    #[test]
    fn test_partial_eq_str_and_string() {
        let id = TestUuid::parse(SAMPLE).unwrap();
        let owned = String::from(SAMPLE);
        assert!(id == SAMPLE);
        assert!(id == owned);
        assert!(id != "not-a-uuid");
    }

    #[test]
    fn test_partial_eq_str_and_string_symmetric() {
        let id = TestUuid::parse(SAMPLE).unwrap();
        let owned = String::from(SAMPLE);
        assert!(SAMPLE == id);
        assert!(owned == id);
    }

    #[test]
    fn test_partial_eq_raw_uuid_symmetric() {
        let typed = TestUuid::parse(SAMPLE).unwrap();
        let raw = ::uuid::Uuid::parse_str(SAMPLE).unwrap();
        assert!(typed == raw);
        assert!(raw == typed);
    }

    #[test]
    fn test_from_typed_uuid_to_bytes() {
        let typed = TestUuid::parse(SAMPLE).unwrap();
        let bytes: [u8; 16] = typed.into();
        assert_eq!(bytes, *::uuid::Uuid::parse_str(SAMPLE).unwrap().as_bytes());
    }

    #[test]
    fn test_as_ref_bytes() {
        let typed = TestUuid::parse(SAMPLE).unwrap();
        let bytes_ref: &[u8; 16] = typed.as_ref();
        assert_eq!(bytes_ref, typed.as_bytes());
    }

    #[test]
    fn test_partial_eq_bytes_symmetric() {
        let typed = TestUuid::parse(SAMPLE).unwrap();
        let bytes = *typed.as_bytes();
        assert!(typed == bytes);
        assert!(<TestUuid as PartialEq<&[u8; 16]>>::eq(&typed, &&bytes));
        assert!(bytes == typed);
        assert!(<&[u8; 16] as PartialEq<TestUuid>>::eq(&&bytes, &typed));
    }

    #[cfg(all(feature = "uuid-v4", not(feature = "uuid-v7")))]
    #[test]
    fn test_new_v4() {
        let id = TestUuid::new();
        assert!(!id.is_nil());
        assert_eq!(id.get().get_version(), Some(::uuid::Version::Random));
    }

    #[cfg(feature = "uuid-v7")]
    #[test]
    fn test_new_v7() {
        let id = TestUuid::new();
        assert!(!id.is_nil());
        assert_eq!(id.get().get_version(), Some(::uuid::Version::SortRand));
    }

    #[cfg(feature = "uuid-v4")]
    #[test]
    #[allow(deprecated)] // intentional: cover deprecated `v4()` shim until removal
    fn test_v4_deprecated_alias() {
        let id = TestUuid::v4();
        assert!(!id.is_nil());
        assert_eq!(id.get().get_version(), Some(::uuid::Version::Random));
    }

    #[cfg(feature = "uuid-v7")]
    #[test]
    fn test_v7() {
        let id = TestUuid::now_v7();
        assert!(!id.is_nil());
        assert_eq!(id.get().get_version(), Some(::uuid::Version::SortRand));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let id = TestUuid::parse(SAMPLE).unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert!(json.contains("550e8400"));
        let deserialized: TestUuid = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_type_safety_different_domains() {
        #[derive(Debug)]
        struct DomainA;
        impl crate::Domain for DomainA {
            const DOMAIN_NAME: &'static str = "a";
        }
        impl UuidDomain for DomainA {}

        #[derive(Debug)]
        struct DomainB;
        impl crate::Domain for DomainB {
            const DOMAIN_NAME: &'static str = "b";
        }
        impl UuidDomain for DomainB {}

        let _a: Uuid<DomainA> = Uuid::nil();
        let _b: Uuid<DomainB> = Uuid::nil();

        // These are different types — cannot be compared or assigned
        // _a == _b would not compile
    }
}
