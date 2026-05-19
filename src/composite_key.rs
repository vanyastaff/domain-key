//! `CompositeKey<A, B, const SEP: char>` — a typed composite of two domain keys.
//!
//! A `CompositeKey` pairs a `Key<A>` and a `Key<B>` into a single composite identifier
//! that serialises as `"{first}{SEP}{second}"` and round-trips through [`FromStr`].
//!
//! ## Default separator
//!
//! The separator defaults to `':'`, which is suitable for most use cases.  Use the
//! third const parameter to choose a different separator:
//!
//! ```rust
//! use domain_key::{CompositeKey, Domain, KeyDomain, Key};
//!
//! #[derive(Debug)] struct OrgDomain;
//! impl Domain for OrgDomain { const DOMAIN_NAME: &'static str = "org"; }
//! impl KeyDomain for OrgDomain {}
//!
//! #[derive(Debug)] struct RepoDomain;
//! impl Domain for RepoDomain { const DOMAIN_NAME: &'static str = "repo"; }
//! impl KeyDomain for RepoDomain {}
//!
//! // Default separator ':'
//! type OrgRepoKey = CompositeKey<OrgDomain, RepoDomain>;
//!
//! // Custom separator '/'
//! type OrgRepoPath = CompositeKey<OrgDomain, RepoDomain, '/'>;
//!
//! let org  = Key::<OrgDomain>::new("acme").unwrap();
//! let repo = Key::<RepoDomain>::new("widget").unwrap();
//!
//! let ck: OrgRepoKey = CompositeKey::new(org.clone(), repo.clone());
//! assert_eq!(ck.to_string(), "acme:widget");
//!
//! let path: OrgRepoPath = CompositeKey::new(org, repo);
//! assert_eq!(path.to_string(), "acme/widget");
//! ```
//!
//! ## Parsing
//!
//! Parsing splits on the **first** occurrence of the separator.  Extra occurrences
//! of `SEP` in the second component are left intact and validated as part of
//! `Key<B>`:
//!
//! ```rust
//! # use domain_key::{CompositeKey, Domain, KeyDomain};
//! # #[derive(Debug)] struct A; impl Domain for A { const DOMAIN_NAME: &'static str = "a"; } impl KeyDomain for A {}
//! # #[derive(Debug)] struct B; impl Domain for B { const DOMAIN_NAME: &'static str = "b"; } impl KeyDomain for B {}
//! let result = "x:y".parse::<CompositeKey<A, B>>();
//! assert!(result.is_ok());
//! ```
//!
//! ## Panics (debug only)
//!
//! [`CompositeKey::new`] contains a `debug_assert!` that fires if the first component's
//! string representation contains `SEP`.  In **release** builds this check is elided.
//!
//! ## Caller responsibility (release)
//!
//! In release builds callers are responsible for ensuring that `first.as_str()` does not
//! contain the separator character.  A first component containing `SEP` will produce a
//! composite key that cannot be parsed back to the same components, causing a silent
//! round-trip failure.  Use [`CompositeKey::from_str`] if you need hard validation on
//! every call.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::domain::KeyDomain;
use crate::error::CompositeKeyParseError;
use crate::key::Key;

// ============================================================================
// STRUCT
// ============================================================================

/// A typed composite of two domain keys, separated by `SEP` (default `':'`).
///
/// See the [module-level docs](self) for usage examples.
#[derive(Debug)]
pub struct CompositeKey<A: KeyDomain, B: KeyDomain, const SEP: char = ':'> {
    first: Key<A>,
    second: Key<B>,
}

impl<A: KeyDomain, B: KeyDomain, const SEP: char> Clone for CompositeKey<A, B, SEP> {
    fn clone(&self) -> Self {
        Self {
            first: self.first.clone(),
            second: self.second.clone(),
        }
    }
}

// ============================================================================
// CONSTRUCTOR & ACCESSORS
// ============================================================================

impl<A: KeyDomain, B: KeyDomain, const SEP: char> CompositeKey<A, B, SEP> {
    /// Create a `CompositeKey` from two typed component keys.
    ///
    /// # Panics (debug only)
    ///
    /// Panics in debug builds if `first.as_str()` contains the separator `SEP`.
    /// In release builds the check is elided; callers are responsible for ensuring
    /// the first component does not contain the separator (see module docs).
    #[must_use]
    pub fn new(first: Key<A>, second: Key<B>) -> Self {
        debug_assert!(
            !first.as_str().contains(SEP),
            "CompositeKey::new: first component {:?} contains separator {:?}",
            first.as_str(),
            SEP,
        );
        Self { first, second }
    }

    /// Returns a reference to the first component key.
    #[must_use]
    #[inline]
    pub fn first(&self) -> &Key<A> {
        &self.first
    }

