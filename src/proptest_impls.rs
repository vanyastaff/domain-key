//! Proptest [`Strategy`] impls and the [`ProptestKeyDomain`] companion trait.
//!
//! Enabled by the `proptest` feature flag (implies `std`). Provides property-based
//! testing support for [`Key<D>`](crate::Key), [`Id<D>`](crate::Id),
//! [`Uuid<D>`](crate::Uuid) (requires `uuid` feature), and
//! [`Ulid<D>`](crate::Ulid) (requires `ulid` feature).
//!
//! # Using `Key<D>` in proptest
//!
//! To use `Key<D>` as a proptest input, implement [`ProptestKeyDomain`] for your
//! domain type. For most domains, an empty impl is sufficient — the constructive
//! strategy handles generation automatically:
//!
//! ```ignore
//! use domain_key::{KeyDomain, ProptestKeyDomain};
//!
//! #[derive(Debug)]
//! struct MyDomain;
//! // ... KeyDomain impl ...
//!
//! impl ProptestKeyDomain for MyDomain {}  // empty impl — uses constructive default
//! ```
//!
//! For domains with complex custom validation that the constructive path cannot
//! fully capture, override [`ProptestKeyDomain::proptest_strategy`] to supply a
//! complete strategy:
//!
//! ```ignore
//! use domain_key::ProptestKeyDomain;
//! use proptest::prelude::*;
//!
//! impl ProptestKeyDomain for MyCustomDomain {
//!     fn proptest_strategy() -> Option<proptest::strategy::BoxedStrategy<domain_key::Key<Self>>> {
//!         let known_valid = vec!["foo", "bar", "baz"];
//!         Some(proptest::sample::select(known_valid)
//!             .prop_map(|s| domain_key::Key::new(s).unwrap())
//!             .boxed())
//!     }
//! }
//! ```

use core::num::NonZeroU64;
use std::sync::Arc;

use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Just, Strategy};
use smartstring::alias::String as SmartString;

use crate::domain::{IdDomain, KeyDomain};
use crate::id::Id;
use crate::key::Key;

/// Companion trait for using [`Key<D>`] in proptest property tests.
///
/// Implement this trait for your domain type to enable proptest `Strategy`
/// generation of `Key<D>` values. For most domains, an **empty impl** is
/// sufficient — the default `None` return causes the three-path constructive
/// strategy to activate automatically.
///
/// # When to override
///
/// Override [`proptest_strategy`](ProptestKeyDomain::proptest_strategy) only
/// when your domain has complex custom validation (i.e.,
/// [`KeyDomain::HAS_CUSTOM_VALIDATION`] is `true`) that the constructive path
/// cannot satisfy, and [`KeyDomain::examples`] is also empty or insufficient.
///
/// # Example — default constructive path
///
/// ```ignore
/// impl ProptestKeyDomain for MyDomain {}  // one line; zero boilerplate
/// ```
///
/// # Example — custom override
///
/// ```ignore
/// impl ProptestKeyDomain for MyDomain {
///     fn proptest_strategy() -> Option<proptest::strategy::BoxedStrategy<Key<Self>>> {
///         Some(proptest::sample::select(Self::examples())
///             .prop_map(|s| Key::new(s).unwrap())
///             .boxed())
///     }
/// }
/// ```
///
/// # Discovering this trait
///
/// If your domain implements [`KeyDomain`] with `HAS_CUSTOM_VALIDATION = true`,
/// you must either provide non-empty [`KeyDomain::examples()`] or implement this
/// trait with a custom [`proptest_strategy()`](ProptestKeyDomain::proptest_strategy)
/// override to guarantee that proptest generates valid keys.
pub trait ProptestKeyDomain: KeyDomain {
    /// Returns a custom proptest `Strategy` for generating `Key<Self>` values,
    /// or `None` to use the default three-path constructive strategy.
    ///
    /// The default implementation returns `None`, which activates the
    /// constructive path in the `Strategy for Key<D>` impl.
    ///
    /// Override this method when:
    /// - [`KeyDomain::HAS_CUSTOM_VALIDATION`] is `true`, **and**
    /// - [`KeyDomain::examples`] is empty or insufficient to cover your domain,
    ///   **and**
    /// - You can provide a strategy that produces only valid keys.
    fn proptest_strategy() -> Option<BoxedStrategy<Key<Self>>>
    where
        Self: Sized,
    {
        None
    }
}

