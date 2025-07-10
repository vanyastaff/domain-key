//! Validation utilities and helper traits for domain-key
//!
//! This module provides comprehensive validation functionality, including
//! validation without key creation, batch validation, and helper traits
//! for converting various types into keys.

use crate::domain::KeyDomain;
use crate::error::KeyParseError;
use crate::key::Key;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::vec;

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

/// Check if a string would be a valid key for a domain without creating the key
///
/// This is useful for pre-validation or filtering operations where you don't
/// need the actual key object.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// assert!(validation::is_valid_key::<TestDomain>("good_key"));
/// assert!(!validation::is_valid_key::<TestDomain>(""));
/// ```
pub fn is_valid_key<T: KeyDomain>(key: &str) -> bool {
    validate_key::<T>(key).is_ok()
}

/// Validate a key string and return detailed error information
///
/// This performs the same validation as `Key::new` but without creating
/// the key object, making it useful for validation-only scenarios.
///
/// # Errors
///
/// Returns `KeyParseError` if the key fails common or domain-specific validation
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation, KeyParseError};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// match validation::validate_key::<TestDomain>("") {
///     Err(KeyParseError::Empty) => println!("Key is empty"),
///     Err(e) => println!("Other error: {}", e),
///     Ok(()) => println!("Key is valid"),
/// }
/// ```
pub fn validate_key<T: KeyDomain>(key: &str) -> Result<(), KeyParseError> {
    Key::<T>::validate_common::<T>(key)?;
    let normalized = Key::<T>::normalize::<T>(key);
    T::validate_domain_rules(&normalized)
}

/// Get validation help text for a domain
///
/// Returns the help text provided by the domain's `validation_help` method,
/// if any. This can be useful for providing user-friendly error messages.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
///     fn validation_help() -> Option<&'static str> {
///         Some("Keys must be alphanumeric with underscores")
///     }
/// }
///
/// if let Some(help) = validation::validation_help::<TestDomain>() {
///     println!("Validation help: {}", help);
/// }
/// ```
pub fn validation_help<T: KeyDomain>() -> Option<&'static str> {
    T::validation_help()
}

/// Get detailed information about validation rules for a domain
///
/// Returns a formatted string containing comprehensive information about
/// the domain's validation rules and characteristics.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
///     const MAX_LENGTH: usize = 32;
/// }
///
/// let info = validation::validation_info::<TestDomain>();
/// println!("{}", info);
/// // Output:
/// // Domain: test
/// // Max length: 32
/// ```
pub fn validation_info<T: KeyDomain>() -> String {
    let mut info = format!("Domain: {}\n", T::DOMAIN_NAME);
    info.push_str(&format!("Max length: {}\n", T::MAX_LENGTH));
    info.push_str(&format!("Min length: {}\n", T::min_length()));
    info.push_str(&format!("Expected length: {}\n", T::EXPECTED_LENGTH));
    info.push_str(&format!("Case insensitive: {}\n", T::CASE_INSENSITIVE));
    info.push_str(&format!(
        "Custom validation: {}\n",
        T::HAS_CUSTOM_VALIDATION
    ));
    info.push_str(&format!(
        "Custom normalization: {}\n",
        T::HAS_CUSTOM_NORMALIZATION
    ));
    info.push_str(&format!(
        "Default separator: '{}'\n",
        T::default_separator()
    ));

    if let Some(help) = T::validation_help() {
        info.push_str("Help: ");
        info.push_str(help);
        info.push('\n');
    }

    let examples = T::examples();
    if !examples.is_empty() {
        info.push_str("Examples: ");
        for (i, example) in examples.iter().enumerate() {
            if i > 0 {
                info.push_str(", ");
            }
            info.push_str(example);
        }
        info.push('\n');
    }

    info
}