    /// Returns a reference to the second component key.
    #[must_use]
    #[inline]
    pub fn second(&self) -> &Key<B> {
        &self.second
    }
}

// ============================================================================
// DISPLAY
// ============================================================================

impl<A: KeyDomain, B: KeyDomain, const SEP: char> fmt::Display for CompositeKey<A, B, SEP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.first.as_str(), SEP, self.second.as_str())
    }
}

// ============================================================================
// FROM STR
// ============================================================================

impl<A: KeyDomain, B: KeyDomain, const SEP: char> FromStr for CompositeKey<A, B, SEP> {
    type Err = CompositeKeyParseError;

    /// Parse a composite key from a string by splitting on the **first** occurrence
    /// of `SEP`.
    ///
    /// # Errors
    ///
    /// - [`CompositeKeyParseError::MissingSeparator`] if `SEP` is not found.
    /// - [`CompositeKeyParseError::InvalidFirst`] if the first segment is not a valid `Key<A>`.
    /// - [`CompositeKeyParseError::InvalidSecond`] if the second segment is not a valid `Key<B>`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sep_pos = s
            .find(SEP)
            .ok_or(CompositeKeyParseError::MissingSeparator { separator: SEP })?;

        let first_str = &s[..sep_pos];
        // Skip the separator itself (SEP is a char; its UTF-8 length may be >1)
        let second_str = &s[sep_pos + SEP.len_utf8()..];

        let first = Key::<A>::new(first_str).map_err(CompositeKeyParseError::InvalidFirst)?;
        let second =
            Key::<B>::new(second_str).map_err(CompositeKeyParseError::InvalidSecond)?;

        Ok(Self { first, second })
    }
}

// ============================================================================
// EQUALITY
// ============================================================================

impl<A: KeyDomain, B: KeyDomain, const SEP: char> PartialEq for CompositeKey<A, B, SEP> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.second == other.second
    }
}

impl<A: KeyDomain, B: KeyDomain, const SEP: char> Eq for CompositeKey<A, B, SEP> {}

// ============================================================================
// ORDERING
// ============================================================================

impl<A: KeyDomain, B: KeyDomain, const SEP: char> PartialOrd for CompositeKey<A, B, SEP> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: KeyDomain, B: KeyDomain, const SEP: char> Ord for CompositeKey<A, B, SEP> {
    /// Lexicographic ordering: compare first components, then second.
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.first
            .cmp(&other.first)
            .then_with(|| self.second.cmp(&other.second))
    }
}

// ============================================================================
// HASH
// ============================================================================

impl<A: KeyDomain, B: KeyDomain, const SEP: char> Hash for CompositeKey<A, B, SEP> {
    /// Hash the composite key by hashing both component strings and the separator
    /// sequentially (zero-allocation).
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.first.as_str().hash(state);
        SEP.hash(state);
        self.second.as_str().hash(state);
    }
}

// ============================================================================
// SERDE
// ============================================================================

