//! Utility functions and helper types for domain-key
//!
//! This module contains internal utility functions used throughout the library,
//! including optimized string operations, caching utilities, and performance helpers.

use smartstring::alias::String as SmartString;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(feature = "std")]
use std::borrow::Cow;

// ============================================================================
// STRING MANIPULATION UTILITIES
// ============================================================================

/// Add a prefix to a string with optimized allocation
///
/// This function efficiently adds a prefix to a string by pre-calculating
/// the required capacity and performing a single allocation.
///
/// # Arguments
///
/// * `key` - The original string
/// * `prefix` - The prefix to add
///
/// # Returns
///
/// A new `SmartString` with the prefix added
#[must_use]
pub fn add_prefix_optimized(key: &str, prefix: &str) -> SmartString {
    let total = prefix.len() + key.len();
    if total <= 23 {
        // Fits inline in SmartString — no heap allocation
        let mut result = SmartString::new();
        result.push_str(prefix);
        result.push_str(key);
        result
    } else {
        let mut s = String::with_capacity(total);
        s.push_str(prefix);
        s.push_str(key);
        SmartString::from(s)
    }
}

/// Add a suffix to a string with optimized allocation
///
/// This function efficiently adds a suffix to a string by pre-calculating
/// the required capacity and performing a single allocation.
///
/// # Arguments
///
/// * `key` - The original string
/// * `suffix` - The suffix to add
///
/// # Returns
///
/// A new `SmartString` with the suffix added
#[must_use]
pub fn add_suffix_optimized(key: &str, suffix: &str) -> SmartString {
    let total = key.len() + suffix.len();
    if total <= 23 {
        let mut result = SmartString::new();
        result.push_str(key);
        result.push_str(suffix);
        result
    } else {
        let mut s = String::with_capacity(total);
        s.push_str(key);
        s.push_str(suffix);
        SmartString::from(s)
    }
}

/// Create a new split cache for consistent API
///
/// This function creates a split iterator that can be used consistently
/// across different optimization levels.
///
/// # Arguments
///
/// * `s` - The string to split
/// * `delimiter` - The character to split on
///
/// # Returns
///
/// A split iterator over the string
#[must_use]
pub fn new_split_cache(s: &str, delimiter: char) -> core::str::Split<'_, char> {
    s.split(delimiter)
}

/// Join string parts with a delimiter, optimizing for common cases
///
/// This function efficiently joins string parts using pre-calculated sizing
/// to minimize allocations.
///
/// # Arguments
///
/// * `parts` - The string parts to join
/// * `delimiter` - The delimiter to use between parts
///
/// # Returns
///
/// A new string with all parts joined
#[must_use]
pub fn join_optimized(parts: &[&str], delimiter: &str) -> String {
    if parts.is_empty() {
        return String::new();
    }

    if parts.len() == 1 {
        return parts[0].to_string();
    }

    // Calculate total capacity needed
    let total_content_len: usize = parts.iter().map(|s| s.len()).sum();
    let delimiter_len = delimiter.len() * (parts.len().saturating_sub(1));
    let total_capacity = total_content_len + delimiter_len;

    let mut result = String::with_capacity(total_capacity);

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push_str(delimiter);
        }
        result.push_str(part);
    }

    result
}

/// Efficiently check if a string contains only ASCII characters
///
/// This function provides a fast path for ASCII-only validation.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string contains only ASCII characters
#[inline]
#[must_use]
pub fn is_ascii_only(s: &str) -> bool {
    s.is_ascii()
}

/// Count the number of occurrences of a character in a string
///
/// This function efficiently counts character occurrences without
/// allocating intermediate collections. Uses byte-level iteration
/// for ASCII characters.
///
/// # Arguments
///
/// * `s` - The string to search
/// * `target` - The character to count
///
/// # Returns
///
/// The number of times the character appears in the string
#[must_use]
pub fn count_char(s: &str, target: char) -> usize {
    if target.is_ascii() {
        let byte = target as u8;
        #[expect(clippy::naive_bytecount)]
        s.as_bytes().iter().filter(|&&b| b == byte).count()
    } else {
        s.chars().filter(|&c| c == target).count()
    }
}

