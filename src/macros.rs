//! Macros for convenient key creation and domain definition in domain-key
//!
//! This module provides helpful macros that simplify the creation and usage
//! of domain-specific keys, reducing boilerplate and improving ergonomics.

// ============================================================================
// STATIC KEY MACRO
// ============================================================================

/// Create a compile-time validated static key
///
/// This macro creates a static key that is validated at compile time,
/// ensuring that the key string is valid for the specified domain.
///
/// # Arguments
///
/// * `$key_type` - The key type (e.g., `UserKey`)
/// * `$key_str` - The string literal for the key
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, KeyDomain, static_key};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct AdminDomain;
///
/// impl KeyDomain for AdminDomain {
///     const DOMAIN_NAME: &'static str = "admin";
/// }
///
/// type AdminKey = Key<AdminDomain>;
///
/// // This key is validated at compile time
/// let admin_key = static_key!(AdminKey, "system_admin");
/// assert_eq!(admin_key.as_str(), "system_admin");
/// ```
#[macro_export]
macro_rules! static_key {
    ($key_type:ty, $key_str:literal) => {{
        // Compile-time validation - check basic properties
        const _: () = {
            let bytes = $key_str.as_bytes();
            if bytes.is_empty() {
                panic!(concat!("Static key cannot be empty: ", $key_str));
            }
            if bytes.len() > $crate::DEFAULT_MAX_KEY_LENGTH {
                panic!(concat!("Static key too long: ", $key_str));
            }
        };

        // Use the safe validation method
        match <$key_type>::try_from_static($key_str) {
            Ok(key) => key,
            Err(e) => panic!("Invalid static key '{}': {}", $key_str, e),
        }
    }};
}

// ============================================================================
// DOMAIN DEFINITION MACRO
// ============================================================================

/// Define a key domain with minimal boilerplate
///
/// This macro simplifies the definition of key domains by generating the
/// required trait implementations automatically.
///
/// # Arguments
///
/// * `$name` - The domain struct name
/// * `$domain_name` - The string name for the domain
/// * `$max_length` - Optional maximum length (defaults to DEFAULT_MAX_KEY_LENGTH)
///
/// # Examples
///
/// ```rust
/// use domain_key::{define_domain, Key};
///
/// // Simple domain with default settings
/// define_domain!(UserDomain, "user");
/// type UserKey = Key<UserDomain>;
///
/// // Domain with custom max length
/// define_domain!(SessionDomain, "session", 128);
/// type SessionKey = Key<SessionDomain>;
///
/// let user = UserKey::new("john_doe")?;
/// let session = SessionKey::new("sess_abc123")?;
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
#[macro_export]
macro_rules! define_domain {
    ($name:ident, $domain_name:literal) => {
        define_domain!($name, $domain_name, $crate::DEFAULT_MAX_KEY_LENGTH);
    };

    ($name:ident, $domain_name:literal, $max_length:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name;

        impl $crate::KeyDomain for $name {
            const DOMAIN_NAME: &'static str = $domain_name;
            const MAX_LENGTH: usize = $max_length;
        }
    };
}

// ============================================================================
// KEY TYPE ALIAS MACRO
// ============================================================================

/// Create a key type alias
///
/// This macro creates a type alias for a key.
///
/// # Arguments
///
/// * `$key_name` - The name for the key type alias
/// * `$domain` - The domain type
///
/// # Examples
///
/// ```rust
/// use domain_key::{define_domain, key_type};
///
/// define_domain!(UserDomain, "user");
/// key_type!(UserKey, UserDomain);
///
/// let user = UserKey::new("john")?;
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
#[macro_export]
macro_rules! key_type {
    ($key_name:ident, $domain:ty) => {
        pub type $key_name = $crate::Key<$domain>;
    };
}

// ============================================================================
// BATCH KEY CREATION MACRO
// ============================================================================

/// Create multiple keys at once with error handling
///
/// This macro simplifies the creation of multiple keys from string literals
/// or expressions, with automatic error collection.
///
/// # Examples
///
/// ```rust
/// use domain_key::{define_domain, key_type, batch_keys};
///
/// define_domain!(UserDomain, "user");
/// key_type!(UserKey, UserDomain);
///
/// // Create multiple keys, collecting any errors
/// let result = batch_keys!(UserKey => [
///     "user_1",
///     "user_2",
///     "user_3",
/// ]);
///
/// match result {
///     Ok(keys) => println!("Created {} keys", keys.len()),
///     Err(errors) => println!("Failed to create {} keys", errors.len()),
/// }
/// ```
#[macro_export]
macro_rules! batch_keys {
    ($key_type:ty => [$($key_str:expr),* $(,)?]) => {{
        let mut keys = Vec::new();
        let mut errors = Vec::new();

        $(
            match <$key_type>::new($key_str) {
                Ok(key) => keys.push(key),
                Err(e) => errors.push(($key_str.to_string(), e)),
            }
        )*

        if errors.is_empty() {
            Ok(keys)
        } else {
            Err(errors)
        }
    }};
}

