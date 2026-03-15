//! Error types and error handling for domain-key
//!
//! This module provides comprehensive error handling for key validation and creation.
//! All errors are designed to provide detailed information for debugging while maintaining
//! performance in the happy path.

use core::fmt;
use thiserror::Error;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::fmt::Write;

// ============================================================================
// CORE ERROR TYPES
// ============================================================================

/// Comprehensive error type for key parsing and validation failures
///
/// This enum covers all possible validation failures that can occur during
/// key creation, providing detailed information for debugging and user feedback.
///
/// # Error Categories
///
/// - **Length Errors**: Empty keys or keys exceeding maximum length
/// - **Character Errors**: Invalid characters at specific positions
/// - **Structure Errors**: Invalid patterns like consecutive special characters
/// - **Domain Errors**: Domain-specific validation failures
/// - **Custom Errors**: Application-specific validation failures
///
/// # Examples
///
/// ```rust
/// use domain_key::{KeyParseError, ErrorCategory};
///
/// // Handle different error types
/// match KeyParseError::Empty {
///     err => {
///         println!("Error: {}", err);
///         println!("Code: {}", err.code());
///         println!("Category: {:?}", err.category());
///     }
/// }
/// ```
#[derive(Debug, Error, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum KeyParseError {
    /// Key cannot be empty or contain only whitespace
    ///
    /// This error occurs when attempting to create a key from an empty string
    /// or a string containing only whitespace characters.
    #[error("Key cannot be empty or whitespace")]
    Empty,

    /// Key contains a character that is not allowed at the specified position
    ///
    /// Each domain defines which characters are allowed. This error provides
    /// the specific character, its position, and optionally what was expected.
    #[error("Invalid character '{character}' at position {position}")]
    InvalidCharacter {
        /// The invalid character that was found
        character: char,
        /// Position where the invalid character was found (0-based)
        position: usize,
        /// Optional description of what characters are expected
        expected: Option<&'static str>,
    },

    /// Key exceeds the maximum allowed length for the domain
    ///
    /// Each domain can specify a maximum length. This error provides both
    /// the limit and the actual length that was attempted.
    #[error("Key is too long (max {max_length} characters, got {actual_length})")]
    TooLong {
        /// The maximum allowed length for this domain
        max_length: usize,
        /// The actual length of the key that was attempted
        actual_length: usize,
    },

    /// Key is shorter than the minimum allowed length for the domain
    ///
    /// Each domain can specify a minimum length. This error provides both
    /// the required minimum and the actual length that was attempted.
    #[error("Key is too short (min {min_length} characters, got {actual_length})")]
    TooShort {
        /// The minimum allowed length for this domain
        min_length: usize,
        /// The actual length of the key that was attempted
        actual_length: usize,
    },

    /// Key has invalid structure (consecutive special chars, invalid start/end)
    ///
    /// This covers structural issues like:
    /// - Starting or ending with special characters
    /// - Consecutive special characters
    /// - Invalid character sequences
    #[error("Key has invalid structure: {reason}")]
    InvalidStructure {
        /// Description of the structural issue
        reason: &'static str,
    },

    /// Domain-specific validation error
    ///
    /// This error is returned when domain-specific validation rules fail.
    /// It includes the domain name and a descriptive message.
    #[error("Domain '{domain}' validation failed: {message}")]
    DomainValidation {
        /// The domain name where validation failed
        domain: &'static str,
        /// The error message describing what validation failed
        message: String,
    },

    /// Custom error for specific use cases
    ///
    /// Applications can define custom validation errors with numeric codes
    /// for structured error handling.
    #[error("Custom validation error (code: {code}): {message}")]
    Custom {
        /// Custom error code for programmatic handling
        code: u32,
        /// The custom error message
        message: String,
    },
}

impl KeyParseError {
    /// Create a domain validation error with domain name
    ///
    /// This is the preferred way to create domain validation errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::KeyParseError;
    ///
    /// let error = KeyParseError::domain_error("my_domain", "Custom validation failed");
    /// // Verify it's the correct error type
    /// match error {
    ///     KeyParseError::DomainValidation { domain, message } => {
    ///         assert_eq!(domain, "my_domain");
    ///         assert!(message.contains("Custom validation failed"));
    ///     },
    ///     _ => panic!("Expected domain validation error"),
    /// }
    /// ```
    pub fn domain_error(domain: &'static str, message: impl Into<String>) -> Self {
        Self::DomainValidation {
            domain,
            message: message.into(),
        }
    }

