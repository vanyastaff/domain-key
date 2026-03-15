//! Macros for convenient key creation and domain definition in domain-key
//!
//! This module provides helpful macros that simplify the creation and usage
//! of domain-specific keys, reducing boilerplate and improving ergonomics.

// ============================================================================
// STATIC KEY MACRO
// ============================================================================

/// Create a validated static key
///
/// This macro creates a static key with a compile-time emptiness check
/// and full runtime validation via `try_from_static`, which enforces the
/// domain's actual `MAX_LENGTH`. If validation fails, the macro **panics**.
///
/// # Arguments
///
/// * `$key_type` - The key type (e.g., `UserKey`)
/// * `$key_str` - The string literal for the key
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, Domain, KeyDomain, static_key};
///
/// #[derive(Debug)]
/// struct AdminDomain;
///
/// impl Domain for AdminDomain {
///     const DOMAIN_NAME: &'static str = "admin";
/// }
/// impl KeyDomain for AdminDomain {}
///
/// type AdminKey = Key<AdminDomain>;
///
/// // Basic checks at compile time, full validation at runtime (panics on failure)
/// let admin_key = static_key!(AdminKey, "system_admin");
/// assert_eq!(admin_key.as_str(), "system_admin");
/// ```
#[macro_export]
macro_rules! static_key {
    ($key_type:ty, $key_str:literal) => {{
        // Compile-time validation - check that the key is non-empty.
        // Length is validated against the domain's actual MAX_LENGTH at
        // runtime via try_from_static below; a compile-time check against
        // DEFAULT_MAX_KEY_LENGTH would be incorrect for domains that
        // override MAX_LENGTH with a smaller value.
        const _: () = {
            let bytes = $key_str.as_bytes();
            if bytes.is_empty() {
                panic!(concat!("Static key cannot be empty: ", $key_str));
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
/// * `$max_length` - Optional maximum length (defaults to `DEFAULT_MAX_KEY_LENGTH`)
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
    ($vis:vis $name:ident, $domain_name:literal) => {
        $crate::define_domain!($vis $name, $domain_name, $crate::DEFAULT_MAX_KEY_LENGTH);
    };

    ($vis:vis $name:ident, $domain_name:literal, $max_length:expr) => {
        #[derive(Debug)]
        $vis struct $name;

        impl $crate::Domain for $name {
            const DOMAIN_NAME: &'static str = $domain_name;
        }

        impl $crate::KeyDomain for $name {
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
    ($vis:vis $key_name:ident, $domain:ty) => {
        $vis type $key_name = $crate::Key<$domain>;
    };
}

// ============================================================================
// ID DOMAIN DEFINITION MACRO
// ============================================================================

/// Define an ID domain with minimal boilerplate
///
/// This macro simplifies the definition of ID domains by generating the
/// required trait implementations automatically.
///
/// # Arguments
///
/// * `$name` - The domain struct name
/// * `$domain_name` - The string name for the domain
///
/// # Examples
///
/// ```rust
/// use domain_key::{define_id_domain, Id};
///
/// define_id_domain!(UserIdDomain, "user");
/// type UserId = Id<UserIdDomain>;
///
/// let id = UserId::new(42).unwrap();
/// assert_eq!(id.domain(), "user");
/// ```
#[macro_export]
macro_rules! define_id_domain {
    // Without explicit name — uses stringify
    ($vis:vis $name:ident) => {
        $crate::define_id_domain!(@inner $vis $name, stringify!($name));
    };
    // With explicit string literal
    ($vis:vis $name:ident, $domain_name:literal) => {
        $crate::define_id_domain!(@inner $vis $name, $domain_name);
    };
    (@inner $vis:vis $name:ident, $domain_name:expr) => {
        #[derive(Debug)]
        $vis struct $name;

        impl $crate::Domain for $name {
            const DOMAIN_NAME: &'static str = $domain_name;
        }

        impl $crate::IdDomain for $name {}
    };
}

// ============================================================================
// UUID DOMAIN DEFINITION MACRO
// ============================================================================

/// Define a UUID domain with minimal boilerplate
///
/// This macro simplifies the definition of UUID domains by generating the
/// required trait implementations automatically.
///
/// Requires the `uuid` feature.
///
/// # Arguments
///
/// * `$name` - The domain struct name
/// * `$domain_name` - The string name for the domain
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "uuid")]
/// # {
/// use domain_key::{define_uuid_domain, Uuid};
///
/// define_uuid_domain!(OrderUuidDomain, "order");
/// type OrderUuid = Uuid<OrderUuidDomain>;
///
/// let id = OrderUuid::nil();
/// assert_eq!(id.domain(), "order");
/// # }
/// ```
#[cfg(feature = "uuid")]
#[macro_export]
macro_rules! define_uuid_domain {
    // Without explicit name — uses stringify
    ($vis:vis $name:ident) => {
        $crate::define_uuid_domain!(@inner $vis $name, stringify!($name));
    };
    // With explicit string literal
    ($vis:vis $name:ident, $domain_name:literal) => {
        $crate::define_uuid_domain!(@inner $vis $name, $domain_name);
    };
    (@inner $vis:vis $name:ident, $domain_name:expr) => {
        #[derive(Debug)]
        $vis struct $name;

        impl $crate::Domain for $name {
            const DOMAIN_NAME: &'static str = $domain_name;
        }

        impl $crate::UuidDomain for $name {}
    };
}

// ============================================================================
// ID TYPE ALIAS MACRO
// ============================================================================

/// Create an Id type alias
///
/// This macro creates a type alias for a numeric Id.
///
/// # Arguments
///
/// * `$id_name` - The name for the Id type alias
/// * `$domain` - The domain type (must implement `IdDomain`)
///
/// # Examples
///
/// ```rust
/// use domain_key::{define_id_domain, id_type};
///
/// define_id_domain!(UserIdDomain, "user");
/// id_type!(UserId, UserIdDomain);
///
/// let id = UserId::new(1).unwrap();
/// assert_eq!(id.get(), 1);
/// ```
#[macro_export]
macro_rules! id_type {
    ($vis:vis $id_name:ident, $domain:ty) => {
        $vis type $id_name = $crate::Id<$domain>;
    };
}

// ============================================================================
// UUID TYPE ALIAS MACRO
// ============================================================================

/// Create a Uuid type alias
///
/// This macro creates a type alias for a typed Uuid.
///
/// Requires the `uuid` feature.
///
/// # Arguments
///
/// * `$uuid_name` - The name for the Uuid type alias
/// * `$domain` - The domain type (must implement `UuidDomain`)
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "uuid")]
/// # {
/// use domain_key::{define_uuid_domain, uuid_type};
///
/// define_uuid_domain!(OrderUuidDomain, "order");
/// uuid_type!(OrderUuid, OrderUuidDomain);
///
/// let id = OrderUuid::nil();
/// assert!(id.is_nil());
/// # }
/// ```
#[cfg(feature = "uuid")]
#[macro_export]
macro_rules! uuid_type {
    ($vis:vis $uuid_name:ident, $domain:ty) => {
        $vis type $uuid_name = $crate::Uuid<$domain>;
    };
}

// ============================================================================
// COMBINED DOMAIN + TYPE ALIAS MACROS
// ============================================================================

/// Define an Id domain and type alias in one step
///
/// This is a convenience macro that combines [`define_id_domain!`] and [`id_type!`].
///
/// # Examples
///
/// ```rust
/// use domain_key::{define_id, Id};
///
/// define_id!(UserIdDomain => UserId);
///
/// let id = UserId::new(42).unwrap();
/// assert_eq!(id.domain(), "UserId");
/// ```
#[macro_export]
macro_rules! define_id {
    ($vis:vis $domain:ident => $alias:ident) => {
        $crate::define_id_domain!(@inner $vis $domain, stringify!($alias));
        $crate::id_type!($vis $alias, $domain);
    };
}

/// Define a Uuid domain and type alias in one step
///
/// This is a convenience macro that combines [`define_uuid_domain!`] and [`uuid_type!`].
///
/// Requires the `uuid` feature.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "uuid")]
/// # {
/// use domain_key::{define_uuid, Uuid};
///
/// define_uuid!(OrderUuidDomain => OrderUuid);
///
/// let id = OrderUuid::nil();
/// assert_eq!(id.domain(), "OrderUuid");
/// # }
/// ```
#[cfg(feature = "uuid")]
#[macro_export]
macro_rules! define_uuid {
    ($vis:vis $domain:ident => $alias:ident) => {
        $crate::define_uuid_domain!(@inner $vis $domain, stringify!($alias));
        $crate::uuid_type!($vis $alias, $domain);
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
        use $crate::__private::{Vec, ToString};
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
    // With explicit module name: test_domain!(MyDomain as my_domain_tests { ... })
    //
    // Use this form when invoking the macro more than once in the same module to
    // avoid the `mod domain_tests` name collision.
    ($domain:ty as $mod_name:ident {
        valid: [$($valid:literal),* $(,)?],
        invalid: [$($invalid:literal),* $(,)?] $(,)?
    }) => {
        #[cfg(test)]
        mod $mod_name {
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
                use $crate::Domain;
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

    // Without explicit module name (backward-compatible) — defaults to `domain_tests`.
    //
    // Note: Only one such invocation is allowed per module.  If you need a second
    // invocation in the same module, supply an explicit name with the `as` form above.
    ($domain:ty {
        valid: [$($valid:literal),* $(,)?],
        invalid: [$($invalid:literal),* $(,)?] $(,)?
    }) => {
        $crate::test_domain!($domain as domain_tests {
            valid: [$($valid),*],
            invalid: [$($invalid),*]
        });
    };
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::{Domain, Key, KeyDomain};
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    // Test define_domain macro
    define_domain!(MacroTestDomain, "macro_test");
    type MacroTestKey = Key<MacroTestDomain>;

    // Test define_domain with custom max length
    define_domain!(LongDomain, "long", 256);
    #[allow(dead_code)]
    type LongKey = Key<LongDomain>;

    #[test]
    fn define_domain_sets_name_and_max_length() {
        assert_eq!(MacroTestDomain::DOMAIN_NAME, "macro_test");
        assert_eq!(MacroTestDomain::MAX_LENGTH, crate::DEFAULT_MAX_KEY_LENGTH);

        assert_eq!(LongDomain::DOMAIN_NAME, "long");
        assert_eq!(LongDomain::MAX_LENGTH, 256);
    }

    #[test]
    fn static_key_validates_and_creates_key() {
        let key = static_key!(MacroTestKey, "static_test");
        assert_eq!(key.as_str(), "static_test");
        assert_eq!(key.domain(), "macro_test");
    }

    #[test]
    fn key_type_creates_usable_alias() {
        key_type!(TestKey, MacroTestDomain);
        let key = TestKey::new("test_key").unwrap();
        assert_eq!(key.as_str(), "test_key");
    }

    #[test]
    fn batch_keys_collects_all_valid_keys() {
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
    fn batch_keys_returns_errors_for_invalid_entries() {
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

    #[test]
    fn define_id_creates_domain_and_alias() {
        define_id!(TestIdDomain2 => TestId2);
        let id = TestId2::new(42).unwrap();
        assert_eq!(id.get(), 42);
        assert_eq!(id.domain(), "TestId2");
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn define_uuid_creates_domain_and_alias() {
        define_uuid!(TestUuidDomain2 => TestUuid2);
        let id = TestUuid2::nil();
        assert!(id.is_nil());
        assert_eq!(id.domain(), "TestUuid2");
    }

    // Test the test_domain macro - use it at module level
    #[cfg(test)]
    mod test_domain_macro_test {

        // Define a test domain specifically for this test
        define_domain!(pub TestMacroDomain, "test_macro");

        // Apply the test_domain macro
        test_domain!(TestMacroDomain {
            valid: ["valid_key", "another_valid", "key123",],
            invalid: ["",]
        });
    }
}