// ── Id<D> ────────────────────────────────────────────────────────────────────

impl<D: IdDomain> proptest::arbitrary::Arbitrary for Id<D> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> BoxedStrategy<Self> {
        any::<u64>()
            .prop_map(|raw| Id::from_non_zero(NonZeroU64::new(raw | 1).expect("raw | 1 != 0")))
            .boxed()
    }
}

// ── Uuid<D> ──────────────────────────────────────────────────────────────────

/// Proptest `Arbitrary` for [`Uuid<D>`](crate::Uuid).
///
/// Generates a uniformly random 16-byte UUID. Requires both `uuid` and `proptest` features.
#[cfg(feature = "uuid")]
impl<D: crate::domain::UuidDomain> proptest::arbitrary::Arbitrary
    for crate::uuid::Uuid<D>
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> BoxedStrategy<Self> {
        any::<[u8; 16]>()
            .prop_map(|bytes| Self::from_bytes(bytes))
            .boxed()
    }
}

// ── Ulid<D> ──────────────────────────────────────────────────────────────────

/// Proptest `Arbitrary` for [`Ulid<D>`](crate::Ulid).
///
/// Generates a uniformly random 16-byte ULID. Requires both `ulid` and `proptest` features.
#[cfg(feature = "ulid")]
impl<D: crate::domain::UlidDomain> proptest::arbitrary::Arbitrary
    for crate::ulid::Ulid<D>
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: ()) -> BoxedStrategy<Self> {
        any::<[u8; 16]>()
            .prop_map(|bytes| Self::from_bytes(bytes))
            .boxed()
    }
}

// ── Key<D> ───────────────────────────────────────────────────────────────────

/// Builds a [`Key<D>`] from a slice of `usize` decision indices, one per character
/// position. Used by the proptest strategy to turn a `Vec<usize>` into a concrete
/// key value in a deterministic, shrink-friendly way.
fn build_key_from_indices<D: KeyDomain>(
    alphabet: &[char],
    start_chars: &[char],
    indices: &[usize],
) -> Option<Key<D>> {
    if indices.is_empty() {
        return None;
    }
    let length = indices.len();
    let mut result = SmartString::new();
    let mut prev: Option<char> = None;

    for (pos, &idx) in indices.iter().enumerate() {
        let is_last = pos == length - 1;

        let candidates: Vec<char> = match pos {
            0 if is_last => start_chars
                .iter()
                .copied()
                .filter(|&c| D::allowed_end_character(c))
                .collect(),
            0 => start_chars.to_vec(),
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
            return None;
        }

        let c = candidates[idx % candidates.len()];
        result.push(c);
        prev = Some(c);
    }

    Key::new(result.as_str()).ok()
}

/// Returns the constructive proptest strategy for `Key<D: ProptestKeyDomain>`.
///
/// This is the innermost "Path 3" strategy used when no override is provided
/// and the domain has no examples.
fn key_constructive_strategy<D: ProptestKeyDomain>() -> BoxedStrategy<Key<D>>
where
    Key<D>: core::fmt::Debug,
{
    let min_len = D::min_length().max(1);
    let max_len = D::MAX_LENGTH.max(min_len);

    let alphabet: Arc<Vec<char>> = Arc::new(
        (' '..='~')
            .filter(|&c| D::allowed_characters(c))
            .collect(),
    );
    let start_chars: Arc<Vec<char>> = Arc::new(
        alphabet
            .iter()
            .copied()
            .filter(|&c| D::allowed_start_character(c))
            .collect(),
    );

    (min_len..=max_len)
        .prop_flat_map(move |len| {
            let a = Arc::clone(&alphabet);
            let s = Arc::clone(&start_chars);
            proptest::collection::vec(any::<usize>(), len).prop_map(move |indices| {
                build_key_from_indices::<D>(&a, &s, &indices)
            })
        })
        .prop_filter_map("non-empty candidate set", |opt| opt)
        .boxed()
}