    /// Create a domain validation error without specifying domain (for internal use)
    pub fn domain_error_generic(message: impl Into<String>) -> Self {
        Self::DomainValidation {
            domain: "unknown",
            message: message.into(),
        }
    }

    /// Create a domain validation error with source error information
    ///
    /// The source error's `Display` representation is appended to `message`,
    /// separated by `": "`.  This is a **flattened** representation — the
    /// resulting `KeyParseError` does **not** implement `std::error::Error::source()`
    /// chaining back to the original error.  In other words, `error.source()`
    /// will return `None`, and tools such as `anyhow` / `tracing` that walk the
    /// error chain will not see the wrapped cause.
    ///
    /// If you need a proper causal chain, wrap the original error in your own
    /// error type (e.g. via `anyhow::Error` or a `thiserror` wrapper) before
    /// converting it to a `KeyParseError`.
    ///
    /// # Limitation
    ///
    /// Properly storing a `Box<dyn Error + Send + Sync>` inside `KeyParseError`
    /// variants would require either a new variant or a breaking field change.
    /// Until that refactor is done, the full error context is preserved only in
    /// the formatted message string.
    #[cfg(feature = "std")]
    pub fn domain_error_with_source(
        domain: &'static str,
        message: impl Into<String>,
        source: &(dyn std::error::Error + Send + Sync),
    ) -> Self {
        let full_message = format!("{}: {}", message.into(), source);
        Self::DomainValidation {
            domain,
            message: full_message,
        }
    }

    /// Create a custom validation error
    ///
    /// Custom errors allow applications to define their own error codes
    /// for structured error handling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::KeyParseError;
    ///
    /// let error = KeyParseError::custom(1001, "Business rule violation");
    /// assert_eq!(error.code(), 1001);
    /// ```
    pub fn custom(code: u32, message: impl Into<String>) -> Self {
        Self::Custom {
            code,
            message: message.into(),
        }
    }

    /// Create a custom validation error with source error information
    ///
    /// The source error's `Display` representation is appended to `message`,
    /// separated by `": "`.  This is a **flattened** representation — the
    /// resulting `KeyParseError` does **not** implement `std::error::Error::source()`
    /// chaining back to the original error.  In other words, `error.source()`
    /// will return `None`, and tools such as `anyhow` / `tracing` that walk the
    /// error chain will not see the wrapped cause.
    ///
    /// If you need a proper causal chain, wrap the original error in your own
    /// error type (e.g. via `anyhow::Error` or a `thiserror` wrapper) before
    /// converting it to a `KeyParseError`.
    ///
    /// # Limitation
    ///
    /// Properly storing a `Box<dyn Error + Send + Sync>` inside `KeyParseError`
    /// variants would require either a new variant or a breaking field change.
    /// Until that refactor is done, the full error context is preserved only in
    /// the formatted message string.
    #[cfg(feature = "std")]
    pub fn custom_with_source(
        code: u32,
        message: impl Into<String>,
        source: &(dyn std::error::Error + Send + Sync),
    ) -> Self {
        let full_message = format!("{}: {}", message.into(), source);
        Self::Custom {
            code,
            message: full_message,
        }
    }

    /// Get the error code for machine processing
    ///
    /// Returns a numeric code that can be used for programmatic error handling.
    /// This is useful for APIs that need to return structured error responses.
    ///
    /// # Error Codes
    ///
    /// - `1001`: Empty key
    /// - `1002`: Invalid character
    /// - `1003`: Key too long
    /// - `1004`: Invalid structure
    /// - `2000`: Domain validation (base code)
    /// - Custom codes: As specified in `Custom` errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::KeyParseError;
    ///
    /// assert_eq!(KeyParseError::Empty.code(), 1001);
    /// assert_eq!(KeyParseError::custom(42, "test").code(), 42);
    /// ```
    #[must_use]
    pub const fn code(&self) -> u32 {
        match self {
            Self::Empty => 1001,
            Self::InvalidCharacter { .. } => 1002,
            Self::TooLong { .. } => 1003,
            Self::InvalidStructure { .. } => 1004,
            Self::TooShort { .. } => 1005,
            Self::DomainValidation { .. } => 2000,
            Self::Custom { code, .. } => *code,
        }
    }