// ============================================================================
// TESTING HELPERS
// ============================================================================

/// Generate test cases for key domains
///
/// This macro creates comprehensive test cases for a domain,
/// testing both valid and invalid keys. The macro generates a `domain_tests`
/// submodule with test functions.
///
/// **Important**: This macro must be used at module level, not inside functions.
///
/// # Arguments
///
/// * `$domain` - The domain type to test
/// * `valid` - Array of string literals that should be valid keys
/// * `invalid` - Array of string literals that should be invalid keys
///
/// # Examples
///
/// ```rust
/// use domain_key::{define_domain, test_domain};
///
/// define_domain!(MyTestDomain, "test");
///
/// // This creates a `domain_tests` module with test functions
/// test_domain!(MyTestDomain {
///     valid: [
///         "valid_key",
///         "another_valid",
///         "key123",
///     ],
///     invalid: [
///         "",
///         "key with spaces",
///     ]
/// });
/// ```
///
/// The generated tests will:
/// - Test that all valid keys can be created successfully
/// - Test that all invalid keys fail to create with appropriate errors
/// - Test basic domain properties (name, max length, etc.)
///
/// Note: This macro should be used at module level, not inside functions.
#[macro_export]
macro_rules! test_domain {
    ($domain:ty {
        valid: [$($valid:literal),* $(,)?],
        invalid: [$($invalid:literal),* $(,)?] $(,)?
    }) => {
        #[cfg(test)]
        mod domain_tests {
            use super::*;

            type TestKey = $crate::Key<$domain>;

            #[test]
            fn test_valid_keys() {
                $(
                    let key = TestKey::new($valid);
                    assert!(key.is_ok(), "Key '{}' should be valid: {:?}", $valid, key.err());
                )*
            }

            #[test]
            fn test_invalid_keys() {
                $(
                    let key = TestKey::new($invalid);
                    assert!(key.is_err(), "Key '{}' should be invalid", $invalid);
                )*
            }

            #[test]
            fn test_domain_properties() {
                use $crate::KeyDomain;

                // Test domain constants
                assert!(!<$domain>::DOMAIN_NAME.is_empty());
                assert!(<$domain>::MAX_LENGTH > 0);

                // Test validation help if available
                if let Some(help) = <$domain>::validation_help() {
                    assert!(!help.is_empty());
                }
            }
        }
    };
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::{Key, KeyDomain, KeyParseError};

    // Test define_domain macro
    define_domain!(MacroTestDomain, "macro_test");
    type MacroTestKey = Key<MacroTestDomain>;

    // Test define_domain with custom max length
    define_domain!(LongDomain, "long", 256);
    type LongKey = Key<LongDomain>;

    #[test]
    fn test_define_domain_macro() {
        assert_eq!(MacroTestDomain::DOMAIN_NAME, "macro_test");
        assert_eq!(MacroTestDomain::MAX_LENGTH, crate::DEFAULT_MAX_KEY_LENGTH);

        assert_eq!(LongDomain::DOMAIN_NAME, "long");
        assert_eq!(LongDomain::MAX_LENGTH, 256);
    }

    #[test]
    fn test_static_key_macro() {
        let key = static_key!(MacroTestKey, "static_test");
        assert_eq!(key.as_str(), "static_test");
        assert_eq!(key.domain(), "macro_test");
    }

    #[test]
    fn test_key_type_macro() {
        key_type!(TestKey, MacroTestDomain);
        let key = TestKey::new("test_key").unwrap();
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn test_batch_keys_macro() {
        let result = batch_keys!(MacroTestKey => [
            "key1",
            "key2",
            "key3",
        ]);

        assert!(result.is_ok());
        let keys = result.unwrap();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0].as_str(), "key1");
        assert_eq!(keys[1].as_str(), "key2");
        assert_eq!(keys[2].as_str(), "key3");
    }

    #[test]
    fn test_batch_keys_with_errors() {
        let result = batch_keys!(MacroTestKey => [
            "valid_key",
            "", // This should fail
            "another_valid",
        ]);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "");
    }

    // Test the test_domain macro - use it at module level
    #[cfg(test)]
    mod test_domain_macro_test {
        use super::*;

        // Define a test domain specifically for this test
        define_domain!(TestMacroDomain, "test_macro");

        // Apply the test_domain macro
        test_domain!(TestMacroDomain {
            valid: [
                "valid_key",
                "another_valid",
                "key123",
            ],
            invalid: [
                "",
            ]
        });
    }
}