//! [`arbitrary::Arbitrary`] impls for domain-key types.
//!
//! Enabled by the `arbitrary` feature flag. Provides structured fuzzing support
//! for [`Key<D>`](crate::Key), [`Id<D>`](crate::Id),
//! [`Uuid<D>`](crate::Uuid) (requires `uuid` feature), and
//! [`Ulid<D>`](crate::Ulid) (requires `ulid` feature).
//!
//! `Key<D>` generation is constructive: characters are assembled position-by-position
//! from the domain's [`KeyDomain`](crate::KeyDomain) predicates over the ASCII printable
//! range (`U+0020–U+007E`). Every generated `Key<D>` is valid by construction for
//! domains where `HAS_CUSTOM_VALIDATION = false`. For domains with
//! `HAS_CUSTOM_VALIDATION = true`, the impl draws uniformly from
//! [`KeyDomain::examples()`](crate::KeyDomain::examples) when examples are provided;
//! otherwise it falls back to the constructive path with best-effort validity.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::num::NonZeroU64;

use arbitrary::{Arbitrary, Unstructured};
use smartstring::alias::String as SmartString;

use crate::domain::{IdDomain, KeyDomain};
use crate::id::Id;
use crate::key::Key;

// ── Id<D> ────────────────────────────────────────────────────────────────────

impl<'a, D: IdDomain> Arbitrary<'a> for Id<D> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let raw: u64 = u.arbitrary()?;
        // Bit-OR with 1 guarantees non-zero without biasing the distribution
        // significantly (only 1/2^64 values are affected).
        Ok(Id::from_non_zero(
            NonZeroU64::new(raw | 1).expect("raw | 1 is always non-zero"),
        ))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        u64::size_hint(depth)
    }
}

// ── Uuid<D> ──────────────────────────────────────────────────────────────────

/// `arbitrary::Arbitrary` for [`Uuid<D>`](crate::Uuid).
///
/// Generates a uniformly random 16-byte UUID. All 128 bits are arbitrary;
/// the implementation does not force any particular UUID version.
///
/// Requires the `uuid` and `arbitrary` features.
#[cfg(feature = "uuid")]
impl<'a, D: crate::domain::UuidDomain> Arbitrary<'a> for crate::uuid::Uuid<D> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let bytes: [u8; 16] = u.arbitrary()?;
        Ok(Self::from_bytes(bytes))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <[u8; 16]>::size_hint(depth)
    }
}

// ── Ulid<D> ──────────────────────────────────────────────────────────────────

/// `arbitrary::Arbitrary` for [`Ulid<D>`](crate::Ulid).
///
/// Generates a uniformly random 16-byte ULID. All 128 bits are arbitrary;
/// the implementation does not enforce timestamp monotonicity.
///
/// Requires the `ulid` and `arbitrary` features.
#[cfg(feature = "ulid")]
impl<'a, D: crate::domain::UlidDomain> Arbitrary<'a> for crate::ulid::Ulid<D> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let bytes: [u8; 16] = u.arbitrary()?;
        Ok(Self::from_bytes(bytes))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <[u8; 16]>::size_hint(depth)
    }
}

// ── Key<D> ───────────────────────────────────────────────────────────────────

impl<'a, D: KeyDomain> Arbitrary<'a> for Key<D> {
    /// Generates an arbitrary valid [`Key<D>`] value.
    ///
    /// # Generation strategy
    ///
    /// - **`HAS_CUSTOM_VALIDATION = true` with examples:** draws uniformly from
    ///   [`KeyDomain::examples()`] (R8a). Examples are trusted to be valid by
    ///   the domain author's contract.
    /// - **Constructive path (all other cases):** assembles a key character-by-character
    ///   over the ASCII printable range (`' '..='~'`, U+0020–U+007E), respecting all
    ///   [`KeyDomain`] predicates at each position. Returns
    ///   [`arbitrary::Error::EmptyChoose`] if no valid key can be constructed.
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // R8a: for custom-validation domains with examples, draw from the examples pool.
        if D::HAS_CUSTOM_VALIDATION {
            let examples = D::examples();
            if !examples.is_empty() {
                let s: &&str = u.choose(examples)?;
                // examples() contract: the domain author guarantees these are valid.
                return Ok(Key::from(SmartString::from(*s)));
            }
        }

        // Constructive path: enumerate ASCII printable (U+0020–U+007E).
        let alphabet: Vec<char> = (' '..='~')
            .filter(|&c| D::allowed_characters(c))
            .collect();

        if alphabet.is_empty() {
            return Err(arbitrary::Error::EmptyChoose);
        }

        let start_chars: Vec<char> = alphabet
            .iter()
            .copied()
            .filter(|&c| D::allowed_start_character(c))
            .collect();

        if start_chars.is_empty() {
            return Err(arbitrary::Error::EmptyChoose);
        }

        let min_len = D::min_length().max(1);
        let max_len = D::MAX_LENGTH.max(min_len);
        let length: usize = u.int_in_range(min_len..=max_len)?;

        let mut result = SmartString::new();
        let mut prev: Option<char> = None;