    /// Get the error category for classification
    ///
    /// Returns the general category of this error for higher-level error handling.
    /// This allows applications to handle broad categories of errors uniformly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use domain_key::{KeyParseError, ErrorCategory};
    ///
    /// match KeyParseError::Empty.category() {
    ///     ErrorCategory::Length => println!("Length-related error"),
    ///     ErrorCategory::Character => println!("Character-related error"),
    ///     _ => println!("Other error type"),
    /// }
    /// ```
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::Empty | Self::TooLong { .. } | Self::TooShort { .. } => ErrorCategory::Length,
            Self::InvalidCharacter { .. } => ErrorCategory::Character,
            Self::InvalidStructure { .. } => ErrorCategory::Structure,
            Self::DomainValidation { .. } => ErrorCategory::Domain,
            Self::Custom { code, .. } => match code {
                1002 => ErrorCategory::Character,
                1003 => ErrorCategory::Length,
                1004 => ErrorCategory::Structure,
                _ => ErrorCategory::Custom,
            },
        }
    }

    /// Get a human-readable description of what went wrong
    ///
    /// This provides additional context beyond the basic error message,
    /// useful for user-facing error messages or debugging.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Empty => "Key cannot be empty or contain only whitespace characters",
            Self::InvalidCharacter { .. } => {
                "Key contains characters that are not allowed by the domain"
            }
            Self::TooLong { .. } => "Key exceeds the maximum length allowed by the domain",
            Self::TooShort { .. } => {
                "Key is shorter than the minimum length required by the domain"
            }
            Self::InvalidStructure { .. } => "Key has invalid structure or formatting",
            Self::DomainValidation { .. } => "Key fails domain-specific validation rules",
            Self::Custom { .. } => "Key fails custom validation rules",
        }
    }

    /// Get suggested actions for fixing this error
    ///
    /// Returns helpful suggestions for how to fix the validation error.
    #[must_use]
    pub fn suggestions(&self) -> &'static [&'static str] {
        match self {
            Self::Empty => &[
                "Provide a non-empty key",
                "Remove leading/trailing whitespace",
            ],
            Self::InvalidCharacter { .. } => &[
                "Use only allowed characters (check domain rules)",
                "Remove or replace invalid characters",
            ],
            Self::TooLong { .. } => &[
                "Shorten the key to fit within length limits",
                "Consider using abbreviated forms",
            ],
            Self::TooShort { .. } => &["Lengthen the key to meet the minimum length requirement"],
            Self::InvalidStructure { .. } => &[
                "Avoid consecutive special characters",
                "Don't start or end with special characters",
                "Follow the expected key format",
            ],
            Self::DomainValidation { .. } => &[
                "Check domain-specific validation rules",
                "Refer to domain documentation",
            ],
            Self::Custom { .. } => &[
                "Check application-specific validation rules",
                "Contact system administrator if needed",
            ],
        }
    }

    /// Check if this error is recoverable through user action
    ///
    /// Returns `true` if the user can potentially fix this error by modifying
    /// their input, `false` if it represents a programming error or system issue.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::Empty
            | Self::InvalidCharacter { .. }
            | Self::TooLong { .. }
            | Self::TooShort { .. }
            | Self::InvalidStructure { .. }
            | Self::DomainValidation { .. } => true,
            Self::Custom { .. } => false, // Depends on the specific custom error
        }
    }
}

// ============================================================================
// ERROR CATEGORIES
// ============================================================================

/// Error category for classification of validation errors
///
/// These categories allow applications to handle broad types of validation
/// errors uniformly, regardless of the specific error details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorCategory {
    /// Length-related errors (empty, too long)
    Length,
    /// Character-related errors (invalid characters)
    Character,
    /// Structure-related errors (invalid format, consecutive special chars)
    Structure,
    /// Domain-specific validation errors
    Domain,
    /// Custom validation errors
    Custom,
}

impl ErrorCategory {
    /// Get a human-readable name for this category
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Length => "Length",
            Self::Character => "Character",
            Self::Structure => "Structure",
            Self::Domain => "Domain",
            Self::Custom => "Custom",
        }
    }

    /// Get a description of what this category represents
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Length => "Errors related to key length (empty, too long, etc.)",
            Self::Character => "Errors related to invalid characters in the key",
            Self::Structure => "Errors related to key structure and formatting",
            Self::Domain => "Errors from domain-specific validation rules",
            Self::Custom => "Custom application-specific validation errors",
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ============================================================================
// ID PARSE ERROR
// ============================================================================