/// Find the position of the nth occurrence of a character
///
/// This function finds the byte position of the nth occurrence of a character
/// in a string, useful for caching split positions.
///
/// # Arguments
///
/// * `s` - The string to search
/// * `target` - The character to find
/// * `n` - Which occurrence to find (0-based)
///
/// # Returns
///
/// The byte position of the nth occurrence, or `None` if not found
#[must_use]
pub fn find_nth_char(s: &str, target: char, n: usize) -> Option<usize> {
    let mut count = 0;
    for (pos, c) in s.char_indices() {
        if c == target {
            if count == n {
                return Some(pos);
            }
            count += 1;
        }
    }
    None
}

// ============================================================================
// NORMALIZATION UTILITIES
// ============================================================================

/// Trim whitespace and normalize case efficiently
///
/// This function combines trimming and case normalization in a single pass
/// when possible.
///
/// # Arguments
///
/// * `s` - The string to normalize
/// * `to_lowercase` - Whether to convert to lowercase
///
/// # Returns
///
/// A normalized string, borrowing when no changes are needed
#[must_use]
pub fn normalize_string(s: &str, to_lowercase: bool) -> Cow<'_, str> {
    let trimmed = s.trim();
    let needs_trim = trimmed.len() != s.len();
    let needs_lowercase = to_lowercase && trimmed.chars().any(|c| c.is_ascii_uppercase());

    match (needs_trim, needs_lowercase) {
        (false, false) => Cow::Borrowed(s),
        (true, false) => Cow::Owned(trimmed.to_string()),
        (_, true) => Cow::Owned(trimmed.to_ascii_lowercase()),
    }
}

/// Replace characters efficiently with a mapping function
///
/// This function applies character replacements without unnecessary allocations
/// when no replacements are needed. Uses a single-pass algorithm that borrows
/// when no changes are needed and only allocates on first replacement found.
///
/// # Arguments
///
/// * `s` - The input string
/// * `replacer` - Function that maps characters to their replacements
///
/// # Returns
///
/// A string with replacements applied, borrowing when no changes are needed
pub fn replace_chars<F>(s: &str, replacer: F) -> Cow<'_, str>
where
    F: Fn(char) -> Option<char>,
{
    // Single-pass: only allocate when we find the first replacement
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        if let Some(replacement) = replacer(c) {
            // Found first replacement — allocate and copy prefix, then continue
            let mut result = String::with_capacity(s.len());
            result.push_str(&s[..i]);
            result.push(replacement);
            for (_, c) in chars {
                if let Some(r) = replacer(c) {
                    result.push(r);
                } else {
                    result.push(c);
                }
            }
            return Cow::Owned(result);
        }
    }
    Cow::Borrowed(s)
}

// ============================================================================
// VALIDATION UTILITIES
// ============================================================================

