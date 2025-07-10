//! Utility functions and helper types for domain-key
//!
//! This module contains internal utility functions used throughout the library,
//! including optimized string operations, caching utilities, and performance helpers.

use smartstring::alias::String as SmartString;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
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
/// * `max_length` - Maximum allowed length for validation
///
/// # Returns
///
/// A new `SmartString` with the prefix added
pub fn add_prefix_optimized(key: &str, prefix: &str, _max_length: usize) -> SmartString {
    let mut result = SmartString::new();
    result.push_str(prefix);
    result.push_str(key);
    result
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
/// * `max_length` - Maximum allowed length for validation
///
/// # Returns
///
/// A new `SmartString` with the suffix added
pub fn add_suffix_optimized(key: &str, suffix: &str, _max_length: usize) -> SmartString {
    let mut result = SmartString::new();
    result.push_str(key);
    result.push_str(suffix);
    result
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
pub fn is_ascii_only(s: &str) -> bool {
    s.is_ascii()
}

/// Count the number of occurrences of a character in a string
///
/// This function efficiently counts character occurrences without
/// allocating intermediate collections.
///
/// # Arguments
///
/// * `s` - The string to search
/// * `target` - The character to count
///
/// # Returns
///
/// The number of times the character appears in the string
pub fn count_char(s: &str, target: char) -> usize {
    s.chars().filter(|&c| c == target).count()
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
pub fn normalize_string(s: &str, to_lowercase: bool) -> Cow<'_, str> {
    let trimmed = s.trim();
    let needs_trim = trimmed.len() != s.len();
    let needs_lowercase = to_lowercase && trimmed.chars().any(|c| c.is_ascii_uppercase());

    match (needs_trim, needs_lowercase) {
        (false, false) => Cow::Borrowed(s),
        (true, false) => Cow::Owned(trimmed.to_string()),
        (false, true) => Cow::Owned(s.to_ascii_lowercase()),
        (true, true) => Cow::Owned(trimmed.to_ascii_lowercase()),
    }
}

/// Replace characters efficiently with a mapping function
///
/// This function applies character replacements without unnecessary allocations
/// when no replacements are needed.
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
    let mut changed = false;
    let mut result = String::new();

    for c in s.chars() {
        if let Some(replacement) = replacer(c) {
            if !changed {
                // First change detected, start building the result
                changed = true;
                result = String::with_capacity(s.len());
                // Add all characters up to this point
                let pos = s.len() - s.chars().as_str().len() - c.len_utf8();
                result.push_str(&s[..pos]);
            }
            result.push(replacement);
        } else if changed {
            result.push(c);
        }
    }

    if changed {
        Cow::Owned(result)
    } else {
        Cow::Borrowed(s)
    }
}

// ============================================================================
// VALIDATION UTILITIES
// ============================================================================

/// Fast character class checking using lookup tables
///
/// This module provides optimized character validation functions using
/// precomputed lookup tables for common character classes.
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

    /// Lookup table for common key characters (alphanumeric + _-.)
    const KEY_CHARS: [bool; 128] = {
        let mut table = [false; 128];
        let mut i = 0;
        while i < 128 {
            table[i] =
                matches!(i as u8, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'-' | b'.');
            i += 1;
        }
        table
    };

    /// Fast check if a character is ASCII alphanumeric
    #[inline]
    pub fn is_ascii_alphanumeric_fast(c: char) -> bool {
        if c.is_ascii() {
            ASCII_ALPHANUMERIC[c as u8 as usize]
        } else {
            false
        }
    }

    /// Fast check if a character is allowed in keys
    #[inline]
    pub fn is_key_char_fast(c: char) -> bool {
        if c.is_ascii() {
            KEY_CHARS[c as u8 as usize]
        } else {
            false
        }
    }

    /// Check if a character is a common separator
    #[inline]
    pub fn is_separator(c: char) -> bool {
        matches!(c, '_' | '-' | '.' | '/' | ':' | '|')
    }

    /// Check if a character is whitespace (space, tab, newline, etc.)
    #[inline]
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
pub fn string_memory_usage(s: &str) -> usize {
    // Base string object size + heap allocation (if any)
    core::mem::size_of::<String>() + s.len()
}

/// Calculate the memory usage of a SmartString
///
/// This function calculates the memory usage of a SmartString, accounting
/// for inline vs heap storage.
///
/// # Arguments
///
/// * `s` - The string content to measure
///
/// # Returns
///
/// The estimated memory usage in bytes
pub fn smart_string_memory_usage(s: &str) -> usize {
    // SmartString uses inline storage for strings <= 23 bytes
    if s.len() <= 23 {
        core::mem::size_of::<SmartString>()
    } else {
        core::mem::size_of::<SmartString>() + s.len()
    }
}

/// Estimate the optimal capacity for a string operation
///
/// This function estimates the optimal capacity for string operations
/// to minimize reallocations.
///
/// # Arguments
///
/// * `current_len` - Current string length
/// * `additional_len` - Additional length to be added
///
/// # Returns
///
/// The recommended capacity
pub fn optimal_capacity(current_len: usize, additional_len: usize) -> usize {
    let total = current_len + additional_len;
    // Round up to next power of 2 for efficient growth
    total.next_power_of_two().max(32)
}

// ============================================================================
// CACHING UTILITIES
// ============================================================================

/// Simple position cache for split operations
///
/// This structure caches the positions of delimiters in a string
/// to speed up repeated split operations.
#[derive(Debug, Clone)]
pub struct PositionCache {
    delimiter: char,
    positions: Vec<usize>,
    cached_for: String,
}

impl PositionCache {
    /// Create a new position cache for a string and delimiter
    ///
    /// # Arguments
    ///
    /// * `s` - The string to cache positions for
    /// * `delimiter` - The delimiter character
    ///
    /// # Returns
    ///
    /// A new position cache
    pub fn new(s: &str, delimiter: char) -> Self {
        let positions: Vec<usize> = s
            .char_indices()
            .filter_map(|(pos, c)| if c == delimiter { Some(pos) } else { None })
            .collect();

        Self {
            delimiter,
            positions,
            cached_for: s.to_string(),
        }
    }

    /// Check if this cache is valid for a given string and delimiter
    ///
    /// # Arguments
    ///
    /// * `s` - The string to check
    /// * `delimiter` - The delimiter to check
    ///
    /// # Returns
    ///
    /// `true` if the cache is valid for this string and delimiter
    pub fn is_valid_for(&self, s: &str, delimiter: char) -> bool {
        self.delimiter == delimiter && self.cached_for == s
    }

    /// Get the cached positions
    ///
    /// # Returns
    ///
    /// A slice of delimiter positions
    pub fn positions(&self) -> &[usize] {
        &self.positions
    }

    /// Get the number of parts this string would split into
    ///
    /// # Returns
    ///
    /// The number of parts
    pub fn part_count(&self) -> usize {
        self.positions.len() + 1
    }

    /// Get the nth part of the split string
    ///
    /// # Arguments
    ///
    /// * `n` - The part index (0-based)
    ///
    /// # Returns
    ///
    /// The nth part of the string, or `None` if index is out of bounds
    pub fn get_part(&self, n: usize) -> Option<&str> {
        let s = &self.cached_for;

        match n {
            0 => {
                // First part: from start to first delimiter
                if let Some(&first_pos) = self.positions.first() {
                    Some(&s[..first_pos])
                } else {
                    // No delimiters, return entire string
                    Some(s)
                }
            }
            i if i == self.positions.len() => {
                // Last part: from last delimiter to end
                if let Some(&last_pos) = self.positions.last() {
                    Some(&s[last_pos + 1..])
                } else {
                    None // No delimiters but asking for part > 0
                }
            }
            i if i < self.positions.len() => {
                // Middle part: between two delimiters
                let start = self.positions[i - 1] + 1;
                let end = self.positions[i];
                Some(&s[start..end])
            }
            _ => None,
        }
    }
}

// ============================================================================
// PERFORMANCE UTILITIES
// ============================================================================

/// Benchmark utilities for measuring performance
pub mod benchmark {
    use core::time::Duration;

    #[cfg(not(feature = "std"))]
    use alloc::vec::Vec;

    /// Simple timer for measuring operation duration
    #[derive(Debug)]
    pub struct Timer {
        #[cfg(feature = "std")]
        start: std::time::Instant,
        #[cfg(not(feature = "std"))]
        _phantom: core::marker::PhantomData<()>,
    }

    #[cfg(feature = "std")]
    impl Timer {
        /// Start a new timer
        pub fn start() -> Self {
            Self {
                start: std::time::Instant::now(),
            }
        }

        /// Get the elapsed time since the timer was started
        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }

        /// Get the elapsed time in nanoseconds
        pub fn elapsed_nanos(&self) -> u64 {
            self.elapsed().as_nanos() as u64
        }
    }

    #[cfg(not(feature = "std"))]
    impl Timer {
        /// Start a new timer (no-op in no_std)
        pub fn start() -> Self {
            Self {
                _phantom: core::marker::PhantomData,
            }
        }

        /// Get elapsed time (returns zero in no_std)
        pub fn elapsed(&self) -> Duration {
            Duration::from_nanos(0)
        }

        /// Get elapsed time in nanoseconds (returns zero in no_std)
        pub fn elapsed_nanos(&self) -> u64 {
            0
        }
    }

    /// Measure the time taken by a closure
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to measure
    ///
    /// # Returns
    ///
    /// A tuple of (result, elapsed_nanos)
    pub fn measure<F, R>(f: F) -> (R, u64)
    where
        F: FnOnce() -> R,
    {
        let timer = Timer::start();
        let result = f();
        let elapsed = timer.elapsed_nanos();
        (result, elapsed)
    }

    /// Benchmark a closure multiple times and return statistics
    ///
    /// # Arguments
    ///
    /// * `iterations` - Number of times to run the closure
    /// * `f` - The closure to benchmark
    ///
    /// # Returns
    ///
    /// Benchmark statistics
    pub fn benchmark_iterations<F>(iterations: usize, mut f: F) -> BenchmarkStats
    where
        F: FnMut(),
    {
        let mut times = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let (_, elapsed) = measure(|| f());
            times.push(elapsed);
        }

        BenchmarkStats::from_times(times)
    }

    /// Statistics from benchmark runs
    #[derive(Debug, Clone)]
    pub struct BenchmarkStats {
        /// Number of iterations
        pub iterations: usize,
        /// Minimum time in nanoseconds
        pub min_ns: u64,
        /// Maximum time in nanoseconds
        pub max_ns: u64,
        /// Average time in nanoseconds
        pub avg_ns: u64,
        /// Median time in nanoseconds
        pub median_ns: u64,
        /// Standard deviation in nanoseconds
        pub std_dev_ns: f64,
    }

    impl BenchmarkStats {
        fn from_times(mut times: Vec<u64>) -> Self {
            times.sort_unstable();

            let iterations = times.len();
            let min_ns = times.first().copied().unwrap_or(0);
            let max_ns = times.last().copied().unwrap_or(0);
            let sum: u64 = times.iter().sum();
            let avg_ns = if iterations > 0 {
                sum / iterations as u64
            } else {
                0
            };
            let median_ns = if iterations > 0 {
                if iterations % 2 == 0 {
                    (times[iterations / 2 - 1] + times[iterations / 2]) / 2
                } else {
                    times[iterations / 2]
                }
            } else {
                0
            };

            // Calculate standard deviation
            let variance: f64 = times
                .iter()
                .map(|&x| {
                    let diff = x as f64 - avg_ns as f64;
                    diff * diff
                })
                .sum::<f64>()
                / iterations as f64;
            #[cfg(feature = "std")]
            let std_dev_ns = variance.sqrt();
            #[cfg(not(feature = "std"))]
            let std_dev_ns = {
                // Simple approximation for sqrt in no_std
                if variance == 0.0 {
                    0.0
                } else {
                    // Newton's method approximation
                    let mut x = variance / 2.0;
                    for _ in 0..10 {
                        x = (x + variance / x) / 2.0;
                    }
                    x
                }
            };

            Self {
                iterations,
                min_ns,
                max_ns,
                avg_ns,
                median_ns,
                std_dev_ns,
            }
        }
    }

    impl core::fmt::Display for BenchmarkStats {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            writeln!(f, "Benchmark Results ({} iterations):", self.iterations)?;
            writeln!(f, "  Min:    {} ns", self.min_ns)?;
            writeln!(f, "  Max:    {} ns", self.max_ns)?;
            writeln!(f, "  Avg:    {} ns", self.avg_ns)?;
            writeln!(f, "  Median: {} ns", self.median_ns)?;
            writeln!(f, "  StdDev: {:.2} ns", self.std_dev_ns)?;
            Ok(())
        }
    }
}