/// Error type for numeric ID parsing failures
///
/// This error is returned when a string cannot be parsed as a valid `Id<D>`.
///
/// # Examples
///
/// ```rust
/// use domain_key::IdParseError;
///
/// let result = "not_a_number".parse::<u64>();
/// assert!(result.is_err());
/// ```
#[derive(Debug, Error, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum IdParseError {
    /// The value is zero, which is not a valid identifier
    #[error("ID cannot be zero")]
    Zero,

    /// The string could not be parsed as a valid non-zero u64 number
    #[error("Invalid numeric ID: {0}")]
    InvalidNumber(#[from] core::num::ParseIntError),
}

impl IdParseError {
    /// Returns a user-friendly error message suitable for display.
    #[must_use]
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Zero => "Identifier cannot be zero",
            Self::InvalidNumber(_) => "Value must be a positive integer",
        }
    }
}

// ============================================================================
// UUID PARSE ERROR
// ============================================================================

/// Error type for UUID parsing failures
///
/// This error is returned when a string cannot be parsed as a valid `Uuid<D>`.
/// Only available when the `uuid` feature is enabled.
///
/// Note: `UuidParseError` does not implement `PartialEq` because
/// `uuid::Error` does not implement it.
#[cfg(feature = "uuid")]
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
pub enum UuidParseError {
    /// The string could not be parsed as a valid UUID
    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] ::uuid::Error),
}

// ============================================================================
// ERROR UTILITIES
// ============================================================================

/// Builder for creating detailed validation errors
///
/// This builder provides a fluent interface for creating complex validation
/// errors with additional context and suggestions.
#[derive(Debug)]
pub struct ErrorBuilder {
    category: ErrorCategory,
    code: Option<u32>,
    domain: Option<&'static str>,
    message: String,
    context: Option<String>,
}

impl ErrorBuilder {
    /// Create a new error builder for the given category
    #[must_use]
    pub fn new(category: ErrorCategory) -> Self {
        Self {
            category,
            code: None,
            domain: None,
            message: String::new(),
            context: None,
        }
    }

    /// Set the error message
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set a custom error code (used when category is `Custom`)
    #[must_use]
    pub fn code(mut self, code: u32) -> Self {
        self.code = Some(code);
        self
    }

    /// Set the domain name (used when category is `Domain`)
    #[must_use]
    pub fn domain(mut self, domain: &'static str) -> Self {
        self.domain = Some(domain);
        self
    }

    /// Set additional context information
    #[must_use]
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Build the final error
    #[must_use]
    pub fn build(self) -> KeyParseError {
        let message = if let Some(context) = self.context {
            format!("{} (Context: {})", self.message, context)
        } else {
            self.message
        };

        match self.category {
            ErrorCategory::Custom => KeyParseError::custom(self.code.unwrap_or(0), message),
            ErrorCategory::Domain => {
                KeyParseError::domain_error(self.domain.unwrap_or("unknown"), message)
            }
            // Use the reserved structural codes so that category() round-trips
            // correctly back to the originally-specified category.
            // 1004 → ErrorCategory::Structure, 1003 → ErrorCategory::Length,
            // 1002 → ErrorCategory::Character  (mirroring code() for each variant)
            ErrorCategory::Structure => KeyParseError::custom(1004, message),
            ErrorCategory::Length => KeyParseError::custom(1003, message),
            ErrorCategory::Character => KeyParseError::custom(1002, message),
        }
    }
}

// ============================================================================
// ERROR FORMATTING UTILITIES
// ============================================================================

/// Format an error for display to end users
///
/// This function provides a user-friendly representation of validation errors,
/// including suggestions for how to fix them.
#[must_use]
pub fn format_user_error(error: &KeyParseError) -> String {
    let mut output = format!("Error: {error}");

    let suggestions = error.suggestions();
    if !suggestions.is_empty() {
        output.push_str("\n\nSuggestions:");
        for suggestion in suggestions {
            write!(output, "\n  - {suggestion}").unwrap();
        }
    }

    output
}