/// Validate multiple keys at once
///
/// This function validates a collection of keys and returns which ones
/// are valid and which ones failed validation.
///
/// # Arguments
///
/// * `keys` - Iterator of string-like items to validate
///
/// # Returns
///
/// A tuple containing:
/// - Vector of valid key strings
/// - Vector of (invalid key string, error) pairs
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// let keys = vec!["valid_key", "", "another_valid", "bad key"];
/// let (valid, invalid) = validation::validate_batch::<TestDomain, _>(keys);
///
/// assert_eq!(valid.len(), 2);
/// assert_eq!(invalid.len(), 2);
/// ```
pub fn validate_batch<T: KeyDomain, I>(keys: I) -> (Vec<String>, Vec<(String, KeyParseError)>)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut valid = Vec::new();
    let mut invalid = Vec::new();

    for key in keys {
        let key_str = key.as_ref();
        match validate_key::<T>(key_str) {
            Ok(()) => valid.push(key_str.to_string()),
            Err(e) => invalid.push((key_str.to_string(), e)),
        }
    }

    (valid, invalid)
}

/// Filter a collection of strings to only include valid keys
///
/// This function takes an iterator of strings and returns only those
/// that would be valid keys for the specified domain.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// let candidates = vec!["valid_key", "", "another_valid", "bad key"];
/// let valid_keys: Vec<_> = validation::filter_valid::<TestDomain, _>(candidates).collect();
///
/// assert_eq!(valid_keys.len(), 2);
/// ```
pub fn filter_valid<T: KeyDomain, I>(keys: I) -> impl Iterator<Item = String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    keys.into_iter().filter_map(|key| {
        let key_str = key.as_ref();
        if is_valid_key::<T>(key_str) {
            Some(key_str.to_string())
        } else {
            None
        }
    })
}

/// Count how many strings in a collection would be valid keys
///
/// This is more efficient than filtering when you only need the count.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// let candidates = vec!["valid_key", "", "another_valid", "bad key"];
/// let count = validation::count_valid::<TestDomain, _>(candidates);
///
/// assert_eq!(count, 2);
/// ```
pub fn count_valid<T: KeyDomain, I>(keys: I) -> usize
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    keys.into_iter()
        .filter(|key| is_valid_key::<T>(key.as_ref()))
        .count()
}

/// Check if all strings in a collection would be valid keys
///
/// Returns `true` only if every string in the collection would be a valid key.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// let all_valid = vec!["valid_key", "another_valid"];
/// let mixed = vec!["valid_key", "", "another_valid"];
///
/// assert!(validation::all_valid::<TestDomain, _>(all_valid));
/// assert!(!validation::all_valid::<TestDomain, _>(mixed));
/// ```
pub fn all_valid<T: KeyDomain, I>(keys: I) -> bool
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    keys.into_iter().all(|key| is_valid_key::<T>(key.as_ref()))
}

/// Check if any string in a collection would be a valid key
///
/// Returns `true` if at least one string in the collection would be a valid key.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// let mixed = vec!["", "valid_key", ""];
/// let all_invalid = vec!["", ""];
///
/// assert!(validation::any_valid::<TestDomain, _>(mixed));
/// assert!(!validation::any_valid::<TestDomain, _>(all_invalid));
/// ```
pub fn any_valid<T: KeyDomain, I>(keys: I) -> bool
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    keys.into_iter().any(|key| is_valid_key::<T>(key.as_ref()))
}

// ============================================================================
// CONVENIENCE TRAITS
// ============================================================================

/// Helper trait for converting strings to keys
///
/// This trait provides convenient methods for converting various string types
/// into keys with proper error handling.
pub trait IntoKey<T: KeyDomain> {
    /// Convert into a key, returning an error if validation fails
    ///
    /// # Errors
    ///
    /// Returns `KeyParseError` if the string fails validation for the domain
    fn into_key(self) -> Result<Key<T>, KeyParseError>;

    /// Convert into a key, returning None if validation fails
    ///
    /// This is useful when you want to filter out invalid keys rather than
    /// handle errors explicitly.
    fn try_into_key(self) -> Option<Key<T>>;
}

