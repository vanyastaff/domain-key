//! Domain trait and related functionality for domain-key
//!
//! This module defines the `KeyDomain` trait that controls validation, normalization,
//! and optimization behavior for different key types. Each domain can customize
//! these behaviors through trait constants and methods.

use core::fmt;
use core::hash::Hash;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(feature = "std")]
use std::borrow::Cow;

use crate::error::KeyParseError;
use crate::key::DEFAULT_MAX_KEY_LENGTH;

// ============================================================================
// KEY DOMAIN TRAIT
// ============================================================================

/// Trait for key domain markers with optimization hints
///
/// This trait defines the behavior for different key domains, including
/// validation rules, normalization behavior, character restrictions, and
/// performance optimization hints.
///
/// # Implementation Requirements
///
/// Types implementing this trait must also implement:
/// - `'static + Send + Sync` - For thread safety and lifetime management
/// - `Debug + PartialEq + Eq + Hash + Ord + PartialOrd` - For standard operations
///
/// # Design Philosophy
///
/// The trait is designed to be both powerful and performant:
/// - **Const generics** for compile-time optimization hints
/// - **Associated constants** for zero-cost configuration
/// - **Default implementations** for common cases
/// - **Hooks** for custom behavior where needed
///
/// # Examples
///
/// ## Basic domain with optimization hints
/// ```rust
/// use domain_key::{KeyDomain, KeyParseError};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct UserDomain;
///
/// impl KeyDomain for UserDomain {
///     const DOMAIN_NAME: &'static str = "user";
///     const MAX_LENGTH: usize = 32;
///     const EXPECTED_LENGTH: usize = 16;    // Optimization hint
///     const TYPICALLY_SHORT: bool = true;   // Enable stack allocation
/// }
/// ```
///
/// ## Domain with custom validation
/// ```rust
/// use domain_key::{KeyDomain, KeyParseError};
/// use std::borrow::Cow;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct EmailDomain;
///
/// impl KeyDomain for EmailDomain {
///     const DOMAIN_NAME: &'static str = "email";
///     const HAS_CUSTOM_VALIDATION: bool = true;
///
///     fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
///         if !key.contains('@') {
///             return Err(KeyParseError::domain_error(Self::DOMAIN_NAME, "Email must contain @"));
///         }
///         Ok(())
///     }
///
///     fn allowed_characters(c: char) -> bool {
///         c.is_ascii_alphanumeric() || c == '@' || c == '.' || c == '_' || c == '-'
///     }
/// }
/// ```
pub trait KeyDomain:
    'static + Send + Sync + fmt::Debug + PartialEq + Eq + Hash + Ord + PartialOrd
{
    /// Human-readable name for this domain
    ///
    /// This name is used in error messages and debugging output.
    /// It should be a valid identifier that clearly describes the domain.
    const DOMAIN_NAME: &'static str;

    /// Maximum length for keys in this domain
    ///
    /// Keys longer than this will be rejected during validation.
    /// Setting this to a reasonable value enables performance optimizations.
    const MAX_LENGTH: usize = DEFAULT_MAX_KEY_LENGTH;

    /// Whether this domain has custom validation rules
    ///
    /// Set to `true` if you override `validate_domain_rules` with custom logic.
    /// This is used for introspection and debugging.
    const HAS_CUSTOM_VALIDATION: bool = false;

    /// Whether this domain has custom normalization rules
    ///
    /// Set to `true` if you override `normalize_domain` with custom logic.
    /// This is used for introspection and debugging.
    const HAS_CUSTOM_NORMALIZATION: bool = false;

    /// Optimization hint: expected average key length for this domain
    ///
    /// This hint helps the library pre-allocate the right amount of memory
    /// for string operations, reducing reallocations.
    const EXPECTED_LENGTH: usize = 32;

    /// Optimization hint: whether keys in this domain are typically short (≤32 chars)
    ///
    /// When `true`, enables stack allocation optimizations for the majority
    /// of keys in this domain. Set to `false` for domains with typically
    /// long keys to avoid stack overflow risks.
    const TYPICALLY_SHORT: bool = true;

    /// Optimization hint: whether keys in this domain are frequently compared
    ///
    /// When `true`, enables additional hash caching and comparison optimizations.
    /// Use for domains where keys are often used in hash maps or comparison operations.
    const FREQUENTLY_COMPARED: bool = false;

    /// Optimization hint: whether keys in this domain are frequently split
    ///
    /// When `true`, enables position caching for split operations.
    /// Use for domains where keys are regularly split into components.
    const FREQUENTLY_SPLIT: bool = false;

    /// Optimization hint: whether this domain allows case-insensitive operations
    ///
    /// When `true`, keys are normalized to lowercase and case-insensitive
    /// comparisons are optimized.
    const CASE_INSENSITIVE: bool = true;

    /// Domain-specific validation rules
    ///
    /// This method is called after common validation passes.
    /// Domains can enforce their own specific rules here.
    ///
    /// # Performance Considerations
    ///
    /// This method is called for every key creation, so it should be fast:
    /// - Prefer simple string operations over complex regex
    /// - Use early returns for quick rejection
    /// - Avoid expensive computations or I/O operations
    ///
    /// # Arguments
    ///
    /// * `key` - The normalized key string to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the key is valid for this domain
    /// * `Err(KeyParseError)` with the specific validation failure
    ///
    /// # Errors
    ///
    /// Returns `KeyParseError` if the key doesn't meet domain-specific
    /// validation requirements. Use `KeyParseError::domain_error` for
    /// consistent error formatting.
    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        let _ = key; // Suppress unused parameter warning
        Ok(()) // Default: no domain-specific validation
    }

    /// Check which characters are allowed for this domain
    ///
    /// Override this method to define domain-specific character restrictions.
    /// The default implementation allows ASCII alphanumeric characters and
    /// common separators.
    ///
    /// # Performance Considerations
    ///
    /// This method is called for every character in every key, so it must be
    /// extremely fast. Consider using lookup tables for complex character sets.
    ///
    /// # Arguments
    ///
    /// * `c` - Character to check
    ///
    /// # Returns
    ///
    /// `true` if the character is allowed, `false` otherwise
    #[must_use]
    fn allowed_characters(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
    }

    /// Domain-specific normalization
    ///
    /// This method is called after common normalization (trimming, lowercasing).
    /// Domains can apply additional normalization rules here.
    /// Uses `Cow` to avoid unnecessary allocations when no changes are needed.
    ///
    /// # Performance Considerations
    ///
    /// - Return `Cow::Borrowed` when no changes are needed
    /// - Only create `Cow::Owned` when actual changes are required
    /// - Keep normalization rules simple for best performance
    ///
    /// # Arguments
    ///
    /// * `key` - The key string after common normalization
    ///
    /// # Returns
    ///
    /// The normalized key string for this domain
    #[must_use]
    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        key // Default: no additional normalization
    }

    /// Check if a key has a reserved prefix for this domain
    ///
    /// Override this method to define domain-specific reserved prefixes.
    /// This can be used to prevent creation of keys that might conflict
    /// with system-generated keys or have special meaning.
    ///
    /// # Arguments
    ///
    /// * `key` - The key string to check
    ///
    /// # Returns
    ///
    /// `true` if the key uses a reserved prefix, `false` otherwise
    #[must_use]
    fn is_reserved_prefix(key: &str) -> bool {
        let _ = key;
        false // Default: no reserved prefixes
    }

    /// Check if a key has a reserved suffix for this domain
    ///
    /// Similar to `is_reserved_prefix` but for suffixes.
    ///
    /// # Arguments
    ///
    /// * `key` - The key string to check
    ///
    /// # Returns
    ///
    /// `true` if the key uses a reserved suffix, `false` otherwise
    #[must_use]
    fn is_reserved_suffix(key: &str) -> bool {
        let _ = key;
        false // Default: no reserved suffixes
    }

    /// Get domain-specific help text for validation errors
    ///
    /// This can provide users with helpful information about what
    /// constitutes a valid key for this domain.
    ///
    /// # Returns
    ///
    /// Optional help text that will be included in error messages
    #[must_use]
    fn validation_help() -> Option<&'static str> {
        None // Default: no help text
    }

    /// Get examples of valid keys for this domain
    ///
    /// This can be used in documentation, error messages, or testing
    /// to show users what valid keys look like.
    ///
    /// # Returns
    ///
    /// Array of example valid keys
    #[must_use]
    fn examples() -> &'static [&'static str] {
        &[] // Default: no examples
    }

    /// Get the default separator character for this domain
    ///
    /// This is used when composing keys from multiple parts.
    /// Different domains might prefer different separators.
    ///
    /// # Returns
    ///
    /// The preferred separator character
    #[must_use]
    fn default_separator() -> char {
        '_' // Default: underscore
    }

    /// Check if the key contains only ASCII characters
    ///
    /// Some domains might require ASCII-only keys for compatibility reasons.
    /// Override this method if your domain has specific ASCII requirements.
    ///
    /// # Arguments
    ///
    /// * `key` - The key string to check
    ///
    /// # Returns
    ///
    /// `true` if ASCII-only is required, `false` otherwise
    #[must_use]
    fn requires_ascii_only(key: &str) -> bool {
        let _ = key;
        false // Default: allow Unicode
    }

    /// Get the minimum allowed length for keys in this domain
    ///
    /// While empty keys are always rejected, some domains might require
    /// a minimum length greater than 1.
    ///
    /// # Returns
    ///
    /// The minimum allowed length (must be >= 1)
    #[must_use]
    fn min_length() -> usize {
        1 // Default: at least 1 character
    }

    /// Check if a character is allowed at the start of a key
    ///
    /// Some domains have stricter rules for the first character.
    /// The default implementation uses the same rules as `allowed_characters`.
    ///
    /// # Arguments
    ///
    /// * `c` - Character to check
    ///
    /// # Returns
    ///
    /// `true` if the character is allowed at the start, `false` otherwise
    #[must_use]
    fn allowed_start_character(c: char) -> bool {
        Self::allowed_characters(c) && c != '_' && c != '-' && c != '.'
    }

    /// Check if a character is allowed at the end of a key
    ///
    /// Some domains have stricter rules for the last character.
    /// The default implementation uses the same rules as `allowed_characters`.
    ///
    /// # Arguments
    ///
    /// * `c` - Character to check
    ///
    /// # Returns
    ///
    /// `true` if the character is allowed at the end, `false` otherwise
    #[must_use]
    fn allowed_end_character(c: char) -> bool {
        Self::allowed_characters(c) && c != '_' && c != '-' && c != '.'
    }

    /// Check if two consecutive characters are allowed
    ///
    /// This can be used to prevent patterns like double underscores
    /// or other consecutive special characters.
    ///
    /// # Arguments
    ///
    /// * `prev` - Previous character
    /// * `curr` - Current character
    ///
    /// # Returns
    ///
    /// `true` if the consecutive characters are allowed, `false` otherwise
    #[must_use]
    fn allowed_consecutive_characters(prev: char, curr: char) -> bool {
        // Default: prevent consecutive special characters
        !(prev == curr && (prev == '_' || prev == '-' || prev == '.'))
    }
}