/// Format an error for logging or debugging
///
/// This function provides a detailed representation suitable for logs,
/// including error codes and categories.
#[must_use]
pub fn format_debug_error(error: &KeyParseError) -> String {
    format!(
        "[{}:{}] {} (Category: {})",
        error.code(),
        error.category().name(),
        error,
        error.description()
    )
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    #[test]
    fn each_variant_has_unique_error_code() {
        assert_eq!(KeyParseError::Empty.code(), 1001);
        assert_eq!(
            KeyParseError::InvalidCharacter {
                character: 'x',
                position: 0,
                expected: None
            }
            .code(),
            1002
        );
        assert_eq!(
            KeyParseError::TooLong {
                max_length: 10,
                actual_length: 20
            }
            .code(),
            1003
        );
        assert_eq!(
            KeyParseError::InvalidStructure { reason: "test" }.code(),
            1004
        );
        assert_eq!(
            KeyParseError::TooShort {
                min_length: 5,
                actual_length: 2
            }
            .code(),
            1005
        );
        assert_eq!(
            KeyParseError::DomainValidation {
                domain: "test",
                message: "msg".to_string()
            }
            .code(),
            2000
        );
        assert_eq!(
            KeyParseError::Custom {
                code: 42,
                message: "msg".to_string()
            }
            .code(),
            42
        );
    }

    #[test]
    fn variants_map_to_correct_category() {
        assert_eq!(KeyParseError::Empty.category(), ErrorCategory::Length);
        assert_eq!(
            KeyParseError::InvalidCharacter {
                character: 'x',
                position: 0,
                expected: None
            }
            .category(),
            ErrorCategory::Character
        );
        assert_eq!(
            KeyParseError::TooLong {
                max_length: 10,
                actual_length: 20
            }
            .category(),
            ErrorCategory::Length
        );
        assert_eq!(
            KeyParseError::InvalidStructure { reason: "test" }.category(),
            ErrorCategory::Structure
        );
        assert_eq!(
            KeyParseError::TooShort {
                min_length: 5,
                actual_length: 2
            }
            .category(),
            ErrorCategory::Length
        );
        assert_eq!(
            KeyParseError::DomainValidation {
                domain: "test",
                message: "msg".to_string()
            }
            .category(),
            ErrorCategory::Domain
        );
        assert_eq!(
            KeyParseError::Custom {
                code: 42,
                message: "msg".to_string()
            }
            .category(),
            ErrorCategory::Custom
        );
    }

    #[test]
    fn empty_error_provides_recovery_suggestions() {
        let error = KeyParseError::Empty;
        let suggestions = error.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("non-empty")));
    }

    #[test]
    fn builder_produces_custom_error_with_code_and_context() {
        let error = ErrorBuilder::new(ErrorCategory::Custom)
            .message("Test error")
            .code(1234)
            .context("In test function")
            .build();

        assert_eq!(error.code(), 1234);
        assert_eq!(error.category(), ErrorCategory::Custom);
    }

    #[test]
    fn variants_carry_correct_payloads() {
        let error1 = KeyParseError::InvalidCharacter {
            character: '!',
            position: 5,
            expected: Some("alphanumeric"),
        };
        assert!(matches!(
            error1,
            KeyParseError::InvalidCharacter {
                character: '!',
                position: 5,
                ..
            }
        ));

        let error2 = KeyParseError::TooLong {
            max_length: 32,
            actual_length: 64,
        };
        assert!(matches!(
            error2,
            KeyParseError::TooLong {
                max_length: 32,
                actual_length: 64
            }
        ));

        let error3 = KeyParseError::InvalidStructure {
            reason: "consecutive underscores",
        };
        assert!(matches!(
            error3,
            KeyParseError::InvalidStructure {
                reason: "consecutive underscores"
            }
        ));

        let error4 = KeyParseError::domain_error("test", "Invalid format");
        assert!(matches!(
            error4,
            KeyParseError::DomainValidation { domain: "test", .. }
        ));
    }

    #[test]
    fn user_and_debug_formats_include_expected_sections() {
        let error = KeyParseError::Empty;
        let user_format = format_user_error(&error);
        let debug_format = format_debug_error(&error);

        assert!(user_format.contains("Error:"));
        assert!(user_format.contains("Suggestions:"));
        assert!(debug_format.contains("1001"));
        assert!(debug_format.contains("Length"));
    }

    #[test]
    fn recoverable_errors_distinguished_from_non_recoverable() {
        assert!(KeyParseError::Empty.is_recoverable());
        assert!(KeyParseError::InvalidCharacter {
            character: 'x',
            position: 0,
            expected: None
        }
        .is_recoverable());
        assert!(KeyParseError::TooShort {
            min_length: 5,
            actual_length: 2
        }
        .is_recoverable());
        assert!(!KeyParseError::Custom {
            code: 42,
            message: "msg".to_string()
        }
        .is_recoverable());
    }

    #[test]
    fn category_display_and_description_are_populated() {
        assert_eq!(ErrorCategory::Length.to_string(), "Length");
        assert_eq!(ErrorCategory::Character.name(), "Character");
        assert!(ErrorCategory::Domain
            .description()
            .contains("domain-specific"));
    }
}