impl<T: KeyDomain> IntoKey<T> for &str {
    #[inline]
    fn into_key(self) -> Result<Key<T>, KeyParseError> {
        Key::new(self)
    }

    #[inline]
    fn try_into_key(self) -> Option<Key<T>> {
        Key::try_new(self)
    }
}

impl<T: KeyDomain> IntoKey<T> for String {
    #[inline]
    fn into_key(self) -> Result<Key<T>, KeyParseError> {
        Key::from_string(self)
    }

    #[inline]
    fn try_into_key(self) -> Option<Key<T>> {
        Key::from_string(self).ok()
    }
}

impl<T: KeyDomain> IntoKey<T> for &String {
    #[inline]
    fn into_key(self) -> Result<Key<T>, KeyParseError> {
        Key::new(self)
    }

    #[inline]
    fn try_into_key(self) -> Option<Key<T>> {
        Key::try_new(self)
    }
}

// ============================================================================
// VALIDATION BUILDER
// ============================================================================

/// Builder for creating comprehensive validation configurations
///
/// This builder allows you to create complex validation scenarios with
/// custom requirements and error handling.
#[derive(Debug)]
pub struct ValidationBuilder<T: KeyDomain> {
    allow_empty_collection: bool,
    max_failures: Option<usize>,
    stop_on_first_error: bool,
    custom_validator: Option<fn(&str) -> Result<(), KeyParseError>>,
    _phantom: core::marker::PhantomData<T>,
}

impl<T: KeyDomain> Default for ValidationBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: KeyDomain> ValidationBuilder<T> {
    /// Create a new validation builder
    pub fn new() -> Self {
        Self {
            allow_empty_collection: false,
            max_failures: None,
            stop_on_first_error: false,
            custom_validator: None,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Allow validation of empty collections
    pub fn allow_empty_collection(mut self, allow: bool) -> Self {
        self.allow_empty_collection = allow;
        self
    }

    /// Set maximum number of failures before stopping validation
    pub fn max_failures(mut self, max: usize) -> Self {
        self.max_failures = Some(max);
        self
    }

    /// Stop validation on the first error encountered
    pub fn stop_on_first_error(mut self, stop: bool) -> Self {
        self.stop_on_first_error = stop;
        self
    }

    /// Add a custom validator function
    pub fn custom_validator(mut self, validator: fn(&str) -> Result<(), KeyParseError>) -> Self {
        self.custom_validator = Some(validator);
        self
    }

    /// Validate a collection of strings with the configured settings
    pub fn validate<I>(&self, keys: I) -> ValidationResult
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut valid = Vec::new();
        let mut errors = Vec::new();
        let keys: Vec<_> = keys.into_iter().collect();

        if keys.is_empty() && !self.allow_empty_collection {
            return ValidationResult {
                valid,
                errors: vec![("".to_string(), KeyParseError::Empty)],
                total_processed: 0,
            };
        }

        for key in keys {
            let key_str = key.as_ref();

            // Check if we should stop due to error limits
            if let Some(max) = self.max_failures {
                if errors.len() >= max {
                    break;
                }
            }

            if self.stop_on_first_error && !errors.is_empty() {
                break;
            }

            // Validate with domain rules
            match validate_key::<T>(key_str) {
                Ok(()) => {
                    // Apply custom validator if present
                    if let Some(custom) = self.custom_validator {
                        match custom(key_str) {
                            Ok(()) => valid.push(key_str.to_string()),
                            Err(e) => errors.push((key_str.to_string(), e)),
                        }
                    } else {
                        valid.push(key_str.to_string());
                    }
                }
                Err(e) => errors.push((key_str.to_string(), e)),
            }
        }

        ValidationResult {
            total_processed: valid.len() + errors.len(),
            valid,
            errors,
        }
    }
}

/// Result of a validation operation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Number of items processed before stopping
    pub total_processed: usize,
    /// Valid key strings
    pub valid: Vec<String>,
    /// Invalid keys with their errors
    pub errors: Vec<(String, KeyParseError)>,
}