// ============================================================================
// DOMAIN UTILITIES
// ============================================================================

/// Information about a domain's characteristics
///
/// This structure provides detailed information about a domain's configuration
/// and optimization hints, useful for debugging and introspection.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainInfo {
    /// Domain name
    pub name: &'static str,
    /// Maximum allowed length
    pub max_length: usize,
    /// Minimum allowed length
    pub min_length: usize,
    /// Expected average length
    pub expected_length: usize,
    /// Whether typically short
    pub typically_short: bool,
    /// Whether frequently compared
    pub frequently_compared: bool,
    /// Whether frequently split
    pub frequently_split: bool,
    /// Whether case insensitive
    pub case_insensitive: bool,
    /// Whether has custom validation
    pub has_custom_validation: bool,
    /// Whether has custom normalization
    pub has_custom_normalization: bool,
    /// Default separator character
    pub default_separator: char,
    /// Validation help text
    pub validation_help: Option<&'static str>,
    /// Example valid keys
    pub examples: &'static [&'static str],
}

impl fmt::Display for DomainInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Domain: {}", self.name)?;
        writeln!(
            f,
            "Length: {}-{} (expected: {})",
            self.min_length, self.max_length, self.expected_length
        )?;
        writeln!(f, "Optimization hints:")?;
        writeln!(f, "  • Typically short: {}", self.typically_short)?;
        writeln!(f, "  • Frequently compared: {}", self.frequently_compared)?;
        writeln!(f, "  • Frequently split: {}", self.frequently_split)?;
        writeln!(f, "  • Case insensitive: {}", self.case_insensitive)?;
        writeln!(f, "Custom features:")?;
        writeln!(f, "  • Custom validation: {}", self.has_custom_validation)?;
        writeln!(
            f,
            "  • Custom normalization: {}",
            self.has_custom_normalization
        )?;
        writeln!(f, "Default separator: '{}'", self.default_separator)?;

        if let Some(help) = self.validation_help {
            writeln!(f, "Validation help: {help}")?;
        }

        if !self.examples.is_empty() {
            writeln!(f, "Examples: {:?}", self.examples)?;
        }

        Ok(())
    }
}