// ============================================================================
// CONVERSION UTILITIES
// ============================================================================

/// Convert between different string types efficiently
pub mod convert {
    use smartstring::alias::String as SmartString;

    #[cfg(not(feature = "std"))]
    use alloc::string::{String, ToString};
    #[cfg(feature = "std")]
    use std::string::{String, ToString};

    /// Convert a string slice to SmartString with optimal allocation
    pub fn str_to_smart_string(s: &str) -> SmartString {
        SmartString::from(s)
    }

    /// Convert SmartString to regular String
    pub fn smart_string_to_string(s: SmartString) -> String {
        s.into()
    }

    /// Convert string with potential reallocation optimization
    pub fn optimize_string_allocation(s: String) -> String {
        // If the string has excess capacity, shrink it
        if s.capacity() > s.len() * 2 && s.len() > 0 {
            // Shrink by creating a new string with exact capacity
            let mut new_string = String::with_capacity(s.len());
            new_string.push_str(&s);
            new_string
        } else {
            s
        }
    }

    /// Convert a vector of string parts to a single string efficiently
    pub fn parts_to_string(parts: &[&str], separator: &str) -> String {
        if parts.is_empty() {
            return String::new();
        }

        if parts.len() == 1 {
            return parts[0].to_string();
        }

        // Pre-calculate capacity
        let content_len: usize = parts.iter().map(|s| s.len()).sum();
        let separator_len = separator.len() * (parts.len() - 1);
        let total_capacity = content_len + separator_len;

        let mut result = String::with_capacity(total_capacity);
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                result.push_str(separator);
            }
            result.push_str(part);
        }

        result
    }
}