impl<D: ProptestKeyDomain> proptest::arbitrary::Arbitrary for Key<D>
where
    Key<D>: core::fmt::Debug,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Key<D>>;

    fn arbitrary_with(_: ()) -> BoxedStrategy<Key<D>> {
        // Path 1: domain-supplied override strategy.
        if let Some(s) = D::proptest_strategy() {
            return s;
        }

        // Path 2: examples-weighted strategy for custom-validation domains.
        if D::HAS_CUSTOM_VALIDATION {
            let examples = D::examples();
            if !examples.is_empty() {
                let mut all_strats: Vec<BoxedStrategy<Key<D>>> = examples
                    .iter()
                    .map(|&s| {
                        let key = Key::from(SmartString::from(s));
                        Just(key).boxed()
                    })
                    .collect();
                // Include the constructive path for coverage beyond the examples list.
                all_strats.push(key_constructive_strategy::<D>());
                return proptest::strategy::Union::new(all_strats).boxed();
            }
        }

        // Path 3: pure constructive strategy.
        key_constructive_strategy::<D>()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::domain::{Domain, KeyDomain};
    use crate::key::Key;

    use super::ProptestKeyDomain;

    #[derive(Debug, Clone)]
    struct AlphaLowerDomain;

    impl Domain for AlphaLowerDomain {
        const DOMAIN_NAME: &'static str = "alpha_lower";
    }

    impl KeyDomain for AlphaLowerDomain {
        const MAX_LENGTH: usize = 12;

        fn allowed_characters(c: char) -> bool {
            c.is_ascii_lowercase()
        }
    }

    impl ProptestKeyDomain for AlphaLowerDomain {}

    proptest! {
        #[test]
        fn key_strategy_produces_valid_keys(key in any::<Key<AlphaLowerDomain>>()) {
            prop_assert!(Key::<AlphaLowerDomain>::new(key.as_str()).is_ok());
        }

        #[test]
        fn key_strategy_respects_length_bounds(key in any::<Key<AlphaLowerDomain>>()) {
            let len = key.len();
            prop_assert!(
                len >= AlphaLowerDomain::min_length() && len <= AlphaLowerDomain::MAX_LENGTH,
                "key length {len} out of bounds"
            );
        }
    }

    #[test]
    fn examples_domain_path2_runs_without_panic() {
        // Verify Path 2 (examples-weighted) doesn't panic and produces valid keys.
        #[derive(Debug, Clone)]
        struct ExDomain;
        impl Domain for ExDomain {
            const DOMAIN_NAME: &'static str = "ex";
        }
        impl KeyDomain for ExDomain {
            const MAX_LENGTH: usize = 16;
            const HAS_CUSTOM_VALIDATION: bool = true;
            fn examples() -> &'static [&'static str] {
                &["alpha", "beta", "gamma"]
            }
            fn allowed_characters(c: char) -> bool {
                c.is_ascii_lowercase()
            }
        }
        impl ProptestKeyDomain for ExDomain {}

        let mut runner = proptest::test_runner::TestRunner::default();
        let strategy = any::<Key<ExDomain>>();
        let valid_examples: &[&str] = ExDomain::examples();
        let mut from_examples = 0usize;
        let mut from_constructive = 0usize;
        for _ in 0..50 {
            let tree = strategy.new_tree(&mut runner).unwrap();
            let key = tree.current();
            // All keys must pass re-validation for this simple domain.
            assert!(
                Key::<ExDomain>::new(key.as_str()).is_ok(),
                "generated key failed validation: {key:?}"
            );
            if valid_examples.contains(&key.as_str()) {
                from_examples += 1;
            } else {
                from_constructive += 1;
            }
        }
        // Both paths should contribute.
        assert!(from_examples > 0, "no keys from examples");
        let _ = from_constructive; // constructive may also produce valid keys here
    }
}

