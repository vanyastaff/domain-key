//! Typed ULID identifier for domain-key
//!
//! This module provides [`Ulid<D>`], a type-safe ULID wrapper with a domain-specific
//! string prefix (`{PREFIX}_{crockford}`), parameterized by [`UlidDomain`].
//!
//! # Feature flags
//!
//! - `ulid` — enables [`Ulid<D>`], [`UlidDomain`], and [`UlidParseError`]
//! - `ulid-monotonic` — enables [`MonotonicUlidGenerator`] (monotonic [`ulid::Generator`])
//!
//! # Examples
//!
//! ```rust
//! use domain_key::{Domain, Ulid, UlidDomain};
//!
//! #[derive(Debug)]
//! struct OrderDomain;
//!
//! impl Domain for OrderDomain {
//!     const DOMAIN_NAME: &'static str = "order";
//! }
//!
//! impl UlidDomain for OrderDomain {
//!     const PREFIX: &'static str = "ord";
//! }
//!
//! type OrderUlid = Ulid<OrderDomain>;
//!
//! let id = OrderUlid::new();
//! let s = id.to_string();
//! assert!(s.starts_with("ord_"));
//! ```

use core::fmt;
use core::marker::PhantomData;
use core::str::FromStr;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::domain::UlidDomain;
use crate::error::UlidParseError;

/// Parses `"{PREFIX}_{ulid}"` into a raw [`ulid::Ulid`].
#[inline]
fn parse_prefixed<D: UlidDomain>(s: &str) -> Result<::ulid::Ulid, UlidParseError> {
    let prefix = D::PREFIX;
    let rest = s.strip_prefix(prefix).ok_or(UlidParseError::WrongPrefix {
        expected_prefix: prefix,
    })?;
    let body = rest.strip_prefix('_').ok_or(UlidParseError::WrongPrefix {
        expected_prefix: prefix,
    })?;
    ::ulid::Ulid::from_string(body).map_err(UlidParseError::InvalidUlid)
}

/// Type-safe ULID identifier with a domain-specific prefix
///
/// [`core::fmt::Display`] and JSON serialization use the form `{PREFIX}_{crockford}` (for example
/// `exe_01J9ABCDEFGHJKMNPQRSTVWXYZ`). The inner value is a [`ulid::Ulid`] (`Copy`, 16 bytes).
///
/// # Type safety
///
/// Different domains produce incompatible types at compile time, same as [`crate::Uuid`].
pub struct Ulid<D: UlidDomain> {
    inner: ::ulid::Ulid,
    _marker: PhantomData<D>,
}

impl<D: UlidDomain> fmt::Debug for Ulid<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", D::DOMAIN_NAME, self)
    }
}

impl<D: UlidDomain> Copy for Ulid<D> {}

impl<D: UlidDomain> Clone for Ulid<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: UlidDomain> PartialEq for Ulid<D> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<D: UlidDomain> Eq for Ulid<D> {}

impl<D: UlidDomain> PartialOrd for Ulid<D> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<D: UlidDomain> Ord for Ulid<D> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<D: UlidDomain> core::hash::Hash for Ulid<D> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<D: UlidDomain> Ulid<D> {
    /// Wraps an existing [`ulid::Ulid`].
    #[inline]
    #[must_use]
    pub const fn from_ulid(ulid: ::ulid::Ulid) -> Self {
        Self {
            inner: ulid,
            _marker: PhantomData,
        }
    }

    /// Parses a prefixed string `"{PREFIX}_{crockford}"`.
    ///
    /// # Errors
    ///
    /// Returns [`UlidParseError`] if the prefix does not match or the body is not a valid ULID.
    pub fn parse(s: &str) -> Result<Self, UlidParseError> {
        s.parse()
    }

    /// The nil ULID (all zero bits).
    #[inline]
    #[must_use]
    pub const fn nil() -> Self {
        Self {
            inner: ::ulid::Ulid::nil(),
            _marker: PhantomData,
        }
    }