impl ValidationResult {
    /// Check if all processed items were valid
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of valid items
    pub fn valid_count(&self) -> usize {
        self.valid.len()
    }

    /// Get the number of invalid items
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_processed == 0 {
            0.0
        } else {
            (self.valid.len() as f64 / self.total_processed as f64) * 100.0
        }
    }

    /// Convert all valid strings to keys
    pub fn into_keys<T: KeyDomain>(self) -> Result<Vec<Key<T>>, KeyParseError> {
        self.valid
            .into_iter()
            .map(|s| Key::from_string(s))
            .collect()
    }

    /// Try to convert all valid strings to keys, ignoring failures
    pub fn try_into_keys<T: KeyDomain>(self) -> Vec<Key<T>> {
        self.valid
            .into_iter()
            .filter_map(|s| Key::from_string(s).ok())
            .collect()
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Create a validation builder with common settings for strict validation
pub fn strict_validator<T: KeyDomain>() -> ValidationBuilder<T> {
    ValidationBuilder::new()
        .stop_on_first_error(true)
        .allow_empty_collection(false)
}

/// Create a validation builder with common settings for lenient validation
pub fn lenient_validator<T: KeyDomain>() -> ValidationBuilder<T> {
    ValidationBuilder::new()
        .stop_on_first_error(false)
        .allow_empty_collection(true)
}

/// Quickly validate and convert a collection of strings to keys
///
/// This is a convenience function that combines validation and conversion
/// in a single step.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, validation};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
///
/// let strings = vec!["key1", "key2", "key3"];
/// let keys = validation::quick_convert::<TestDomain, _>(strings).unwrap();
///
/// assert_eq!(keys.len(), 3);
/// ```
pub fn quick_convert<T: KeyDomain, I>(keys: I) -> Result<Vec<Key<T>>, Vec<(String, KeyParseError)>>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let (valid, invalid) = validate_batch::<T, I>(keys);

    if invalid.is_empty() {
        Ok(valid
            .into_iter()
            .map(|s| Key::from_string(s))
            .collect::<Result<Vec<_>, _>>()
            .expect("All keys were pre-validated"))
    } else {
        Err(invalid)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test domain
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct TestDomain;

    impl KeyDomain for TestDomain {
        const DOMAIN_NAME: &'static str = "test";
        const MAX_LENGTH: usize = 32;

        fn validation_help() -> Option<&'static str> {
            Some("Test domain help")
        }

        fn examples() -> &'static [&'static str] {
            &["example1", "example2"]
        }
    }

    #[test]
    fn test_is_valid_key() {
        assert!(is_valid_key::<TestDomain>("valid_key"));
        assert!(!is_valid_key::<TestDomain>(""));
        assert!(!is_valid_key::<TestDomain>("a".repeat(50).as_str()));
    }

    #[test]
    fn test_validate_key() {
        assert!(validate_key::<TestDomain>("valid_key").is_ok());
        assert!(validate_key::<TestDomain>("").is_err());
    }

    #[test]
    fn test_validation_info() {
        let info = validation_info::<TestDomain>();
        assert!(info.contains("Domain: test"));
        assert!(info.contains("Max length: 32"));
        assert!(info.contains("Help: Test domain help"));
        assert!(info.contains("Examples: example1, example2"));
    }

    #[test]
    fn test_validate_batch() {
        let keys = vec!["valid1", "", "valid2", "bad key"];
        let (valid, invalid) = validate_batch::<TestDomain, _>(&keys);

        assert_eq!(valid.len(), 2);
        assert_eq!(invalid.len(), 2);
        assert!(valid.contains(&"valid1".to_string()));
        assert!(valid.contains(&"valid2".to_string()));
    }

    #[test]
    fn test_filter_valid() {
        let keys = vec!["valid1", "", "valid2", "bad key"];
        let valid: Vec<_> = filter_valid::<TestDomain, _>(&keys).collect();

        assert_eq!(valid.len(), 2);
        assert!(valid.contains(&"valid1".to_string()));
        assert!(valid.contains(&"valid2".to_string()));
    }

    #[test]
    fn test_count_valid() {
        let keys = vec!["valid1", "", "valid2", "bad key"];
        let count = count_valid::<TestDomain, _>(&keys);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_all_valid() {
        let all_valid_keys = vec!["valid1", "valid2"];
        let mixed = vec!["valid1", "", "valid2"];

        assert!(all_valid::<TestDomain, _>(&all_valid_keys));
        assert!(!all_valid::<TestDomain, _>(&mixed));
    }

    #[test]
    fn test_any_valid() {
        let mixed = vec!["", "valid1", ""];
        let all_invalid = vec!["", ""];

        assert!(any_valid::<TestDomain, _>(&mixed));
        assert!(!any_valid::<TestDomain, _>(&all_invalid));
    }

    #[test]
    fn test_into_key_trait() {
        let key1: Key<TestDomain> = "test_key".into_key().unwrap();
        let key2: Key<TestDomain> = "another_key".to_string().into_key().unwrap();

        assert_eq!(key1.as_str(), "test_key");
        assert_eq!(key2.as_str(), "another_key");

        let invalid: Option<Key<TestDomain>> = "".try_into_key();
        assert!(invalid.is_none());
    }

    #[test]
    fn test_validation_builder() {
        let builder = ValidationBuilder::<TestDomain>::new()
            .allow_empty_collection(true)
            .max_failures(2)
            .stop_on_first_error(false);

        let keys = vec!["valid1", "", "valid2", "", "valid3"];
        let result = builder.validate(&keys);

        assert_eq!(result.valid_count(), 3);
        assert_eq!(result.error_count(), 2);
        assert!(!result.is_success());
        assert!(result.success_rate() > 50.0);
    }

    #[test]
    fn test_validation_result() {
        let keys = vec!["valid1", "valid2"];
        let (valid, errors) = validate_batch::<TestDomain, _>(keys);

        let result = ValidationResult {
            total_processed: valid.len() + errors.len(),
            valid,
            errors,
        };

        assert!(result.is_success());
        assert_eq!(result.valid_count(), 2);
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.success_rate(), 100.0);

        let keys = result.try_into_keys::<TestDomain>();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_strict_validator() {
        let validator = strict_validator::<TestDomain>();
        let keys = vec!["valid", "", "another"];
        let result = validator.validate(&keys);

        // Should stop on first error (empty string)
        assert_eq!(result.total_processed, 2); // "valid" + "" (error)
        assert_eq!(result.valid_count(), 1);
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_lenient_validator() {
        let validator = lenient_validator::<TestDomain>();
        let keys = vec!["valid", "", "another"];
        let result = validator.validate(&keys);

        // Should process all items
        assert_eq!(result.total_processed, 3);
        assert_eq!(result.valid_count(), 2);
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_quick_convert() {
        let strings = vec!["key1", "key2", "key3"];
        let keys = quick_convert::<TestDomain, _>(&strings).unwrap();
        assert_eq!(keys.len(), 3);

        let mixed = vec!["key1", "", "key2"];
        let result = quick_convert::<TestDomain, _>(&mixed);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_validator() {
        fn custom_check(key: &str) -> Result<(), KeyParseError> {
            if key.starts_with("custom_") {
                Ok(())
            } else {
                Err(KeyParseError::custom(9999, "Must start with custom_"))
            }
        }

        let validator = ValidationBuilder::<TestDomain>::new().custom_validator(custom_check);

        let keys = vec!["custom_key", "invalid_key"];
        let result = validator.validate(&keys);

        assert_eq!(result.valid_count(), 1);
        assert_eq!(result.error_count(), 1);
    }
}