/// Fast character class checking using lookup tables
///
/// This module provides optimized character validation functions using
/// precomputed lookup tables for common character classes.
#[expect(clippy::cast_possible_truncation)]
pub mod char_validation {
    /// Lookup table for ASCII alphanumeric characters
    const ASCII_ALPHANUMERIC: [bool; 128] = {
        let mut table = [false; 128];
        let mut i = 0;
        while i < 128 {
            table[i] = matches!(i as u8, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z');
            i += 1;
        }
        table
    };

    const KEY_CHARS: [bool; 128] = {
        let mut table = [false; 128];
        let mut i = 0;
        while i < 128 {
            table[i] =
                matches!(i as u8,  b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-' | b'.');
            i += 1;
        }
        table
    };

    /// Fast check if a character is ASCII alphanumeric
    #[inline]
    #[must_use]
    pub fn is_ascii_alphanumeric_fast(c: char) -> bool {
        if c.is_ascii() {
            ASCII_ALPHANUMERIC[c as u8 as usize]
        } else {
            false
        }
    }

    /// Fast check if a character is allowed in keys
    #[inline]
    #[must_use]
    pub fn is_key_char_fast(c: char) -> bool {
        if c.is_ascii() {
            KEY_CHARS[c as u8 as usize]
        } else {
            false
        }
    }

    /// Check if a character is a common separator
    #[inline]
    #[must_use]
    pub fn is_separator(c: char) -> bool {
        matches!(c, '_' | '-' | '.' | '/' | ':' | '|')
    }

    /// Check if a character is whitespace (space, tab, newline, etc.)
    #[inline]
    #[must_use]
    pub fn is_whitespace_fast(c: char) -> bool {
        matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C')
    }
}

// ============================================================================
// MEMORY UTILITIES
// ============================================================================

/// Calculate the memory usage of a string
///
/// This function calculates the total memory usage of a string, including
/// heap allocation overhead.
///
/// # Arguments
///
/// * `s` - The string to measure
///
/// # Returns
///
/// The estimated memory usage in bytes
#[must_use]
pub fn string_memory_usage(s: &str) -> usize {
    // Base string object size + heap allocation (if any)
    core::mem::size_of::<String>() + s.len()
}

/// Calculate the memory usage of a `SmartString`
///
/// This function calculates the memory usage of a `SmartString`, accounting
/// for inline vs heap storage.
///
/// # Arguments
///
/// * `s` - The string content to measure
///
/// # Returns
///
/// The estimated memory usage in bytes
#[must_use]
pub fn smart_string_memory_usage(s: &str) -> usize {
    // SmartString uses inline storage for strings <= 23 bytes
    if s.len() <= 23 {
        core::mem::size_of::<SmartString>()
    } else {
        core::mem::size_of::<SmartString>() + s.len()
    }
}

// ============================================================================
// FEATURE DETECTION
// ============================================================================

/// Returns the name of the active hash algorithm
///
/// The algorithm is selected at compile time based on feature flags:
/// - `fast` — `GxHash` (requires AES-NI), falls back to `AHash`
/// - `secure` — `AHash` (`DoS`-resistant)
/// - `crypto` — Blake3 (cryptographic)
/// - default — `DefaultHasher` (std) or FNV-1a (`no_std`)
///
/// # Examples
///
/// ```rust
/// let algo = domain_key::hash_algorithm();
/// println!("Using hash algorithm: {algo}");
/// ```
#[must_use]
pub const fn hash_algorithm() -> &'static str {
    #[cfg(feature = "fast")]
    {
        #[cfg(any(
            all(target_arch = "x86_64", target_feature = "aes"),
            all(target_arch = "aarch64", target_feature = "aes")
        ))]
        return "GxHash";

        #[cfg(not(any(
            all(target_arch = "x86_64", target_feature = "aes"),
            all(target_arch = "aarch64", target_feature = "aes")
        )))]
        return "AHash (GxHash fallback)";
    }

    #[cfg(all(feature = "secure", not(feature = "fast")))]
    return "AHash";

    #[cfg(all(feature = "crypto", not(any(feature = "fast", feature = "secure"))))]
    return "Blake3";

    #[cfg(not(any(feature = "fast", feature = "secure", feature = "crypto")))]
    {
        #[cfg(feature = "std")]
        return "DefaultHasher";

        #[cfg(not(feature = "std"))]
        return "FNV-1a";
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ValidationResult;
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    #[cfg(not(feature = "std"))]
    use alloc::vec::Vec;

    #[test]
    fn test_add_prefix_suffix() {
        let result = add_prefix_optimized("test", "prefix_");
        assert_eq!(result, "prefix_test");

        let result = add_suffix_optimized("test", "_suffix");
        assert_eq!(result, "test_suffix");
    }

    #[test]
    fn test_join_optimized() {
        let parts = vec!["a", "b", "c"];
        let result = join_optimized(&parts, "_");
        assert_eq!(result, "a_b_c");

        let empty: Vec<&str> = vec![];
        let result = join_optimized(&empty, "_");
        assert_eq!(result, "");

        let single = vec!["alone"];
        let result = join_optimized(&single, "_");
        assert_eq!(result, "alone");
    }

    #[test]
    fn test_char_validation() {
        use char_validation::*;

        assert!(is_ascii_alphanumeric_fast('a'));
        assert!(is_ascii_alphanumeric_fast('Z'));
        assert!(is_ascii_alphanumeric_fast('5'));
        assert!(!is_ascii_alphanumeric_fast('_'));
        assert!(!is_ascii_alphanumeric_fast('ñ'));

        assert!(is_key_char_fast('a'));
        assert!(is_key_char_fast('_'));
        assert!(is_key_char_fast('-'));
        assert!(is_key_char_fast('.'));
        assert!(!is_key_char_fast(' '));

        assert!(is_separator('_'));
        assert!(is_separator('/'));
        assert!(!is_separator('a'));

        assert!(is_whitespace_fast(' '));
        assert!(is_whitespace_fast('\t'));
        assert!(!is_whitespace_fast('a'));
    }

    #[test]
    fn test_string_utilities() {
        assert!(is_ascii_only("hello"));
        assert!(!is_ascii_only("héllo"));

        assert_eq!(count_char("hello_world_test", '_'), 2);
        assert_eq!(count_char("no_underscores", '_'), 1);

        assert_eq!(find_nth_char("a_b_c_d", '_', 0), Some(1));
        assert_eq!(find_nth_char("a_b_c_d", '_', 1), Some(3));
        assert_eq!(find_nth_char("a_b_c_d", '_', 2), Some(5));
        assert_eq!(find_nth_char("a_b_c_d", '_', 3), None);
    }

    #[test]
    fn test_normalize_string() {
        let result = normalize_string("  Hello  ", true);
        assert_eq!(result, "hello");

        let result = normalize_string("hello", true);
        assert_eq!(result, "hello");

        let result = normalize_string("  hello  ", false);
        assert_eq!(result, "hello");

        let result = normalize_string("hello", false);
        assert!(matches!(result, Cow::Borrowed("hello")));
    }

    #[test]
    fn test_memory_utilities() {
        let s = "hello";
        let usage = string_memory_usage(s);
        assert!(usage >= s.len());
    }

    #[test]
    fn test_float_comparison() {
        const EPSILON: f64 = 1e-10;
        let result = ValidationResult {
            total_processed: 2,
            valid: vec!["key1".to_string(), "key2".to_string()],
            errors: vec![],
        };

        // Use approximate comparison for floats

        assert!((result.success_rate() - 100.0).abs() < EPSILON);
    }

    #[test]
    fn test_replace_chars() {
        let result = replace_chars("hello-world", |c| if c == '-' { Some('_') } else { None });
        assert_eq!(result, "hello_world");

        let result = replace_chars("hello_world", |c| if c == '-' { Some('_') } else { None });
        assert!(matches!(result, Cow::Borrowed("hello_world")));
    }

    #[test]
    fn test_replace_chars_fixed() {
        let result = replace_chars("hello-world", |c| if c == '-' { Some('_') } else { None });
        assert_eq!(result, "hello_world");

        let result = replace_chars("hello_world", |c| if c == '-' { Some('_') } else { None });
        assert!(matches!(result, Cow::Borrowed("hello_world")));

        // Test with multiple replacements
        let result = replace_chars("a-b-c", |c| if c == '-' { Some('_') } else { None });
        assert_eq!(result, "a_b_c");

        // Test with no replacements needed
        let result = replace_chars("hello", |c| if c == 'x' { Some('y') } else { None });
        assert!(matches!(result, Cow::Borrowed(_)));

        // Test empty string
        let result = replace_chars("", |c| if c == 'x' { Some('y') } else { None });
        assert_eq!(result, "");
    }
}