    /// Creates a ULID from a 16-byte big-endian representation.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            inner: ::ulid::Ulid::from_bytes(bytes),
            _marker: PhantomData,
        }
    }

    /// Returns `true` if this is the nil ULID.
    #[inline]
    #[must_use]
    pub fn is_nil(&self) -> bool {
        self.inner.is_nil()
    }

    /// Raw [`ulid::Ulid`] value.
    #[inline]
    #[must_use]
    pub const fn get(&self) -> ::ulid::Ulid {
        self.inner
    }

    /// Big-endian bytes of the underlying ULID.
    #[inline]
    #[must_use]
    pub const fn to_bytes(&self) -> [u8; 16] {
        self.inner.to_bytes()
    }

    /// [`DOMAIN_NAME`](crate::Domain::DOMAIN_NAME) for this identifier type.
    #[inline]
    #[must_use]
    pub fn domain(&self) -> &'static str {
        D::DOMAIN_NAME
    }

    /// UTC timestamp from the ULID's 48-bit millisecond field.
    ///
    /// Uses the ULID timestamp only (same millisecond resolution as the spec). If the millisecond
    /// value cannot be represented as a `chrono` [`DateTime`](chrono::DateTime) (extremely unlikely
    /// for valid ULIDs), this returns [`DateTime::UNIX_EPOCH`](chrono::DateTime::UNIX_EPOCH).
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        let ms = self.inner.timestamp_ms();
        let Ok(ms_i64) = i64::try_from(ms) else {
            return chrono::DateTime::<chrono::Utc>::UNIX_EPOCH;
        };
        chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms_i64)
            .unwrap_or(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH)
    }

    /// Generates a new ULID using the current time (not monotonic within the same millisecond).
    ///
    /// For monotonic ordering within a process, enable `ulid-monotonic` and use
    /// [`MonotonicUlidGenerator`].
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ::ulid::Ulid::new(),
            _marker: PhantomData,
        }
    }
}

impl<D: UlidDomain> Default for Ulid<D> {
    fn default() -> Self {
        Self::nil()
    }
}

impl<D: UlidDomain> fmt::Display for Ulid<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", D::PREFIX, self.inner)
    }
}

impl<D: UlidDomain> FromStr for Ulid<D> {
    type Err = UlidParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = parse_prefixed::<D>(s)?;
        Ok(Self {
            inner,
            _marker: PhantomData,
        })
    }
}

impl<D: UlidDomain> From<::ulid::Ulid> for Ulid<D> {
    #[inline]
    fn from(ulid: ::ulid::Ulid) -> Self {
        Self::from_ulid(ulid)
    }
}

impl<D: UlidDomain> From<Ulid<D>> for ::ulid::Ulid {
    #[inline]
    fn from(typed: Ulid<D>) -> Self {
        typed.inner
    }
}

impl<D: UlidDomain> From<[u8; 16]> for Ulid<D> {
    #[inline]
    fn from(bytes: [u8; 16]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl<D: UlidDomain> TryFrom<&str> for Ulid<D> {
    type Error = UlidParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl<D: UlidDomain> TryFrom<String> for Ulid<D> {
    type Error = UlidParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl<D: UlidDomain> TryFrom<&[u8]> for Ulid<D> {
    type Error = UlidParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 16 {
            return Err(UlidParseError::InvalidUlid(
                ::ulid::DecodeError::InvalidLength,
            ));
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes);
        Ok(Self::from_bytes(arr))
    }
}

impl<D: UlidDomain> AsRef<::ulid::Ulid> for Ulid<D> {
    fn as_ref(&self) -> &::ulid::Ulid {
        &self.inner
    }
}

#[cfg(feature = "serde")]
impl<D: UlidDomain> Serialize for Ulid<D> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de, D: UlidDomain> Deserialize<'de> for Ulid<D> {
    fn deserialize<De: serde::Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        struct UlidVisitor<D: UlidDomain>(PhantomData<D>);

        impl<D: UlidDomain> serde::de::Visitor<'_> for UlidVisitor<D> {
            type Value = Ulid<D>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a prefixed ULID string")
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                s.parse().map_err(E::custom)
            }

            fn visit_string<E: serde::de::Error>(self, s: String) -> Result<Self::Value, E> {
                self.visit_str(&s)
            }
        }

        deserializer.deserialize_str(UlidVisitor(PhantomData))
    }
}

/// Monotonic ULID generator scoped to [`UlidDomain`] `D`.
///
/// Wraps [`ulid::Generator`]: each [`MonotonicUlidGenerator::generate`] yields a strictly
/// increasing [`Ulid<D>`] (by raw ULID ordering).
///
/// Requires the `ulid-monotonic` feature.
#[cfg(feature = "ulid-monotonic")]
pub struct MonotonicUlidGenerator<D: UlidDomain> {
    inner: ::ulid::Generator,
    _marker: PhantomData<D>,
}

#[cfg(feature = "ulid-monotonic")]
impl<D: UlidDomain> MonotonicUlidGenerator<D> {
    /// Creates a new generator (state starts at nil; first value uses current time).
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: ::ulid::Generator::new(),
            _marker: PhantomData,
        }
    }

    /// Generates the next monotonic ULID for the current time.
    ///
    /// # Errors
    ///
    /// Returns [`UlidMonotonicError`](crate::UlidMonotonicError) if incrementing within the same
    /// millisecond would overflow the 80 random bits.
    #[inline]
    pub fn generate(&mut self) -> Result<Ulid<D>, ::ulid::MonotonicError> {
        let inner = self.inner.generate()?;
        Ok(Ulid {
            inner,
            _marker: PhantomData,
        })
    }

    /// Generates the next monotonic ULID for a specific [`std::time::SystemTime`].
    ///
    /// # Errors
    ///
    /// Same as [`MonotonicUlidGenerator::generate`].
    #[inline]
    pub fn generate_from_datetime(
        &mut self,
        datetime: std::time::SystemTime,
    ) -> Result<Ulid<D>, ::ulid::MonotonicError> {
        let inner = self.inner.generate_from_datetime(datetime)?;
        Ok(Ulid {
            inner,
            _marker: PhantomData,
        })
    }
}