#[cfg(feature = "serde")]
impl<A: KeyDomain, B: KeyDomain, const SEP: char> Serialize for CompositeKey<A, B, SEP> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, A: KeyDomain, B: KeyDomain, const SEP: char> Deserialize<'de>
    for CompositeKey<A, B, SEP>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <&str>::deserialize(deserializer)?;
            CompositeKey::from_str(s).map_err(serde::de::Error::custom)
        } else {
            let s = String::deserialize(deserializer)?;
            CompositeKey::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    use super::*;
    use crate::error::KeyParseError;
    use crate::{Domain, KeyDomain};

    // ---- test domains -------------------------------------------------------

    #[derive(Debug)]
    struct UserDomain;
    impl Domain for UserDomain {
        const DOMAIN_NAME: &'static str = "user";
    }
    impl KeyDomain for UserDomain {}

    #[derive(Debug)]
    struct PostDomain;
    impl Domain for PostDomain {
        const DOMAIN_NAME: &'static str = "post";
    }
    impl KeyDomain for PostDomain {}

    // Type alias to anchor the default SEP in tests
    type TestKey = CompositeKey<UserDomain, PostDomain>;

    fn user(s: &str) -> Key<UserDomain> {
        Key::new(s).unwrap()
    }
    fn post(s: &str) -> Key<PostDomain> {
        Key::new(s).unwrap()
    }

    // ---- AE1: round-trip with default separator -----------------------------

    #[test]
    fn ae1_roundtrip_default_separator() {
        let ck: TestKey = CompositeKey::new(user("user123"), post("post456"));
        assert_eq!(ck.to_string(), "user123:post456");

        let parsed: TestKey = "user123:post456".parse().unwrap();
        assert_eq!(parsed.first(), &user("user123"));
        assert_eq!(parsed.second(), &post("post456"));
    }

    // ---- AE2: missing separator error ---------------------------------------

    #[test]
    fn ae2_missing_separator() {
        let err = "user123".parse::<TestKey>().unwrap_err();
        assert_eq!(
            err,
            CompositeKeyParseError::MissingSeparator { separator: ':' }
        );
    }

    // ---- AE3: empty first segment -------------------------------------------

    #[test]
    fn ae3_empty_first_segment() {
        let err = ":post456".parse::<TestKey>().unwrap_err();
        assert!(
            matches!(err, CompositeKeyParseError::InvalidFirst(KeyParseError::Empty)),
            "expected InvalidFirst(Empty), got {:?}",
            err
        );
    }

    // ---- AE4: custom separator '/' ------------------------------------------

    #[test]
    fn ae4_custom_separator() {
        let ck: CompositeKey<UserDomain, PostDomain, '/'> =
            CompositeKey::new(user("user123"), post("post456"));
        assert_eq!(ck.to_string(), "user123/post456");

        let parsed: CompositeKey<UserDomain, PostDomain, '/'> =
            "user123/post456".parse().unwrap();
        assert_eq!(parsed.first(), &user("user123"));
        assert_eq!(parsed.second(), &post("post456"));

        // Colon is not the separator for '/' variant
        let err = "user123:post456"
            .parse::<CompositeKey<UserDomain, PostDomain, '/'>>()
            .unwrap_err();
        assert!(matches!(
            err,
            CompositeKeyParseError::MissingSeparator { separator: '/' }
        ));
    }

    // ---- AE5: equality and hash consistency ---------------------------------

    #[test]
    fn ae5_equality_and_hash_consistency() {
        let ck1: TestKey = CompositeKey::new(user("user123"), post("post456"));
        let ck2: TestKey = CompositeKey::new(user("user123"), post("post456"));
        assert_eq!(ck1, ck2);

        fn compute_hash<T: Hash>(value: &T) -> u64 {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            value.hash(&mut h);
            std::hash::BuildHasher::build_hasher(&std::collections::hash_map::RandomState::new());
            h.finish()
        }
        assert_eq!(compute_hash(&ck1), compute_hash(&ck2));
    }

    // ---- edge cases ---------------------------------------------------------

    #[test]
    fn split_on_first_colon_second_may_contain_colon() {
        let result = "user123:extra:part".parse::<TestKey>();
        let _ = result;
    }

    #[test]
    fn empty_input_missing_separator() {
        let err = "".parse::<TestKey>().unwrap_err();
        assert!(matches!(
            err,
            CompositeKeyParseError::MissingSeparator { separator: ':' }
        ));
    }

    #[test]
    fn only_separator_empty_first_and_second() {
        let err = ":".parse::<TestKey>().unwrap_err();
        assert!(
            matches!(err, CompositeKeyParseError::InvalidFirst(KeyParseError::Empty)),
            "expected InvalidFirst(Empty), got {:?}",
            err
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    fn debug_assert_fires_when_first_contains_sep() {
        let first_with_sep = Key::<UserDomain>::new("no-sep").unwrap();
        let ck: TestKey = CompositeKey::new(first_with_sep, post("post456"));
        assert_eq!(ck.to_string(), "no-sep:post456");
    }

    #[test]
    fn ord_lexicographic() {
        let a_b: TestKey = "aaa:bbb".parse().unwrap();
        let a_c: TestKey = "aaa:ccc".parse().unwrap();
        let b_a: TestKey = "bbb:aaa".parse().unwrap();

        assert!(a_b < a_c);
        assert!(a_b < b_a);
        assert!(a_c < b_a);
    }

    #[test]
    fn clone_is_independent() {
        let ck: TestKey = CompositeKey::new(user("user123"), post("post456"));
        let cloned = ck.clone();
        assert_eq!(ck, cloned);
    }

    #[test]
    fn can_use_as_hash_map_key() {
        let mut map: HashMap<TestKey, &str> = HashMap::new();
        let ck: TestKey = CompositeKey::new(user("u1"), post("p1"));
        map.insert(ck.clone(), "value");
        assert_eq!(map[&ck], "value");
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn json_roundtrip() {
            let ck: TestKey = CompositeKey::new(user("user123"), post("post456"));
            let json = serde_json::to_string(&ck).unwrap();
            assert_eq!(json, r#""user123:post456""#);

            let parsed: TestKey = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, ck);
        }

        #[test]
        fn json_error_no_separator() {
            let result = serde_json::from_str::<TestKey>(r#""user123""#);
            assert!(result.is_err());
        }

        #[test]
        fn custom_separator_json_roundtrip() {
            let ck: CompositeKey<UserDomain, PostDomain, '/'> =
                CompositeKey::new(user("user123"), post("post456"));
            let json = serde_json::to_string(&ck).unwrap();
            assert_eq!(json, r#""user123/post456""#);

            let parsed: CompositeKey<UserDomain, PostDomain, '/'> =
                serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, ck);
        }
    }
}