/// Get comprehensive information about a domain
///
/// This function returns detailed information about a domain's configuration,
/// useful for debugging, documentation, and introspection.
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyDomain, domain_info};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
///     const MAX_LENGTH: usize = 32;
/// }
///
/// let info = domain_info::<TestDomain>();
/// println!("{}", info);
/// ```
#[must_use]
pub fn domain_info<T: KeyDomain>() -> DomainInfo {
    DomainInfo {
        name: T::DOMAIN_NAME,
        max_length: T::MAX_LENGTH,
        min_length: T::min_length(),
        expected_length: T::EXPECTED_LENGTH,
        typically_short: T::TYPICALLY_SHORT,
        frequently_compared: T::FREQUENTLY_COMPARED,
        frequently_split: T::FREQUENTLY_SPLIT,
        case_insensitive: T::CASE_INSENSITIVE,
        has_custom_validation: T::HAS_CUSTOM_VALIDATION,
        has_custom_normalization: T::HAS_CUSTOM_NORMALIZATION,
        default_separator: T::default_separator(),
        validation_help: T::validation_help(),
        examples: T::examples(),
    }
}

/// Check if two domains are compatible for key operations
///
/// This function checks if keys from two different domains can be safely
/// compared or used together in certain operations.
#[must_use]
pub fn domains_compatible<T1: KeyDomain, T2: KeyDomain>() -> bool {
    // Domains are compatible if they have the same basic characteristics
    T1::MAX_LENGTH == T2::MAX_LENGTH
        && T1::CASE_INSENSITIVE == T2::CASE_INSENSITIVE
        && T1::default_separator() == T2::default_separator()
}