        for pos in 0..length {
            let is_last = pos == length - 1;

            let candidates: Vec<char> = match pos {
                // Single-character key: must satisfy both start and end predicates.
                0 if is_last => start_chars
                    .iter()
                    .copied()
                    .filter(|&c| D::allowed_end_character(c))
                    .collect(),
                // First character of a multi-character key.
                0 => start_chars.clone(),
                // Interior or final character.
                _ => {
                    let p = prev.expect("prev is set after position 0");
                    alphabet
                        .iter()
                        .copied()
                        .filter(|&c| D::allowed_consecutive_characters(p, c))
                        .filter(|&c| !is_last || D::allowed_end_character(c))
                        .collect()
                }
            };

            if candidates.is_empty() {
                return Err(arbitrary::Error::EmptyChoose);
            }

            let &c = u.choose(&candidates)?;
            result.push(c);
            prev = Some(c);
        }

        Key::new(result.as_str()).map_err(|_| arbitrary::Error::IncorrectFormat)
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        // Rough estimate: 1 byte for length selection + up to MAX_LENGTH chars.
        (1, Some(D::MAX_LENGTH * 4 + 8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A minimal test domain: only lowercase ASCII letters, length 1–16.
    #[derive(Debug)]
    struct SimpleDomain;

    impl crate::domain::Domain for SimpleDomain {
        const DOMAIN_NAME: &'static str = "simple";
    }

    impl KeyDomain for SimpleDomain {
        const MAX_LENGTH: usize = 16;

        fn allowed_characters(c: char) -> bool {
            c.is_ascii_lowercase()
        }
    }

    // A custom-validation domain backed by examples only.
    #[derive(Debug)]
    struct ExamplesDomain;

    impl crate::domain::Domain for ExamplesDomain {
        const DOMAIN_NAME: &'static str = "examples";
    }

    impl KeyDomain for ExamplesDomain {
        const MAX_LENGTH: usize = 32;
        const HAS_CUSTOM_VALIDATION: bool = true;

        fn examples() -> &'static [&'static str] {
            &["foo", "bar", "baz"]
        }

        fn allowed_characters(c: char) -> bool {
            c.is_ascii_lowercase()
        }
    }

    /// Constructs an `Unstructured` from a repeated-byte buffer; returns None on error.
    fn arb_from_bytes(data: &[u8]) -> arbitrary::Unstructured<'_> {
        arbitrary::Unstructured::new(data)
    }

    #[test]
    fn id_arbitrary_is_non_zero() {
        #[derive(Debug)]
        struct TestId;
        impl crate::domain::Domain for TestId {
            const DOMAIN_NAME: &'static str = "test_id";
        }
        impl crate::domain::IdDomain for TestId {}

        let data = [0u8; 64];
        let mut u = arb_from_bytes(&data);
        // Even all-zero input must produce a non-zero Id.
        let id = Id::<TestId>::arbitrary(&mut u);
        if let Ok(id) = id {
            assert!(id.get() != 0);
        }
    }

    #[test]
    fn key_arbitrary_simple_domain_is_valid() {
        // Generate 256 keys from arbitrary data; all must be valid for SimpleDomain.
        let data: Vec<u8> = (0u8..=255).cycle().take(8192).collect();
        let mut u = arb_from_bytes(&data);
        let mut success_count = 0usize;
        for _ in 0..100 {
            match Key::<SimpleDomain>::arbitrary(&mut u) {
                Ok(key) => {
                    // Re-validate via Key::new to confirm correctness.
                    assert!(
                        Key::<SimpleDomain>::new(key.as_str()).is_ok(),
                        "generated key failed re-validation: {key:?}"
                    );
                    success_count += 1;
                }
                Err(arbitrary::Error::NotEnoughData) => break,
                Err(e) => panic!("unexpected error: {e:?}"),
            }
        }
        assert!(success_count > 0, "no keys generated");
    }

    #[test]
    fn key_arbitrary_examples_domain_draws_from_examples() {
        let data: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        let mut u = arb_from_bytes(&data);
        let valid: &[&str] = ExamplesDomain::examples();
        let mut count = 0usize;
        for _ in 0..50 {
            match Key::<ExamplesDomain>::arbitrary(&mut u) {
                Ok(key) => {
                    assert!(
                        valid.contains(&key.as_str()),
                        "generated key not in examples: {key:?}"
                    );
                    count += 1;
                }
                Err(arbitrary::Error::NotEnoughData) => break,
                Err(e) => panic!("unexpected error: {e:?}"),
            }
        }
        assert!(count > 0);
    }

    #[test]
    fn key_arbitrary_min_eq_max_produces_fixed_length() {
        #[derive(Debug)]
        struct FixedLen;
        impl crate::domain::Domain for FixedLen {
            const DOMAIN_NAME: &'static str = "fixed";
        }
        impl KeyDomain for FixedLen {
            const MAX_LENGTH: usize = 4;
            fn min_length() -> usize {
                4
            }
            fn allowed_characters(c: char) -> bool {
                c == 'a'
            }
        }

        let data: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        let mut u = arb_from_bytes(&data);
        for _ in 0..10 {
            match Key::<FixedLen>::arbitrary(&mut u) {
                Ok(key) => assert_eq!(key.len(), 4),
                Err(arbitrary::Error::NotEnoughData) => break,
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}