#[cfg(feature = "ulid-monotonic")]
impl<D: UlidDomain> Default for MonotonicUlidGenerator<D> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::format;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    #[derive(Debug)]
    struct TestDomain;
    impl crate::Domain for TestDomain {
        const DOMAIN_NAME: &'static str = "test";
    }
    impl UlidDomain for TestDomain {
        const PREFIX: &'static str = "tst";
    }

    type TestUlid = Ulid<TestDomain>;

    const SAMPLE_BODY: &str = "01D39ZY06FGSCTVN4T2V9PKHFZ";

    fn sample_prefixed() -> String {
        format!("tst_{SAMPLE_BODY}")
    }

    #[test]
    fn test_nil() {
        let id = TestUlid::nil();
        assert!(id.is_nil());
        assert_eq!(id.to_string(), "tst_00000000000000000000000000");
    }

    #[test]
    fn test_default_is_nil() {
        let id = TestUlid::default();
        assert!(id.is_nil());
    }

    #[test]
    fn test_from_ulid() {
        let raw = ::ulid::Ulid::nil();
        let typed = TestUlid::from(raw);
        assert_eq!(typed.get(), raw);
    }

    #[test]
    fn test_parse_roundtrip() {
        let s = sample_prefixed();
        let id = TestUlid::parse(&s).unwrap();
        assert!(!id.is_nil());
        assert_eq!(id.to_string(), s);
    }

    #[test]
    fn test_parse_wrong_prefix() {
        let err = TestUlid::parse("xxx_01D39ZY06FGSCTVN4T2V9PKHFZ").unwrap_err();
        assert_eq!(
            err,
            UlidParseError::WrongPrefix {
                expected_prefix: "tst"
            }
        );
    }

    #[test]
    fn test_parse_missing_separator() {
        let err = TestUlid::parse("tst01D39ZY06FGSCTVN4T2V9PKHFZ").unwrap_err();
        assert!(matches!(err, UlidParseError::WrongPrefix { .. }));
    }

    #[test]
    fn test_parse_invalid_body() {
        let err = TestUlid::parse("tst_!!!").unwrap_err();
        assert!(matches!(err, UlidParseError::InvalidUlid(_)));
    }

    #[test]
    fn test_from_bytes() {
        let bytes = [1u8; 16];
        let id = TestUlid::from_bytes(bytes);
        assert_eq!(id.to_bytes(), bytes);
    }

    #[test]
    fn test_debug_format() {
        let id = TestUlid::nil();
        let dbg = format!("{id:?}");
        assert!(dbg.contains("test("));
        assert!(dbg.contains("tst_"));
    }

    #[test]
    fn test_domain() {
        let id = TestUlid::nil();
        assert_eq!(id.domain(), "test");
    }

    #[test]
    fn test_display() {
        let id = TestUlid::parse(&sample_prefixed()).unwrap();
        assert_eq!(id.to_string(), sample_prefixed());
    }

    #[test]
    fn test_from_str() {
        let id: TestUlid = sample_prefixed().parse().unwrap();
        assert!(!id.is_nil());
    }

    #[test]
    fn test_try_from_bytes_slice() {
        let u = ::ulid::Ulid::from_string(SAMPLE_BODY).unwrap();
        let bytes: [u8; 16] = u.to_bytes();
        let id = TestUlid::try_from(bytes.as_slice()).unwrap();
        assert_eq!(id.get(), u);
    }

    #[test]
    fn test_try_from_slice_invalid_length() {
        let bytes = [0u8; 15];
        let result: Result<TestUlid, _> = TestUlid::try_from(bytes.as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn test_ord() {
        let a = TestUlid::nil();
        let b = TestUlid::parse(&sample_prefixed()).unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_new() {
        let id = TestUlid::new();
        assert!(!id.is_nil());
    }

    #[test]
    fn test_created_at_matches_inner_timestamp_ms() {
        let id = TestUlid::new();
        let ms = id.get().timestamp_ms();
        assert_eq!(
            id.created_at().timestamp_millis(),
            i64::try_from(ms).expect("ULID timestamp fits in i64")
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let id = TestUlid::parse(&sample_prefixed()).unwrap();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: TestUlid = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[cfg(feature = "ulid-monotonic")]
    #[test]
    fn test_monotonic_generator() {
        let mut gen = MonotonicUlidGenerator::<TestDomain>::new();
        let a = gen.generate().unwrap();
        let b = gen.generate().unwrap();
        assert!(a < b);
    }
}