// ============================================================================
// BUILT-IN DOMAIN IMPLEMENTATIONS
// ============================================================================

/// A simple default domain for general-purpose keys
///
/// This domain provides sensible defaults for most use cases:
/// - Alphanumeric characters plus underscore, hyphen, and dot
/// - Case-insensitive (normalized to lowercase)
/// - Maximum length of 64 characters
/// - No custom validation or normalization
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, DefaultDomain};
///
/// type DefaultKey = Key<DefaultDomain>;
///
/// let key = DefaultKey::new("example_key")?;
/// assert_eq!(key.as_str(), "example_key");
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefaultDomain;

impl KeyDomain for DefaultDomain {
    const DOMAIN_NAME: &'static str = "default";
    const MAX_LENGTH: usize = 64;
    const EXPECTED_LENGTH: usize = 24;
    const TYPICALLY_SHORT: bool = true;
    const CASE_INSENSITIVE: bool = true;

    fn validation_help() -> Option<&'static str> {
        Some("Use alphanumeric characters, underscores, hyphens, and dots. Case insensitive.")
    }

    fn examples() -> &'static [&'static str] {
        &["user_123", "session-abc", "cache.key", "simple"]
    }
}

/// A strict domain for identifiers that must follow strict naming rules
///
/// This domain is suitable for cases where keys must be valid identifiers
/// in programming languages or databases:
/// - Must start with a letter or underscore
/// - Can contain letters, numbers, and underscores only
/// - Case-sensitive
/// - No consecutive underscores
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, IdentifierDomain};
///
/// type IdKey = Key<IdentifierDomain>;
///
/// let key = IdKey::new("valid_identifier")?;
/// assert_eq!(key.as_str(), "valid_identifier");
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdentifierDomain;

impl KeyDomain for IdentifierDomain {
    const DOMAIN_NAME: &'static str = "identifier";
    const MAX_LENGTH: usize = 64;
    const EXPECTED_LENGTH: usize = 20;
    const TYPICALLY_SHORT: bool = true;
    const CASE_INSENSITIVE: bool = false;
    const HAS_CUSTOM_VALIDATION: bool = true;

    fn allowed_characters(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn allowed_start_character(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        if let Some(first) = key.chars().next() {
            if !Self::allowed_start_character(first) {
                return Err(KeyParseError::domain_error(
                    Self::DOMAIN_NAME,
                    "Identifier must start with a letter or underscore",
                ));
            }
        }
        Ok(())
    }

    fn validation_help() -> Option<&'static str> {
        Some("Must start with letter or underscore, contain only letters, numbers, and underscores. Case sensitive.")
    }

    fn examples() -> &'static [&'static str] {
        &["user_id", "session_key", "_private", "publicVar"]
    }
}

/// A domain for file path-like keys
///
/// This domain allows forward slashes and is suitable for hierarchical keys
/// that resemble file paths:
/// - Allows alphanumeric, underscore, hyphen, dot, and forward slash
/// - Case-insensitive
/// - No consecutive slashes
/// - Cannot start or end with slash
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, PathDomain};
///
/// type PathKey = Key<PathDomain>;
///
/// let key = PathKey::new("users/profile/settings")?;
/// assert_eq!(key.as_str(), "users/profile/settings");
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PathDomain;