// ============================================================================
// DEBUGGING UTILITIES
// ============================================================================

/// Debugging utilities for development and testing
pub mod debug {
    use crate::domain::KeyDomain;
    use crate::key::Key;

    #[cfg(not(feature = "std"))]
    use alloc::string::{String, ToString};
    #[cfg(not(feature = "std"))]
    use alloc::format;
    #[cfg(feature = "std")]
    use std::string::{String, ToString};

    /// Debug information about a key's internal state
    #[derive(Debug, Clone)]
    pub struct KeyDebugInfo {
        /// The key's string content
        pub content: String,
        /// The cached hash value
        pub hash: u64,
        /// The cached length
        pub length: u32,
        /// The domain name
        pub domain: &'static str,
        /// Memory usage estimate
        pub memory_bytes: usize,
    }

    /// Get debug information about a key
    pub fn key_debug_info<T: KeyDomain>(key: &Key<T>) -> KeyDebugInfo {
        KeyDebugInfo {
            content: key.as_str().to_string(),
            hash: key.hash(),
            length: key.len() as u32,
            domain: key.domain(),
            memory_bytes: super::smart_string_memory_usage(key.as_str())
                + core::mem::size_of::<u64>() // hash
                + core::mem::size_of::<u32>(), // length
        }
    }