impl KeyDomain for PathDomain {
    const DOMAIN_NAME: &'static str = "path";
    const MAX_LENGTH: usize = 256;
    const EXPECTED_LENGTH: usize = 48;
    const TYPICALLY_SHORT: bool = false;
    const CASE_INSENSITIVE: bool = true;
    const FREQUENTLY_SPLIT: bool = true;
    const HAS_CUSTOM_VALIDATION: bool = true;

    fn allowed_characters(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/'
    }

    fn allowed_start_character(c: char) -> bool {
        Self::allowed_characters(c) && c != '/'
    }

    fn allowed_end_character(c: char) -> bool {
        Self::allowed_characters(c) && c != '/'
    }

    fn allowed_consecutive_characters(prev: char, curr: char) -> bool {
        // Prevent consecutive slashes
        !(prev == '/' && curr == '/')
    }

    fn default_separator() -> char {
        '/'
    }

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        if key.starts_with('/') || key.ends_with('/') {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Path cannot start or end with '/'",
            ));
        }

        if key.contains("//") {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Path cannot contain consecutive '/'",
            ));
        }

        Ok(())
    }

    fn validation_help() -> Option<&'static str> {
        Some("Use path-like format with '/' separators. Cannot start/end with '/' or have consecutive '//'.")
    }

    fn examples() -> &'static [&'static str] {
        &["users/profile", "cache/session/data", "config/app.settings"]
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::borrow::Cow;
    #[cfg(not(feature = "std"))]
    use alloc::format;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(feature = "std")]
    use std::borrow::Cow;

    #[test]
    fn test_default_domain() {
        let info = domain_info::<DefaultDomain>();
        assert_eq!(info.name, "default");
        assert_eq!(info.max_length, 64);
        assert!(info.case_insensitive);
        assert!(!info.has_custom_validation);
    }

    #[test]
    fn test_identifier_domain() {
        let info = domain_info::<IdentifierDomain>();
        assert_eq!(info.name, "identifier");
        assert!(!info.case_insensitive);
        assert!(info.has_custom_validation);

        // Test character validation
        assert!(IdentifierDomain::allowed_characters('a'));
        assert!(IdentifierDomain::allowed_characters('_'));
        assert!(!IdentifierDomain::allowed_characters('-'));

        // Test start character validation
        assert!(IdentifierDomain::allowed_start_character('a'));
        assert!(IdentifierDomain::allowed_start_character('_'));
        assert!(!IdentifierDomain::allowed_start_character('1'));
    }

    #[test]
    fn test_path_domain() {
        let info = domain_info::<PathDomain>();
        assert_eq!(info.name, "path");
        assert_eq!(info.default_separator, '/');
        assert!(info.frequently_split);
        assert!(info.has_custom_validation);

        // Test character validation
        assert!(PathDomain::allowed_characters('/'));
        assert!(!PathDomain::allowed_start_character('/'));
        assert!(!PathDomain::allowed_end_character('/'));
        assert!(!PathDomain::allowed_consecutive_characters('/', '/'));
    }

    #[test]
    fn test_domain_info_display() {
        let info = domain_info::<DefaultDomain>();
        let display = format!("{info}");
        assert!(display.contains("Domain: default"));
        assert!(display.contains("Length: 1-64"));
        assert!(display.contains("Case insensitive: true"));
    }

    #[test]
    fn test_domains_compatible() {
        assert!(domains_compatible::<DefaultDomain, DefaultDomain>());
        assert!(!domains_compatible::<DefaultDomain, IdentifierDomain>());
        assert!(!domains_compatible::<IdentifierDomain, PathDomain>());
    }

    #[test]
    fn test_validation_methods() {
        // Test default implementations
        assert!(DefaultDomain::allowed_characters('a'));
        assert!(!DefaultDomain::is_reserved_prefix("test"));
        assert!(!DefaultDomain::is_reserved_suffix("test"));
        assert!(!DefaultDomain::requires_ascii_only("test"));
        assert_eq!(DefaultDomain::min_length(), 1);

        // Test validation help
        assert!(DefaultDomain::validation_help().is_some());
        assert!(!DefaultDomain::examples().is_empty());
    }

    #[test]
    fn test_normalization() {
        // Test default normalization (no change)
        let input = Cow::Borrowed("test");
        let output = DefaultDomain::normalize_domain(input);
        assert!(matches!(output, Cow::Borrowed("test")));

        // Test with owned string
        let input = Cow::Owned("test".to_string());
        let output = DefaultDomain::normalize_domain(input);
        assert!(matches!(output, Cow::Owned(_)));
    }
}