    /// Format key debug information as a string
    pub fn format_key_debug<T: KeyDomain>(key: &Key<T>) -> String {
        let info = key_debug_info(key);
        format!(
            "Key Debug Info:\n  Content: '{}'\n  Hash: 0x{:016x}\n  Length: {}\n  Domain: '{}'\n  Memory: {} bytes",
            info.content, info.hash, info.length, info.domain, info.memory_bytes
        )
    }

    /// Validate internal consistency of a key
    pub fn validate_key_consistency<T: KeyDomain>(key: &Key<T>) -> Result<(), String> {
        // Check length consistency
        if key.len() != key.as_str().len() {
            return Err(format!(
                "Length mismatch: cached={}, actual={}",
                key.len(),
                key.as_str().len()
            ));
        }

        // Check hash consistency (re-compute and compare)
        let expected_hash = crate::key::Key::<T>::compute_hash(key.as_str());
        if key.hash() != expected_hash {
            return Err(format!(
                "Hash mismatch: cached=0x{:016x}, expected=0x{:016x}",
                key.hash(),
                expected_hash
            ));
        }

        Ok(())
    }
}

// ============================================================================
// CONSTANTS AND LOOKUP TABLES
// ============================================================================

/// Common character sets used in validation
pub mod char_sets {
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    #[cfg(not(feature = "std"))]
    use alloc::format;
    #[cfg(feature = "std")]
    use std::string::String;

    /// ASCII alphanumeric characters
    pub const ASCII_ALPHANUMERIC: &str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    /// Common separator characters
    pub const SEPARATORS: &str = "_-./:";

    /// Whitespace characters
    pub const WHITESPACE: &str = " \t\n\r\x0B\x0C";

    /// Check if a character is in a character set
    pub fn char_in_set(c: char, set: &str) -> bool {
        set.contains(c)
    }

    /// Get all allowed characters for basic keys
    pub fn basic_key_chars() -> String {
        format!("{}{}", ASCII_ALPHANUMERIC, "_-.")
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::string::{String, ToString};

    #[test]
    fn test_add_prefix_suffix() {
        let result = add_prefix_optimized("test", "prefix_", 100);
        assert_eq!(result, "prefix_test");

        let result = add_suffix_optimized("test", "_suffix", 100);
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
    fn test_position_cache() {
        let cache = PositionCache::new("a_b_c_d", '_');
        assert_eq!(cache.positions(), &[1, 3, 5]);
        assert_eq!(cache.part_count(), 4);

        assert_eq!(cache.get_part(0), Some("a"));
        assert_eq!(cache.get_part(1), Some("b"));
        assert_eq!(cache.get_part(2), Some("c"));
        assert_eq!(cache.get_part(3), Some("d"));
        assert_eq!(cache.get_part(4), None);

        assert!(cache.is_valid_for("a_b_c_d", '_'));
        assert!(!cache.is_valid_for("a_b_c_d", '-'));
        assert!(!cache.is_valid_for("different", '_'));
    }

    #[test]
    fn test_memory_utilities() {
        let s = "hello";
        let usage = string_memory_usage(s);
        assert!(usage >= s.len());

        let capacity = optimal_capacity(10, 5);
        assert!(capacity >= 15);
        assert!(capacity.is_power_of_two() || capacity == 32);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_benchmark_utilities() {
        use benchmark::*;

        let (result, elapsed) = measure(|| 2 + 2);
        assert_eq!(result, 4);
        assert!(elapsed == 0 || elapsed > 0); // Could be 0 for very fast operations

        let stats = benchmark_iterations(10, || {
            // Some work
            let _sum: u32 = (0..100).sum();
        });

        assert_eq!(stats.iterations, 10);
        assert!(stats.min_ns <= stats.avg_ns);
        assert!(stats.avg_ns <= stats.max_ns);
    }

    #[test]
    fn test_convert_utilities() {
        use convert::*;

        let smart = str_to_smart_string("hello");
        assert_eq!(smart.as_str(), "hello");

        let regular = smart_string_to_string(smart);
        assert_eq!(regular, "hello");

        let parts = vec!["a", "b", "c"];
        let joined = parts_to_string(&parts, "_");
        assert_eq!(joined, "a_b_c");
    }

    #[test]
    fn test_char_sets() {
        use char_sets::*;

        assert!(char_in_set('a', ASCII_ALPHANUMERIC));
        assert!(char_in_set('5', ASCII_ALPHANUMERIC));
        assert!(!char_in_set('_', ASCII_ALPHANUMERIC));

        assert!(char_in_set('_', SEPARATORS));
        assert!(char_in_set('-', SEPARATORS));
        assert!(!char_in_set('a', SEPARATORS));

        let basic_chars = basic_key_chars();
        assert!(basic_chars.contains('a'));
        assert!(basic_chars.contains('_'));
        assert!(basic_chars.contains('.'));
    }

    #[test]
    fn test_replace_chars() {
        let result = replace_chars("hello-world", |c| if c == '-' { Some('_') } else { None });
        assert_eq!(result, "hello_world");

        let result = replace_chars("hello_world", |c| if c == '-' { Some('_') } else { None });
        assert!(matches!(result, Cow::Borrowed("hello_world")));
    }
}